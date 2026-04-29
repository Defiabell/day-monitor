//! User-tunable settings persisted to ~/.day-monitor/config.json.
//!
//! All fields have sensible defaults; the file is created lazily.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Capture interval in seconds (10–120 reasonable).
    pub interval_secs: u32,

    /// Max screenshot width in pixels before sending to Claude. Smaller = cheaper.
    pub max_image_width: u32,

    /// How many days of events to keep in the DB. 0 = forever.
    pub retention_days: u32,

    /// Hamming distance threshold for treating two consecutive screenshots as
    /// "the same screen" and skipping the API call. Higher = skip more.
    pub dedup_threshold: u32,

    /// Soft monthly USD budget. 0 = no limit.
    pub monthly_budget_usd: f32,

    /// True once the user has accepted the privacy notice.
    pub privacy_accepted: bool,

    /// Active hours window (24h, inclusive start, exclusive end). Outside the
    /// window, capture is paused. If start == end, capture runs all day.
    /// Default: 0..24 (always on).
    #[serde(default = "default_hour_start")]
    pub active_hour_start: u32,
    #[serde(default = "default_hour_end")]
    pub active_hour_end: u32,

    /// Pause when on battery and battery level <= this percent. 0 = never pause for battery.
    #[serde(default)]
    pub pause_on_battery_below: u32,
}

fn default_hour_start() -> u32 {
    0
}
fn default_hour_end() -> u32 {
    24
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            interval_secs: 20,
            max_image_width: 640,
            retention_days: 30,
            dedup_threshold: 12,
            monthly_budget_usd: 0.0,
            privacy_accepted: false,
            active_hour_start: 0,
            active_hour_end: 24,
            pause_on_battery_below: 0,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".day-monitor")
        .join("config.json")
}

static CACHE: RwLock<Option<Settings>> = RwLock::new(None);

/// Load settings from disk (or defaults if file missing/corrupt). Cached after first read.
pub fn get() -> Settings {
    if let Some(s) = CACHE.read().unwrap().clone() {
        return s;
    }
    let s = std::fs::read_to_string(config_path())
        .ok()
        .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
        .unwrap_or_default();
    *CACHE.write().unwrap() = Some(s.clone());
    s
}

/// Persist settings to disk and update the cache.
pub fn save(s: &Settings) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    *CACHE.write().unwrap() = Some(s.clone());
    Ok(())
}

/// Force-reload from disk (used by command handlers after external edits).
pub fn invalidate_cache() {
    *CACHE.write().unwrap() = None;
}
