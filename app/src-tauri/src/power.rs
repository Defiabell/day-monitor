//! Power and time-of-day gates for the capture loop.

use std::process::Command;

/// True if the current local hour is inside the configured active window.
/// `start == end` is treated as "always active".
pub fn within_active_hours(now_hour: u32, start: u32, end: u32) -> bool {
    if start == end {
        return true;
    }
    if start < end {
        // Normal window, e.g., 9..22
        now_hour >= start && now_hour < end
    } else {
        // Wraps midnight, e.g., 22..6
        now_hour >= start || now_hour < end
    }
}

/// Best-effort battery check via `pmset`. Returns (percent, on_battery).
/// On any error, reports (100, false) so capture continues.
pub fn battery_status() -> (u32, bool) {
    let out = match Command::new("/usr/bin/pmset").arg("-g").arg("batt").output() {
        Ok(o) => o,
        Err(_) => return (100, false),
    };
    let s = String::from_utf8_lossy(&out.stdout);
    // Format example:
    //   Now drawing from 'Battery Power'
    //    -InternalBattery-0 (id=...)  84%; discharging; 4:23 remaining present: true
    let on_battery = s.contains("Battery Power");
    let pct = s
        .split_whitespace()
        .find_map(|tok| tok.strip_suffix('%').and_then(|p| p.parse::<u32>().ok()))
        .unwrap_or(100);
    (pct, on_battery)
}

/// True if the loop should pause because the user's battery is below the
/// configured threshold AND we're on battery (i.e., not plugged in).
/// `threshold == 0` disables this gate.
pub fn should_pause_for_battery(threshold: u32) -> bool {
    if threshold == 0 {
        return false;
    }
    let (pct, on_battery) = battery_status();
    on_battery && pct <= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_day_window() {
        assert!(within_active_hours(0, 0, 0));
        assert!(within_active_hours(13, 0, 0));
        assert!(within_active_hours(23, 0, 0));
    }

    #[test]
    fn normal_window() {
        // 9..18: active 9..17
        assert!(!within_active_hours(8, 9, 18));
        assert!(within_active_hours(9, 9, 18));
        assert!(within_active_hours(17, 9, 18));
        assert!(!within_active_hours(18, 9, 18));
    }

    #[test]
    fn wraparound_window() {
        // 22..6: active 22, 23, 0..5
        assert!(within_active_hours(22, 22, 6));
        assert!(within_active_hours(0, 22, 6));
        assert!(within_active_hours(5, 22, 6));
        assert!(!within_active_hours(6, 22, 6));
        assert!(!within_active_hours(12, 22, 6));
    }
}
