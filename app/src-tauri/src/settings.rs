//! User-tunable settings persisted to ~/.day-monitor/config.json.
//!
//! All fields have sensible defaults; the file is created lazily.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

/// Built-in endpoint, model and pricing. Leaving the corresponding settings
/// blank yields exactly these, so an install that never opens the new fields
/// behaves the way it always has.
pub const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
pub const DEFAULT_API_BASE: &str = "https://api.anthropic.com";
/// Claude Haiku pricing as of 2026-04 ($/Mtoken).
pub const DEFAULT_PRICE_INPUT_PER_MTOK: f64 = 0.80;
pub const DEFAULT_PRICE_OUTPUT_PER_MTOK: f64 = 4.00;

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

    /// App names (case-insensitive substring match against the frontmost app)
    /// that must never be captured — the screenshot is skipped entirely, so
    /// nothing ever leaves the machine for these apps.
    #[serde(default = "default_blacklist")]
    pub blacklist_apps: Vec<String>,

    /// Show today's tracked total as the menu-bar tray title.
    #[serde(default = "default_true")]
    pub show_tray_stats: bool,

    /// Base URL of an **Anthropic Messages API compatible** endpoint. Blank =
    /// the official API. Anything that answers `POST /v1/messages` works: a
    /// gateway in Anthropic mode (LiteLLM, OpenRouter), or ollama >= 0.14
    /// running on this machine — which is what makes a fully local setup
    /// possible without any screenshot leaving the host.
    #[serde(default)]
    pub api_base_url: String,

    /// Model id to send. Blank = DEFAULT_MODEL.
    #[serde(default)]
    pub model: String,

    /// $/Mtoken for the configured model. See `price_input` for what 0 means —
    /// these exist because `monthly_budget_usd` is derived from token counts,
    /// so pointing at a different model without restating its price would make
    /// the budget cap quietly wrong instead of merely unknown.
    #[serde(default)]
    pub price_input_per_mtok: f64,
    #[serde(default)]
    pub price_output_per_mtok: f64,
}

impl Settings {
    /// True when the user has pointed the app somewhere other than the
    /// official API. Several behaviours key off this, not least skipping the
    /// `claude` CLI path, which deliberately forces the official endpoint.
    pub fn uses_custom_endpoint(&self) -> bool {
        !self.api_base_url.trim().is_empty()
    }

    pub fn resolved_model(&self) -> &str {
        let m = self.model.trim();
        if m.is_empty() {
            DEFAULT_MODEL
        } else {
            m
        }
    }

    /// Full Messages endpoint URL. Providers document their base in at least
    /// four shapes, so all four are accepted rather than making the user guess
    /// which one this field wants:
    ///   host, host/, host/v1, host/v1/messages
    pub fn messages_url(&self) -> String {
        let trimmed = self.api_base_url.trim().trim_end_matches('/');
        let base = if trimmed.is_empty() { DEFAULT_API_BASE } else { trimmed };
        if base.ends_with("/v1/messages") {
            base.to_string()
        } else if base.ends_with("/v1") {
            format!("{base}/messages")
        } else {
            format!("{base}/v1/messages")
        }
    }

    /// Price actually used for budget accounting.
    ///
    /// An unset price means different things on the two endpoints. On the
    /// official API it means "the built-in Haiku rate". On a custom endpoint
    /// it means **free**, because the overwhelmingly common custom endpoint is
    /// a local model: charging it phantom Haiku rates would march a local
    /// install toward `monthly_budget_usd` and eventually stop capture for a
    /// bill nobody is paying.
    pub fn price_input(&self) -> f64 {
        if self.price_input_per_mtok > 0.0 {
            self.price_input_per_mtok
        } else if self.uses_custom_endpoint() {
            0.0
        } else {
            DEFAULT_PRICE_INPUT_PER_MTOK
        }
    }

    pub fn price_output(&self) -> f64 {
        if self.price_output_per_mtok > 0.0 {
            self.price_output_per_mtok
        } else if self.uses_custom_endpoint() {
            0.0
        } else {
            DEFAULT_PRICE_OUTPUT_PER_MTOK
        }
    }
}

fn default_hour_start() -> u32 {
    0
}
fn default_hour_end() -> u32 {
    24
}
fn default_blacklist() -> Vec<String> {
    vec!["1Password".to_string(), "Keychain Access".to_string()]
}
fn default_true() -> bool {
    true
}

/// True when `app` matches any non-empty blacklist entry (case-insensitive
/// substring, so "1Password" also matches "1Password 8").
pub fn is_blacklisted(app: &str, list: &[String]) -> bool {
    let app_lower = app.to_lowercase();
    list.iter().any(|entry| {
        let e = entry.trim().to_lowercase();
        !e.is_empty() && app_lower.contains(&e)
    })
}

#[derive(Debug, PartialEq)]
pub enum BlacklistVerdict {
    Allow,
    Block(String),
    /// Blacklist is active but the frontmost app could not be determined.
    /// A privacy gate that can't tell who's in front must not guess —
    /// callers skip the tick (fail closed) with a user-visible reason.
    Indeterminate,
}

