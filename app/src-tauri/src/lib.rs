mod analyze;
mod capture;
mod commands;
mod db;
mod loop_runner;
mod migration;
mod power;
mod report;
mod settings;
mod state;
mod stats;

use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::state::AppState;

fn read_api_key() -> Option<String> {
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        if !k.is_empty() {
            return Some(k);
        }
    }
    let env_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".day-monitor")
        .join(".env");
    let content = std::fs::read_to_string(&env_path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("ANTHROPIC_API_KEY=") {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn show_popover_at_tray(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("popover") {
        // tauri-plugin-positioner: place under the tray icon
        let _ = window.as_ref().window().move_window(Position::TrayCenter);
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let builder = WebviewWindowBuilder::new(app, "popover", WebviewUrl::App("index.html".into()))
        .title("Day Monitor")
        .inner_size(200.0, 300.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false);
    if let Ok(window) = builder.build() {
        let _ = window.as_ref().window().move_window(Position::TrayCenter);
        let _ = window.show();
        let _ = window.set_focus();
        // Auto-hide when the popover loses focus (user clicked elsewhere).
        let app_handle = app.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::Focused(false) = event {
                if let Some(w) = app_handle.get_webview_window("popover") {
                    let _ = w.hide();
                }
            }
        });
    }
}

fn toggle_popover(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("popover") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }
    }
    show_popover_at_tray(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_positioner::init())
        .manage(Arc::new(AppState::new()))
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
                    // Tell positioner where the tray icon is so it can place popovers under it.
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
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

            // Start the in-process monitoring loop (pure Rust — no sidecar)
            let state: tauri::State<Arc<AppState>> = app.state();
            let state_arc: Arc<AppState> = state.inner().clone();
            match read_api_key() {
                Some(key) => {
                    loop_runner::spawn(state_arc, key);
                }
                None => {
                    eprintln!(
                        "[day-monitor] ANTHROPIC_API_KEY not set — open Settings to configure"
                    );
                }
            }

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
            commands::get_app_trends,
            commands::get_events,
            commands::list_categories,
            commands::generate_ai_report,
            commands::open_dashboard,
            commands::open_settings,
            commands::save_api_key,
            commands::get_api_key_set,
            commands::get_settings,
            commands::save_settings,
            commands::get_cost_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
