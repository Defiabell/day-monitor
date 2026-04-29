//! Pure-Rust monitoring loop. macOS TCC only needs to grant screen-recording
//! permission to Day Monitor.app — no external python3 process in the chain.

use crate::analyze::{analyze_screenshot, estimate_cost_usd};
use crate::capture::{compute_hash, hash_distance, is_screen_active, take_screenshot};
use crate::db::Db;
use crate::power;
use crate::settings;
use crate::state::AppState;
use chrono::Timelike;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

pub fn spawn(state: Arc<AppState>, api_key: String) {
    tauri::async_runtime::spawn(async move {
        // Cleanup old events on startup using configured retention.
        let s = settings::get();
        if let Ok(db) = Db::open(Db::default_path()) {
            let _ = db.cleanup_older_than(s.retention_days);
        }

        loop {
            if !state.running.load(Ordering::Relaxed) {
                break;
            }
            let s = settings::get();
            // Privacy gate.
            if !s.privacy_accepted {
                set_skip(&state, Some("privacy notice not accepted".into()));
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            // User pause.
            if state.paused.load(Ordering::Relaxed) {
                set_skip(&state, Some("paused by user".into()));
                tokio::time::sleep(Duration::from_secs(s.interval_secs.max(5) as u64)).await;
                continue;
            }
            // Active hours gate.
            let now_hour = chrono::Local::now().hour();
            if !power::within_active_hours(now_hour, s.active_hour_start, s.active_hour_end) {
                set_skip(&state, Some("outside active hours".into()));
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
            // Battery gate.
            if power::should_pause_for_battery(s.pause_on_battery_below) {
                set_skip(&state, Some("low battery".into()));
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            match tick(&api_key, &s).await {
                Ok(skipped) => {
                    set_skip(&state, skipped);
                    *state.last_error.write().unwrap() = None;
                }
                Err(e) => {
                    eprintln!("[day-monitor] tick error: {e}");
                    *state.last_error.write().unwrap() = Some(e);
                }
            }
            tokio::time::sleep(Duration::from_secs(s.interval_secs.max(5) as u64)).await;
        }
    });
}

fn set_skip(state: &Arc<AppState>, reason: Option<String>) {
    *state.skip_reason.write().unwrap() = reason;
}

/// Returns Ok(None) if a normal capture happened, Ok(Some(reason)) if skipped
/// for a non-error reason (screen off, dedup hit, budget exhausted), Err on
/// real failure.
async fn tick(api_key: &str, s: &settings::Settings) -> Result<Option<String>, String> {
    if !is_screen_active() {
        return Ok(Some("screen off / locked".into()));
    }

    let max_w = s.max_image_width;
    let raw = tauri::async_runtime::spawn_blocking(take_screenshot)
        .await
        .map_err(|e| format!("join: {e}"))??;
    let resized = tauri::async_runtime::spawn_blocking(move || {
        crate::capture::resize_to_width(&raw, max_w)
    })
    .await
    .map_err(|e| format!("join: {e}"))??;

    let hash_bytes = resized.clone();
    let new_hash = tauri::async_runtime::spawn_blocking(move || compute_hash(&hash_bytes))
        .await
        .map_err(|e| format!("join: {e}"))??;

    let last = tauri::async_runtime::spawn_blocking(|| {
        Db::open(Db::default_path())
            .map_err(|e| e.to_string())
            .and_then(|db| db.last_event().map_err(|e| e.to_string()))
    })
    .await
    .map_err(|e| format!("join: {e}"))??;

    if let Some((_, prev_hash, _)) = last {
        if hash_distance(&new_hash, &prev_hash) < s.dedup_threshold {
            let interval = s.interval_secs;
            tauri::async_runtime::spawn_blocking(move || {
                Db::open(Db::default_path())
                    .map_err(|e| e.to_string())
                    .and_then(|db| db.increment_last_duration(interval).map_err(|e| e.to_string()))
            })
            .await
            .map_err(|e| format!("join: {e}"))??;
            return Ok(Some("screen unchanged (deduped)".into()));
        }
    }

    // Budget guard: stop calling the API if user has set a budget and we're over it
    if s.monthly_budget_usd > 0.0 {
        let month_prefix = chrono::Local::now().format("%Y-%m").to_string();
        let month_cost = tauri::async_runtime::spawn_blocking(move || {
            Db::open(Db::default_path())
                .map_err(|e| e.to_string())
                .and_then(|db| {
                    db.api_call_totals_for_prefix(&month_prefix)
                        .map_err(|e| e.to_string())
                })
        })
        .await
        .map_err(|e| format!("join: {e}"))??;
        if month_cost.2 >= s.monthly_budget_usd as f64 {
            return Ok(Some(format!(
                "月度预算已用尽 (${:.2}/${:.2})",
                month_cost.2, s.monthly_budget_usd
            )));
        }
    }

    // Resize for API if not already done. resize_to_width with max_w handles this.
    let result = analyze_screenshot(&resized, api_key).await?;
    let app_name = result
        .app
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let summary = result.summary;
    let category = result.category;
    let new_hash_for_insert = new_hash.clone();
    let in_tok = result.input_tokens;
    let out_tok = result.output_tokens;
    let cost = estimate_cost_usd(in_tok, out_tok);
    let ts_clone = ts.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
        db.insert_event(
            &ts,
            &new_hash_for_insert,
            &summary,
            &category,
            app_name.as_deref(),
        )
        .map_err(|e| e.to_string())?;
        db.insert_api_call(&ts_clone, in_tok, out_tok, cost)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("join: {e}"))??;

    Ok(None)
}
