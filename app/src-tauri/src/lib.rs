mod commands;
mod db;
mod migration;
mod report;
mod sidecar;
mod state;
mod stats;

use std::path::PathBuf;
use tauri::menu::{Menu, MenuItem};
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
    // Position near the top-right of the primary monitor (where the menu bar lives)
    let position = if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let popover_w = (200.0 * scale) as i32;
        // 30px from right edge, 30px from top
        let x = size.width as i32 - popover_w - 30;
        let y = 30;
        Some(tauri::PhysicalPosition::new(x, y))
    } else {
        None
    };

    let mut builder =
        WebviewWindowBuilder::new(app, "popover", WebviewUrl::App("index.html".into()))
            .title("Day Monitor")
            .inner_size(200.0, 300.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false);
    if let Some(p) = position {
        builder = builder.position(p.x as f64, p.y as f64);
    }
    if let Ok(window) = builder.build() {
        let _ = window.show();
        let _ = window.set_focus();
    }
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

            // Tray right-click menu
            let open_dashboard_item =
                MenuItem::with_id(app, "open_dashboard", "Open Dashboard", true, None::<&str>)?;
            let open_settings_item =
                MenuItem::with_id(app, "open_settings", "Settings…", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, Some("Cmd+Q"))?;
            let menu = Menu::with_items(
                app,
                &[&open_dashboard_item, &open_settings_item, &quit_item],
            )?;

            // Tray icon — reuse the app's default window icon
            let icon = app
                .default_window_icon()
                .ok_or("no default window icon")?
                .clone();
            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open_dashboard" => {
                        let h = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = commands::open_dashboard(h).await;
                        });
                    }
                    "open_settings" => {
                        let h = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = commands::open_settings(h).await;
                        });
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
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
