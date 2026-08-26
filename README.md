# Day Monitor

> macOS menu-bar app that auto-tracks your day with AI and turns it into a visual daily report.

Every ~20 seconds: screenshot → Claude Haiku labels the activity → local SQLite → dashboard with timeline, categories, trends and an AI-written daily report.

[![Release](https://img.shields.io/github/v/release/Defiabell/day-monitor)](https://github.com/Defiabell/day-monitor/releases/latest)
[![License](https://img.shields.io/github/license/Defiabell/day-monitor)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS-blue)
[![Downloads](https://img.shields.io/github/downloads/Defiabell/day-monitor/total)](https://github.com/Defiabell/day-monitor/releases)

**🎬 Live demo (sample data, right in your browser): https://defiabell.github.io/day-monitor/**

English | [简体中文](README.zh-CN.md)

## What it does

| Feature | Details |
|---------|---------|
| Auto-tracking | No timers to start/stop — runs in the background |
| AI recognition | Claude sees the screenshot and labels it: "coding in VS Code", "replying on Slack" |
| 6 dashboard views | Timeline · Categories · Trends · App ranking · Events · AI daily report |
| Cost control | Built-in token / cost stats + monthly budget cap (auto-pauses when exceeded) |
| Privacy-first | **All data stays on your machine.** Screenshots are never stored — only a one-line text description per capture is written to a local SQLite database. |

## Screenshots

**Menu-bar popover** — today's progress at a glance, pause or open the dashboard:

<img src="docs/screenshots/popover.png" width="200" alt="Popover"/>

**Dashboard · Timeline** — the whole day as colored segments plus an activity table:

<img src="docs/screenshots/dashboard_timeline.png" width="800" alt="Dashboard — Timeline"/>

**Dashboard · Categories** — where the hours went:

<img src="docs/screenshots/dashboard_categories.png" width="800" alt="Dashboard — Categories"/>

**Dashboard · AI Report** — a daily work summary written by Claude:

<img src="docs/screenshots/dashboard_aireport.png" width="800" alt="Dashboard — AI Report"/>

## Install

**Download** (easiest): grab the `.dmg` or `.zip` from [Releases](https://github.com/Defiabell/day-monitor/releases/latest) (universal binary). The app is not notarized — on first launch, right-click the icon → Open, then confirm.

**Homebrew:**

```bash
brew tap defiabell/tap
brew install --cask day-monitor
```

**Build from source:**

```bash
git clone https://github.com/Defiabell/day-monitor
cd day-monitor

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# frontend deps + dev mode
cd app && yarn install
yarn tauri dev

# release build → app/src-tauri/target/release/bundle/macos/Day Monitor.app
yarn tauri build
```

## Requirements

- macOS 12+ (releases ship universal binaries; primarily tested on Apple Silicon)
- An Anthropic API key ([sign up](https://console.anthropic.com) — new accounts come with free credits)

## Privacy

> **TL;DR — your data never leaves your machine. Only screenshots are sent to Anthropic for activity recognition.**

- Screenshots are processed in RAM and **immediately discarded** — never written to disk or a database
- The local SQLite DB stores: timestamp + perceptual hash (for dedup, cannot be reversed into an image) + one-line text description + category
- Screenshots go to **Anthropic** (Claude Haiku) over HTTPS; nothing is ever sent to the project author — zero telemetry
- Everything lives under `~/.day-monitor/`; `rm -rf ~/.day-monitor` wipes it all

See [PRIVACY.md](PRIVACY.md) for full details.

## Cost control

Day Monitor calls a paid API, so it treats cost as a first-class feature: per-day token/cost stats live in the dashboard, and a monthly budget cap auto-pauses API calls when exceeded. Tunables in Settings:

| Setting | Default | Notes |
|---------|---------|-------|
| Capture interval | 20s | shorter = finer data, longer = cheaper |
| Screenshot resolution | 640px | directly ~50% of token cost |
| Dedup threshold | 12 | skip captures when the screen barely changed |
| Data retention | 30 days | old rows auto-cleaned |
| Monthly budget | 0 (unlimited) | hard stop for API spend |

## Architecture

```
┌────────────────────────────────────────────────┐
│  Day Monitor.app  (Tauri 2 / Rust + React)      │
│                                                   │
│  Menu Bar ──click──> Popover (200×300)          │
│      │                                           │
│      ├──> Dashboard (1100×700, 6 views)         │
│      └──> Settings (interval, budget, etc.)     │
│                                                   │
│  Rust Backend                                    │
│   ├─ tokio loop: capture → hash → analyze       │
│   ├─ /usr/sbin/screencapture (native)           │
│   ├─ Claude Haiku via reqwest                   │
│   └─ rusqlite (~/.day-monitor/monitor.db)       │
└────────────────────────────────────────────────┘
```

The live demo is this same dashboard running in the browser with a deterministic mock data source ([app/src/lib/mockApi.ts](app/src/lib/mockApi.ts)) swapped in when the Tauri runtime is absent.

## Development

Seed 7 days of mock data locally instead of waiting a week:

```bash
python3 scripts/seed_mock_data.py            # fill without overwriting
python3 scripts/seed_mock_data.py --reset --days 30
```

## Contributing

PRs welcome. Directions we care about:

- Intel Mac (x86_64) validation
- Linux / Windows support (needs a `screencapture` replacement)
- Multiple LLM backends (OpenAI, local models via ollama)
- i18n (prompt and UI are currently Chinese-first)
- Finer-grained settings (work hours, low-battery pause)

## License

MIT — see [LICENSE](LICENSE).
