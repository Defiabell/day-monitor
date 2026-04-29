use std::path::PathBuf;
use std::process::Command;

const LEGACY_PLIST_NAME: &str = "com.daymonitor.plist";

fn legacy_plist_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join("Library")
        .join("LaunchAgents")
        .join(LEGACY_PLIST_NAME)
}

pub fn legacy_launchd_present() -> bool {
    legacy_plist_path().exists()
}

pub fn remove_legacy_launchd() -> Result<(), String> {
    let path = legacy_plist_path();
    if !path.exists() {
        return Ok(());
    }
    let _ = Command::new("launchctl")
        .args(["unload", path.to_str().unwrap_or_default()])
        .output();
    std::fs::remove_file(&path).map_err(|e| format!("rm plist failed: {e}"))
}
