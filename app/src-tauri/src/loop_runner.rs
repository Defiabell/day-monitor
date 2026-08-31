//! Pure-Rust monitoring loop. macOS TCC only needs to grant screen-recording
//! permission to Day Monitor.app — no external python3 process in the chain.

use crate::analyze::{analyze_screenshot, estimate_cost_usd, is_unidentifiable, Endpoint};
use crate::capture::{
    compute_hash, get_focused_display_num, get_frontmost_app, hash_distance, is_black_screen,
    is_screen_active, take_screenshot,
};
use crate::db::Db;
use crate::power;
use crate::settings;
use crate::state::AppState;
use chrono::Timelike;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

pub fn spawn(state: Arc<AppState>, api_key: String, app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Cleanup old events on startup using configured retention, and
        // rewrite any legacy category values to the v2 taxonomy (idempotent).
        let s = settings::get();
        if let Ok(db) = Db::open(Db::default_path()) {
            let _ = db.cleanup_older_than(s.retention_days);
            if !db.category_migration_done().unwrap_or(false)
                && db.remap_categories(crate::analyze::LEGACY_CATEGORY_MAP).is_ok()
            {
                let _ = db.mark_category_migration_done();
            }
        }

        let mut last_tray_title: Option<String> = None;

        // Hash of the last frame the model could not identify (black screen
        // etc.). Kept in memory only, so similar frames skip the API call
        // without ever being stored.
        let mut last_skipped_hash: Option<String> = None;

        loop {
            if !state.running.load(Ordering::Relaxed) {
                break;
            }
            let s = settings::get();
            // Keep the menu-bar number fresh even while paused/gated, so a
            // date rollover or settings change is reflected within one tick.
            update_tray_title(&app, s.show_tray_stats, &mut last_tray_title).await;
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

            match tick(&api_key, &s, &mut last_skipped_hash).await {
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

/// Refresh the tray title with today's tracked total. Only touches the tray
/// when the formatted value actually changed.
async fn update_tray_title(
    app: &tauri::AppHandle,
    show: bool,
    last: &mut Option<String>,
) {
    let title = if show {
        tauri::async_runtime::spawn_blocking(|| {
            Db::open(Db::default_path())
                .ok()
                .and_then(|db| crate::stats::today_stats(&db).ok())
                .map(|t| fmt_tray_title(t.total_seconds))
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default()
    } else {
        String::new()
    };
    if last.as_deref() == Some(title.as_str()) {
        return;
    }
    let app2 = app.clone();
    let t = title.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = app2.tray_by_id("main") {
            let _ = tray.set_title(if t.is_empty() { None } else { Some(t.as_str()) });
        }
    });
    *last = Some(title);
}

/// Menu-bar headline number: "" when nothing tracked, "45m" under an hour,
/// "3.2h" beyond.
pub fn fmt_tray_title(total_seconds: u32) -> String {
    if total_seconds == 0 {
        return String::new();
    }
    if total_seconds < 3600 {
        return format!("{}m", total_seconds / 60);
    }
    format!("{:.1}h", total_seconds as f64 / 3600.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_title_formats() {
        assert_eq!(fmt_tray_title(0), "");
        assert_eq!(fmt_tray_title(45 * 60), "45m");
        assert_eq!(fmt_tray_title(3 * 3600 + 12 * 60), "3.2h");
    }
}

/// Returns Ok(None) if a normal capture happened, Ok(Some(reason)) if skipped
/// for a non-error reason (screen off, dedup hit, budget exhausted), Err on
/// real failure.
async fn tick(
    api_key: &str,
    s: &settings::Settings,
    last_skipped_hash: &mut Option<String>,
) -> Result<Option<String>, String> {
    if !is_screen_active() {
        return Ok(Some("screen off / locked / user idle".into()));
    }

    // Blacklist gate #1 — before the screenshot is even taken. With an
    // active blacklist an unidentifiable frontmost app skips the tick
    // (fail closed): a privacy gate that can't tell who's in front must not
    // guess. The reason is user-visible in the popover.
    let detected_app = tauri::async_runtime::spawn_blocking(get_frontmost_app)
        .await
        .map_err(|e| format!("join: {e}"))?;
    match settings::blacklist_verdict(detected_app.as_deref(), &s.blacklist_apps) {
        settings::BlacklistVerdict::Block(front) => {
            return Ok(Some(format!("blacklisted app ({front})")));
        }
        settings::BlacklistVerdict::Indeterminate => {
            return Ok(Some(
                "cannot determine frontmost app (blacklist active)".into(),
            ));
        }
        settings::BlacklistVerdict::Allow => {}
    }

    let max_w = s.max_image_width;
    let raw = tauri::async_runtime::spawn_blocking(|| {
        let display = get_focused_display_num();
        take_screenshot(display)
    })
    .await
    .map_err(|e| format!("join: {e}"))??;
    let resized = tauri::async_runtime::spawn_blocking(move || {
        crate::capture::resize_to_width(&raw, max_w)
    })
    .await
    .map_err(|e| format!("join: {e}"))??;

    let black_bytes = resized.clone();
    let black = tauri::async_runtime::spawn_blocking(move || is_black_screen(&black_bytes))
        .await
        .map_err(|e| format!("join: {e}"))??;
    if black {
        return Ok(Some("black screen".into()));
    }

    let hash_bytes = resized.clone();
    let new_hash = tauri::async_runtime::spawn_blocking(move || compute_hash(&hash_bytes))
        .await
        .map_err(|e| format!("join: {e}"))??;

    if let Some(skipped) = last_skipped_hash.as_deref() {
        if hash_distance(&new_hash, skipped) < s.dedup_threshold {
            return Ok(Some("unidentifiable frame (deduped, not stored)".into()));
        }
    }

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

    // Blacklist gate #2 — re-check right before the image leaves the machine.
    // The user may have switched into a blacklisted app since gate #1
    // (classic check-then-act race: check password, tab back). Dropping a
    // safe frame here is a cheap false positive; uploading a blacklisted one
    // is not. The fresh app name is also the better analyzer hint.
    let front_now = tauri::async_runtime::spawn_blocking(get_frontmost_app)
        .await
        .map_err(|e| format!("join: {e}"))?;
    match settings::blacklist_verdict(front_now.as_deref(), &s.blacklist_apps) {
        settings::BlacklistVerdict::Block(front) => {
            return Ok(Some(format!(
                "blacklisted app focused before upload ({front}) — frame dropped"
            )));
        }
        settings::BlacklistVerdict::Indeterminate => {
            return Ok(Some(
                "cannot determine frontmost app (blacklist active) — frame dropped".into(),
            ));
        }
        settings::BlacklistVerdict::Allow => {}
    }
    let app_hint = front_now.or(detected_app);

    let endpoint = Endpoint::from_settings(&crate::settings::get());
    let result = analyze_screenshot(&resized, api_key, app_hint.as_deref(), &endpoint).await?;
    if result.category == "other" && is_unidentifiable(&result.summary) {
        *last_skipped_hash = Some(new_hash);
        return Ok(Some("unidentifiable frame (not stored)".into()));
    }
    *last_skipped_hash = None;
    let app_name = result
        .app
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(app_hint.as_deref())
        .map(|s| s.to_string());
    let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let summary = result.summary;
    let category = result.category;
    let new_hash_for_insert = new_hash.clone();
    let in_tok = result.input_tokens;
    let out_tok = result.output_tokens;
    let cfg_for_price = crate::settings::get();
    let cost = estimate_cost_usd(
        in_tok,
        out_tok,
        cfg_for_price.price_input(),
        cfg_for_price.price_output(),
    );
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
