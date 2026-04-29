use crate::sidecar::SidecarManager;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub struct AppState {
    pub sidecar: Mutex<SidecarManager>,
    pub paused: AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            sidecar: Mutex::new(SidecarManager::new()),
            paused: AtomicBool::new(false),
        }
    }
}