pub fn blacklist_verdict(app: Option<&str>, list: &[String]) -> BlacklistVerdict {
    let active = list.iter().any(|e| !e.trim().is_empty());
    if !active {
        return BlacklistVerdict::Allow;
    }
    match app {
        Some(a) if is_blacklisted(a, list) => BlacklistVerdict::Block(a.to_string()),
        Some(_) => BlacklistVerdict::Allow,
        None => BlacklistVerdict::Indeterminate,
    }
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
            blacklist_apps: default_blacklist(),
            show_tray_stats: true,
            api_base_url: String::new(),
            model: String::new(),
            price_input_per_mtok: 0.0,
            price_output_per_mtok: 0.0,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blacklist_matches_case_insensitive_substring() {
        let list = vec!["1Password".to_string(), "Keychain Access".to_string()];
        assert!(is_blacklisted("1Password 8", &list));
        assert!(is_blacklisted("keychain access", &list));
        assert!(!is_blacklisted("Safari", &list));
    }

    #[test]
    fn blacklist_ignores_empty_entries() {
        let list = vec!["".to_string(), "   ".to_string()];
        assert!(!is_blacklisted("Safari", &list));
    }

    #[test]
    fn default_settings_include_password_manager_blacklist() {
        let s = Settings::default();
        assert!(s.blacklist_apps.iter().any(|a| a == "1Password"));
        assert!(s.show_tray_stats);
    }

    #[test]
    fn verdict_blocks_matching_app() {
        let list = vec!["1Password".to_string()];
        assert_eq!(
            blacklist_verdict(Some("1Password 8"), &list),
            BlacklistVerdict::Block("1Password 8".to_string())
        );
    }

    #[test]
    fn verdict_allows_non_matching_app() {
        let list = vec!["1Password".to_string()];
        assert_eq!(blacklist_verdict(Some("Safari"), &list), BlacklistVerdict::Allow);
    }

    #[test]
    fn verdict_fails_closed_when_app_unknown_and_list_active() {
        let list = vec!["1Password".to_string()];
        assert_eq!(blacklist_verdict(None, &list), BlacklistVerdict::Indeterminate);
    }

    fn with_endpoint(base: &str) -> Settings {
        Settings { api_base_url: base.to_string(), ..Default::default() }
    }

    #[test]
    fn default_settings_target_the_official_endpoint_and_model() {
        let s = Settings::default();
        assert!(!s.uses_custom_endpoint());
        assert_eq!(s.resolved_model(), DEFAULT_MODEL);
        assert_eq!(s.messages_url(), "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn messages_url_accepts_every_shape_a_provider_documents() {
        for base in [
            "http://localhost:11434",
            "http://localhost:11434/",
            "http://localhost:11434/v1",
            "http://localhost:11434/v1/",
            "http://localhost:11434/v1/messages",
        ] {
            assert_eq!(
                with_endpoint(base).messages_url(),
                "http://localhost:11434/v1/messages",
                "base {base} produced the wrong URL"
            );
        }
    }

    #[test]
    fn blank_or_whitespace_endpoint_is_not_custom() {
        assert!(!with_endpoint("   ").uses_custom_endpoint());
        assert_eq!(with_endpoint("   ").messages_url(), "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn model_override_applies_and_blank_falls_back() {
        let s = Settings { model: "qwen2.5vl:7b".to_string(), ..Default::default() };
        assert_eq!(s.resolved_model(), "qwen2.5vl:7b");
        let blank = Settings { model: "  ".to_string(), ..Default::default() };
        assert_eq!(blank.resolved_model(), DEFAULT_MODEL);
    }

    #[test]
    fn unset_price_is_haiku_on_official_api_but_free_on_a_custom_endpoint() {
        // The distinction matters: billing a local model at Haiku rates would
        // walk it into monthly_budget_usd and stop capture over a phantom bill.
        let official = Settings::default();
        assert_eq!(official.price_input(), DEFAULT_PRICE_INPUT_PER_MTOK);
        assert_eq!(official.price_output(), DEFAULT_PRICE_OUTPUT_PER_MTOK);

        let local = with_endpoint("http://localhost:11434");
        assert_eq!(local.price_input(), 0.0);
        assert_eq!(local.price_output(), 0.0);
    }

    #[test]
    fn explicit_price_wins_on_either_endpoint() {
        let s = Settings {
            api_base_url: "https://gw.example/v1".to_string(),
            price_input_per_mtok: 0.25,
            price_output_per_mtok: 1.25,
            ..Default::default()
        };
        assert_eq!(s.price_input(), 0.25);
        assert_eq!(s.price_output(), 1.25);
    }

    #[test]
    fn config_written_before_these_fields_existed_still_loads() {
        // Real ~/.day-monitor/config.json files predate the endpoint fields;
        // if serde rejected them every existing user would silently reset.
        let legacy = r#"{"interval_secs":30,"max_image_width":800,"retention_days":30,
            "dedup_threshold":12,"monthly_budget_usd":5.0,"privacy_accepted":true}"#;
        let s: Settings = serde_json::from_str(legacy).expect("legacy config must still parse");
        assert_eq!(s.interval_secs, 30);
        assert!(!s.uses_custom_endpoint());
        assert_eq!(s.resolved_model(), DEFAULT_MODEL);
        assert_eq!(s.price_input(), DEFAULT_PRICE_INPUT_PER_MTOK);
    }

    #[test]
    fn verdict_allows_unknown_app_when_list_empty_or_blank() {
        assert_eq!(blacklist_verdict(None, &[]), BlacklistVerdict::Allow);
        let blank = vec!["".to_string(), "   ".to_string()];
        assert_eq!(blacklist_verdict(None, &blank), BlacklistVerdict::Allow);
    }
}
