# Day Monitor

> macOS menu-bar app that auto-tracks your day with AI and turns it into a visual daily report.

Every ~20 seconds: screenshot → Claude Haiku labels the activity → local SQLite → dashboard with timeline, categories, trends and an AI-written daily report.

[![Release](https://img.shields.io/github/v/release/Defiabell/day-monitor)](https://github.com/Defiabell/day-monitor/releases/latest)
[![License](https://img.shields.io/github/license/Defiabell/day-monitor)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS-blue)
[![Downloads](https://img.shields.io/github/downloads/Defiabell/day-monitor/total)](https://github.com/Defiabell/day-monitor/releases)

**🎬 Live demo (sample data, right in your browser): https://defiabell.github.io/day-monitor/**

![Day Monitor dashboard demo](docs/demo.gif)

English | [简体中文](README.zh-CN.md)

## What it does

| Feature | Details |
|---------|---------|
| Auto-tracking | No timers to start/stop — runs in the background |
| AI recognition | Claude sees the screenshot and labels it: "coding in VS Code", "replying on Slack" |
| 6 dashboard views | Timeline · Categories · Trends · App ranking · Events · AI daily report |
| Cost control | Built-in token / cost stats + monthly budget cap (auto-pauses when exceeded) |
| Privacy-first | **All data stays on your machine.** Screenshots are never stored — only a one-line text description per capture is written to a local SQLite database. Point it at a local model and nothing leaves the machine at all. |
| Bring your own model | Any Anthropic-compatible endpoint: local ollama, LiteLLM, OpenRouter, or the official API |

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
- An Anthropic API key ([sign up](https://console.anthropic.com) — new accounts come with free credits), **or** any Anthropic-compatible endpoint including a local one — see [Bring your own model](#bring-your-own-model-or-run-it-fully-local)

## Privacy

> **TL;DR — everything is stored on your machine. By default the screenshot itself is sent to Anthropic for recognition and nothing else leaves; point the app at a local model and nothing leaves at all.**

- Screenshots are processed in RAM and **immediately discarded** — never written to disk or a database
- The local SQLite DB stores: timestamp + perceptual hash (for dedup, cannot be reversed into an image) + one-line text description + category
- Screenshots go to **Anthropic** (Claude Haiku) over HTTPS by default. This is the one step that leaves your machine, and you can remove it entirely by [pointing the app at a local model](#bring-your-own-model-or-run-it-fully-local)
- Nothing is ever sent to the project author — **zero telemetry**, no analytics, no crash reporting, no update ping
- Apps on the blacklist are never screenshotted at all — the capture is skipped before it happens, so there is nothing to send. If the app can't tell which window is in front while a blacklist is active, it skips the capture rather than guessing
- Everything lives under `~/.day-monitor/`; `rm -rf ~/.day-monitor` wipes it all

See [PRIVACY.md](PRIVACY.md) for full details.

## Bring your own model (or run it fully local)

Settings → **模型端点 / Model endpoint**. Leave it blank and you get the official Anthropic API, exactly as before. Fill it in and every request goes to your address instead — the bundled `claude` CLI path is skipped entirely, so nothing can silently fall back to Anthropic.

Any server that answers `POST /v1/messages` works:

| Setup | Endpoint | Model |
|-------|----------|-------|
| Local, nothing leaves the machine | `http://localhost:11434` | `qwen2.5vl:7b` (any vision model) |
| Gateway (LiteLLM, OpenRouter, …) | your gateway base URL | whatever it exposes |
| Official API (default) | *blank* | *blank* |

Notes:

- **Local needs a vision model.** The app sends an image; a text-only model will fail. [ollama](https://ollama.com) has supported the Anthropic Messages API natively since v0.14.
- The base URL is forgiving — `host`, `host/`, `host/v1` and `host/v1/messages` all resolve to the same place.
- Both `x-api-key` and `Authorization: Bearer` are sent to custom endpoints, because compatible servers disagree about which one they want.
- **Set the price fields for a paid endpoint.** The monthly budget cap is computed from token counts × price. On a custom endpoint an unset price means *free*, which is right for a local model and wrong for a paid gateway.

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

## FAQ

### Isn't this what macOS Screen Time already does?

Screen Time answers *which app* had focus. Day Monitor answers *what you were doing in it*. Two hours in Cursor comes back from Screen Time as "Cursor — 2h"; Day Monitor breaks the same two hours into "writing a React component", "chasing an API error", "reading a diff", and files each stretch under an activity category that is comparable **across** apps. Coding in a terminal and coding in an IDE land in the same bucket; watching an AI agent work lands in a different one from typing the code yourself. The end product is a written daily report, not a bar chart of app names.

### Does it send my screenshots anywhere?

By default, yes — to Anthropic, for recognition, and to nowhere else. That is the one step that leaves your machine, and it is the honest cost of getting a description instead of an app name. If that trade isn't acceptable, [point it at a local model](#bring-your-own-model-or-run-it-fully-local) and nothing leaves at all. There is no telemetry to the project author in either mode.

### Are screenshots stored on disk?

No. The image is held in memory, sent for recognition, and dropped. What lands in SQLite is a timestamp, a one-line description, a category, an app name, and a perceptual hash used to notice that the screen hasn't changed. The hash cannot be turned back into an image.

### Can I stop it from ever seeing certain apps?

Yes — the blacklist skips the screenshot *before* it is taken, so those apps produce no capture to send or store. 1Password and Keychain Access are blacklisted out of the box. If a blacklist is active and the app cannot determine which window is in front, it skips that tick rather than guessing.

### How much does it cost to run?

On the default settings (20s interval, 640px screenshots, dedup on) a normal working day lands in the low tens of US cents on Claude Haiku. The dashboard shows running token and cost stats, and a monthly budget cap hard-stops API calls when it's hit. On a local model the cost is zero.

### Can I run it without an Anthropic account?

Yes, with a local vision model through ollama, or through any gateway you already have. See [Bring your own model](#bring-your-own-model-or-run-it-fully-local).

### Does it work on Windows or Linux?

Not today — the capture layer and the menu-bar integration are macOS-specific. macOS 12+, universal binary.

### What happens to the data if I stop using it?

Everything lives under `~/.day-monitor/`. Deleting that directory removes the database, config and cached reports. There is no account and no server-side copy.

## Contributing

PRs welcome. Directions we care about:

- Intel Mac (x86_64) validation
- Linux / Windows support (needs a `screencapture` replacement)
- Multiple LLM backends (OpenAI, local models via ollama)
- i18n (prompt and UI are currently Chinese-first)
- Finer-grained settings (work hours, low-battery pause)

## License

MIT — see [LICENSE](LICENSE).
