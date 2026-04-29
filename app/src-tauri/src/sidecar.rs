use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::path::PathBuf;
use std::process::{Child, Command};

pub struct SidecarManager {
    child: Option<Child>,
}

fn pid_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".day-monitor")
        .join("monitor.pid")
}

/// Read the real loop PID written by loop_entry.py. The pyinstaller --onefile launcher
/// is a different process from the Python loop child, so we can't reliably target
/// the actual loop using Child::id().
fn read_loop_pid() -> Option<u32> {
    std::fs::read_to_string(pid_file())
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

fn process_alive(pid: u32) -> bool {
    // signal 0 doesn't deliver, just checks permission/existence
    kill(Pid::from_raw(pid as i32), None).is_ok()
}

impl SidecarManager {
    pub fn new() -> Self {
        SidecarManager { child: None }
    }

    pub fn start(&mut self, sidecar_path: &PathBuf) -> Result<u32, String> {
        if self.child.is_some() {
            return Err("sidecar already running".into());
        }
        let child = Command::new(sidecar_path)
            .spawn()
            .map_err(|e| format!("failed to spawn sidecar at {:?}: {e}", sidecar_path))?;
        let pid = child.id();
        self.child = Some(child);
        Ok(pid)
    }

    pub fn pid(&self) -> Option<u32> {
        // Prefer the real loop PID written by loop_entry.py
        read_loop_pid().or_else(|| self.child.as_ref().map(|c| c.id()))
    }

    fn send_signal(&self, sig: Signal) -> Result<(), String> {
        let pid = read_loop_pid().ok_or("sidecar not running")?;
        kill(Pid::from_raw(pid as i32), sig)
            .map_err(|e| format!("kill({sig:?}, pid={pid}) failed: {e}"))
    }

    pub fn pause(&self) -> Result<(), String> {
        self.send_signal(Signal::SIGUSR1)
    }

    pub fn resume(&self) -> Result<(), String> {
        self.send_signal(Signal::SIGUSR2)
    }

    pub fn stop(&mut self) -> Result<(), String> {
        // First try to stop the actual loop via PID file
        if let Some(loop_pid) = read_loop_pid() {
            let _ = kill(Pid::from_raw(loop_pid as i32), Signal::SIGTERM);
        }
        // Then reap the launcher child
        if let Some(mut child) = self.child.take() {
            let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        // Trust the PID file: if the real loop process is alive, sidecar is healthy
        if let Some(loop_pid) = read_loop_pid() {
            if process_alive(loop_pid) {
                return true;
            }
        }
        // Fall back to the launcher child status
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    self.child = None;
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
