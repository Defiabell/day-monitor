# Day Monitor App (Tauri)

Tauri 2 menu bar app wrapping the day-monitor Python loop. Click the tray icon → popover with today's stats and a Pause button.

## Development

```bash
cd app
yarn install
yarn tauri dev
```

Wait ~30s for first compile. A tray icon appears in the menu bar (top right). Click → popover opens.

The app spawns `python3 loop_entry.py` as a sidecar; the loop writes to `~/.day-monitor/monitor.db`. The Tauri Rust backend reads the same DB to render the popover.

## Architecture

```
[Menu Bar Icon] --click-> [Popover Window 200x300]
                              |
                              v Tauri invoke()
[Rust backend] --reads--> [~/.day-monitor/monitor.db]
       |
       +-signals--> [Python sidecar (loop_entry.py)]
                       |
                       +-writes--> [SQLite DB]
```

## Tauri commands (Rust -> TS)

| command | returns |
|---------|---------|
| `get_today_stats` | `TodayStats` (total seconds, by-category breakdown, current activity) |
| `get_status` | `MonitorStatus` (recording / paused / error) |
| `toggle_pause` | new status (sends SIGUSR1/2 to sidecar) |
| `check_legacy_launchd` | bool (is the old com.daymonitor.plist present?) |
| `remove_legacy_launchd` | unloads + deletes the plist |
| `quit_app` | exits the app |

## Plan Status

- [x] Plan 1 (Foundation): tray + popover + sidecar control + Rust backend
- [ ] Plan 2 (Dashboard + Packaging): full dashboard window with 6 views, AI report, .app bundle, Login Item auto-start

## Tests

```bash
# Python (unchanged from CLI tool)
cd .. && python -m pytest -q

# Rust unit tests (db + stats aggregations)
cd app/src-tauri && cargo test --lib
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
