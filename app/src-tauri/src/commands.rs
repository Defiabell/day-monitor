use crate::db::Db;
use crate::migration;
use crate::state::AppState;
use crate::stats::{self, TodayStats};
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::State;

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
