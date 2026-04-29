use crate::db::{Db, Event};
use crate::migration;
use crate::report;
use crate::state::AppState;
use crate::stats::{self, AppUsage, TimelineSegment, TodayStats, TrendDay};
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{Manager, State};

#[derive(Debug, Serialize)]
pub struct MonitorStatus {
    pub state: String,
    pub message: Option<String>,
    pub pid: Option<u32>,
}

#[tauri::command]
pub async fn get_today_stats() -> Result<TodayStats, String> {
    let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
    stats::today_stats(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<MonitorStatus, String> {
    let mut sm = state.sidecar.lock().unwrap();
    let alive = sm.is_alive();
    let pid = sm.pid();
    if !alive {
        return Ok(MonitorStatus {
            state: "error".into(),
            message: Some("sidecar not running".into()),
            pid: None,
        });
    }
    if state.paused.load(Ordering::Relaxed) {
        Ok(MonitorStatus {
            state: "paused".into(),
            message: None,
            pid,
        })
    } else {
        Ok(MonitorStatus {
            state: "recording".into(),
            message: None,
            pid,
        })
    }
}

#[tauri::command]
pub async fn toggle_pause(state: State<'_, AppState>) -> Result<MonitorStatus, String> {
    {
        let sm = state.sidecar.lock().unwrap();
        let was_paused = state.paused.load(Ordering::Relaxed);
        if was_paused {
            sm.resume()?;
            state.paused.store(false, Ordering::Relaxed);
        } else {
            sm.pause()?;
            state.paused.store(true, Ordering::Relaxed);
        }
    }
    get_status(state).await
}

#[tauri::command]
pub async fn check_legacy_launchd() -> bool {
    migration::legacy_launchd_present()
}

#[tauri::command]
pub async fn remove_legacy_launchd() -> Result<(), String> {
    migration::remove_legacy_launchd()
}

#[tauri::command]
pub async fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub async fn get_timeline(date: Option<String>) -> Result<Vec<TimelineSegment>, String> {
    let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
    let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    stats::timeline_for_date(&db, &date).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_trends(days: u32) -> Result<Vec<TrendDay>, String> {
    let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
    stats::trends(&db, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_ranking(days: u32) -> Result<Vec<AppUsage>, String> {
    let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
    stats::app_ranking(&db, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_events(
    date: String,
    search: Option<String>,
    category: Option<String>,
) -> Result<Vec<Event>, String> {
    let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
    let mut events = db.events_for_date(&date).map_err(|e| e.to_string())?;
    if let Some(s) = search.filter(|s| !s.is_empty()) {
        let s = s.to_lowercase();
        events.retain(|e| e.summary.to_lowercase().contains(&s));
    }
    if let Some(c) = category.filter(|c| !c.is_empty() && c != "all") {
        events.retain(|e| e.category == c);
    }
    Ok(events)
}

#[tauri::command]
pub async fn list_categories() -> Result<Vec<String>, String> {
    let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
    db.distinct_categories().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_ai_report(date: String, force: Option<bool>) -> Result<String, String> {
    let api_key = read_api_key()?;
    let events = {
        let db = Db::open(Db::default_path()).map_err(|e| e.to_string())?;
        db.events_for_date(&date).map_err(|e| e.to_string())?
    };
    report::generate_report(events, &date, &api_key, force.unwrap_or(false)).await
}

#[tauri::command]
pub async fn open_dashboard(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    // Switch to Regular activation policy so the window is focusable
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }
    let window =
        WebviewWindowBuilder::new(&app, "dashboard", WebviewUrl::App("dashboard.html".into()))
            .title("Day Monitor")
            .inner_size(1100.0, 700.0)
            .resizable(true)
            .visible(true)
            .build()
            .map_err(|e| e.to_string())?;
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
pub async fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);

    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    let window =
        WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
            .title("Day Monitor Settings")
            .inner_size(420.0, 360.0)
            .resizable(false)
            .visible(true)
            .build()
            .map_err(|e| e.to_string())?;
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

fn read_api_key() -> Result<String, String> {
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        if !k.is_empty() {
            return Ok(k);
        }
    }
    let env_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".day-monitor")
        .join(".env");
    if let Ok(content) = std::fs::read_to_string(&env_path) {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("ANTHROPIC_API_KEY=") {
                return Ok(rest.trim().trim_matches('"').to_string());
            }
        }
    }
    Err("ANTHROPIC_API_KEY not set".into())
}

#[tauri::command]
pub async fn save_api_key(key: String) -> Result<(), String> {
    let env_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".day-monitor")
        .join(".env");
    if let Some(parent) = env_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&env_path, format!("ANTHROPIC_API_KEY={}\n", key)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_api_key_set() -> bool {
    read_api_key().is_ok()
}
