use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

pub struct AppState {
    pub paused: AtomicBool,
    pub running: AtomicBool,
    /// The most recent loop-tick error (None if last tick succeeded).
    pub last_error: RwLock<Option<String>>,
    /// Reason capture is currently auto-skipped (off-hours, battery, screen off, etc.).
    /// None when actively capturing.
    pub skip_reason: RwLock<Option<String>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            paused: AtomicBool::new(false),
            running: AtomicBool::new(true),
            last_error: RwLock::new(None),
            skip_reason: RwLock::new(None),
        }
    }
}
