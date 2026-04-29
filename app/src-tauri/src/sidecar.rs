use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::path::PathBuf;
use std::process::{Child, Command};

pub struct SidecarManager {
    child: Option<Child>,
    last_pid: Option<u32>,
}

impl SidecarManager {
    pub fn new() -> Self {
        SidecarManager {
            child: None,
            last_pid: None,
        }
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
        self.last_pid = Some(pid);
        Ok(pid)
    }

    pub fn pid(&self) -> Option<u32> {
        self.last_pid
    }

    fn send_signal(&self, sig: Signal) -> Result<(), String> {
        let pid = self
            .child
            .as_ref()
            .ok_or("sidecar not running")?
            .id();
        kill(Pid::from_raw(pid as i32), sig)
            .map_err(|e| format!("kill({sig:?}) failed: {e}"))
    }

    pub fn pause(&self) -> Result<(), String> {
        self.send_signal(Signal::SIGUSR1)
    }

    pub fn resume(&self) -> Result<(), String> {
        self.send_signal(Signal::SIGUSR2)
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
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
