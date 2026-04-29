mod commands;
mod db;
mod migration;
mod report;
mod sidecar;
mod state;
mod stats;

use std::path::PathBuf;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use crate::state::AppState;

fn sidecar_path() -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    if cfg!(debug_assertions) {
        // Dev: spawn the un-packaged Python via wrapper script
        let proj = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let wrapper = proj.join(".dev_sidecar.sh");
        if !wrapper.exists() {
            let py = proj.join("loop_entry.py");
            let _ = std::fs::write(
                &wrapper,
                format!("#!/bin/sh\nexec python3 \"{}\"\n", py.display()),
            );
            let _ = std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755));
        }
        wrapper
    } else {
        // Prod: bundled sidecar binary (added in Plan 2 packaging)
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("daymonitor-loop")))
            .unwrap_or_else(|| PathBuf::from("daymonitor-loop"))
    }
}

fn toggle_popover(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("popover") {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "popover", WebviewUrl::App("index.html".into()))
        .title("Day Monitor")
        .inner_size(200.0, 300.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState::new())
        .setup(|app| {
            // Hide dock icon on macOS — pure menu bar app
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Tray icon — reuse the app's default window icon
            let icon = app
                .default_window_icon()
                .ok_or("no default window icon")?
                .clone();
            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_popover(tray.app_handle());
                    }
                })
                .build(app)?;

            // Spawn the Python sidecar
            let sp = sidecar_path();
            let state: tauri::State<AppState> = app.state();
            state
                .sidecar
                .lock()
                .unwrap()
                .start(&sp)
                .map_err(|e| format!("failed to start sidecar: {e}"))?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_today_stats,
            commands::get_status,
            commands::toggle_pause,
            commands::check_legacy_launchd,
            commands::remove_legacy_launchd,
            commands::quit_app,
            commands::get_timeline,
            commands::get_trends,
            commands::get_app_ranking,
            commands::get_events,
            commands::list_categories,
            commands::generate_ai_report,
            commands::open_dashboard,
            commands::open_settings,
            commands::save_api_key,
            commands::get_api_key_set,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
