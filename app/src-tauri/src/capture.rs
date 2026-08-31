//! Screenshot capture and perceptual hashing — direct Rust replacement
//! for the Python `capture.py`, so the .app calls `screencapture` itself
//! and macOS TCC only needs one permission grant.

use image_hasher::{HashAlg, HasherConfig, ImageHash};
use std::path::PathBuf;
use std::process::Command;

const MAX_WIDTH: u32 = 640;

/// Take a screenshot of a specific display via `screencapture -D <n>`.
/// `display_num` is 1-based (1 = main display). Returns the PNG bytes.
pub fn take_screenshot(display_num: u32) -> Result<Vec<u8>, String> {
    let path: PathBuf = std::env::temp_dir().join(format!(
        "daymonitor-{}-{}.png",
        std::process::id(),
        chrono::Local::now().timestamp_millis()
    ));

    let status = Command::new("/usr/sbin/screencapture")
        .args(["-x", "-t", "png", "-D"])
        .arg(display_num.to_string())
        .arg(&path)
        .status()
        .map_err(|e| format!("failed to launch screencapture: {e}"))?;

    if !status.success() {
        let _ = std::fs::remove_file(&path);
        return Err(format!("screencapture exited with {status}"));
    }

    let bytes = std::fs::read(&path).map_err(|e| format!("failed to read screenshot: {e}"))?;
    let _ = std::fs::remove_file(&path);
    Ok(bytes)
}

/// Get the name of the frontmost application via System Events.
pub fn get_frontmost_app() -> Option<String> {
    let out = Command::new("/usr/bin/osascript")
        .args([
            "-e",
            r#"tell application "System Events" to get name of first process whose frontmost is true"#,
        ])
        .output()
        .ok()?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

/// Detect which display (1-based) contains the mouse cursor.
/// Falls back to 1 (main display) on any error.
pub fn get_focused_display_num() -> u32 {
    let script = r#"
ObjC.import('AppKit');
var mouse = $.NSEvent.mouseLocation;
var screens = $.NSScreen.screens;
var num = 1;
for (var i = 0; i < screens.count; i++) {
    var f = screens.objectAtIndex(i).frame;
    if (mouse.x >= f.origin.x && mouse.x < f.origin.x + f.size.width &&
        mouse.y >= f.origin.y && mouse.y < f.origin.y + f.size.height) {
        num = i + 1;
        break;
    }
}
num;
"#;
    let out = Command::new("/usr/bin/osascript")
        .args(["-l", "JavaScript", "-e", script])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .trim()
            .parse::<u32>()
            .unwrap_or(1)
            .max(1),
        Err(_) => 1,
    }
}

/// Resize a PNG so its width is at most max_width (preserving aspect ratio).
/// max_width = 0 means "don't resize, use the original".
pub fn resize_to_width(image_bytes: &[u8], max_width: u32) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| format!("decode failed: {e}"))?;
    if max_width == 0 || img.width() <= max_width {
        return Ok(image_bytes.to_vec());
    }
    let ratio = max_width as f32 / img.width() as f32;
    let new_h = (img.height() as f32 * ratio) as u32;
    let resized = img.resize(max_width, new_h, image::imageops::FilterType::Lanczos3);

    let mut out = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| format!("encode failed: {e}"))?;
    Ok(out)
}

/// Backward-compat wrapper using the default 640px width.
pub fn resize_for_api(image_bytes: &[u8]) -> Result<Vec<u8>, String> {
    resize_to_width(image_bytes, MAX_WIDTH)
}

/// Compute a perceptual hash of an image. Returns hex string for storage in SQLite.
pub fn compute_hash(image_bytes: &[u8]) -> Result<String, String> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| format!("decode failed: {e}"))?;
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Mean)
        .hash_size(8, 8)
        .to_hasher();
    let hash = hasher.hash_image(&img);
    Ok(hash.to_base64())
}

/// Hamming distance between two perceptual hashes (lower = more similar).
pub fn hash_distance(h1: &str, h2: &str) -> u32 {
    match (
        ImageHash::<Vec<u8>>::from_base64(h1),
        ImageHash::<Vec<u8>>::from_base64(h2),
    ) {
        (Ok(a), Ok(b)) => a.dist(&b),
        _ => u32::MAX, // hash parsing failed → treat as completely different
    }
}

/// No keyboard/mouse input for this long means nobody is working — skip
/// capture. The display can stay "active" overnight, which used to record
/// hours of black screens.
pub const IDLE_THRESHOLD_S: f64 = 600.0;

/// Parse the HIDIdleTime value (nanoseconds) out of `ioreg -c IOHIDSystem`
/// output and convert to seconds.
fn parse_hid_idle_seconds(ioreg_output: &str) -> Option<f64> {
    let line = ioreg_output
        .lines()
        .find(|l| l.contains("HIDIdleTime"))?;
    let ns: f64 = line.rsplit('=').next()?.trim().parse().ok()?;
    Some(ns / 1_000_000_000.0)
}

/// Seconds since the last keyboard/mouse input, via IOKit's HIDIdleTime.
/// None if it can't be determined.
pub fn seconds_since_last_input() -> Option<f64> {
    let out = Command::new("/usr/sbin/ioreg")
        .args(["-c", "IOHIDSystem", "-d", "4"])
        .output()
        .ok()?;
    parse_hid_idle_seconds(&String::from_utf8_lossy(&out.stdout))
}

/// Parse `ioreg -n Root -d1` output for the IOConsoleLocked flag.
fn parse_console_locked(ioreg_output: &str) -> bool {
    ioreg_output
        .lines()
        .find(|l| l.contains("IOConsoleLocked"))
        .map(|l| l.contains("Yes"))
        .unwrap_or(false)
}

/// True when the login session is locked (lock screen / fast user switch).
/// Catches the window between locking and the HID idle threshold, which used
/// to burn API calls on lock-screen frames. False on any error.
pub fn is_console_locked() -> bool {
    let out = Command::new("/usr/sbin/ioreg")
        .args(["-n", "Root", "-d1"])
        .output();
    match out {
        Ok(o) => parse_console_locked(&String::from_utf8_lossy(&o.stdout)),
        Err(_) => false,
    }
}

/// Best-effort screen-active check: false when the console is locked or the
/// user has been idle past IDLE_THRESHOLD_S; falls back to true on any error
/// so we don't silently stop capturing.
pub fn is_screen_active() -> bool {
    if is_console_locked() {
        return false;
    }
    match seconds_since_last_input() {
        Some(idle) if idle > IDLE_THRESHOLD_S => false,
        _ => true,
    }
}

/// True when the frame is (nearly) black — a sleeping display or an idle
/// dark terminal. Rules, in order: everything darker than 25; mean below 10;
/// mean below 20 with under 2% of pixels brighter than 60. A dark-theme
/// editor full of text has a much higher bright fraction and is not flagged.
pub fn is_black_screen(image_bytes: &[u8]) -> Result<bool, String> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| format!("decode failed: {e}"))?
        .to_luma8();
    let pixels = img.as_raw();
    if pixels.is_empty() {
        return Ok(true);
    }
    let max = *pixels.iter().max().unwrap();
    if max < 25 {
        return Ok(true);
    }
    let mean = pixels.iter().map(|&p| p as u64).sum::<u64>() as f64 / pixels.len() as f64;
    if mean < 10.0 {
        return Ok(true);
    }
    let bright_frac =
        pixels.iter().filter(|&&p| p > 60).count() as f64 / pixels.len() as f64;
    Ok(mean < 20.0 && bright_frac < 0.02)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_png(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb(color));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn resize_shrinks_large_image() {
        let big = make_png(2560, 1440, [128, 128, 128]);
        let resized = resize_for_api(&big).unwrap();
        let img = image::load_from_memory(&resized).unwrap();
        assert_eq!(img.width(), 640);
    }

    #[test]
    fn resize_keeps_small_image() {
        let small = make_png(320, 240, [128, 128, 128]);
        let resized = resize_for_api(&small).unwrap();
        let img = image::load_from_memory(&resized).unwrap();
        assert_eq!(img.width(), 320);
    }

    #[test]
    fn hash_identical_images_zero_distance() {
        let png = make_png(100, 100, [100, 100, 100]);
        let h1 = compute_hash(&png).unwrap();
        let h2 = compute_hash(&png).unwrap();
        assert_eq!(hash_distance(&h1, &h2), 0);
    }

    /// Uniform dark background with `spot_count` pixels of `spot_value` gray.
    fn make_dark_png(bg: u8, spot_value: u8, spot_count: usize) -> Vec<u8> {
        let mut img = image::GrayImage::from_pixel(100, 100, image::Luma([bg]));
        for (i, p) in img.pixels_mut().enumerate() {
            if i >= spot_count {
                break;
            }
            *p = image::Luma([spot_value]);
        }
        let mut bytes = Vec::new();
        image::DynamicImage::ImageLuma8(img)
            .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn black_screen_pure_black() {
        assert!(is_black_screen(&make_dark_png(0, 0, 0)).unwrap());
    }

    #[test]
    fn black_screen_normal_bright_image_is_not_black() {
        assert!(!is_black_screen(&make_png(100, 100, [128, 128, 128])).unwrap());
    }

    #[test]
    fn black_screen_dark_with_sparse_bright_content() {
        // bg=15 with 1% bright pixels: mean ~16.9 — the frames that used to slip through
        assert!(is_black_screen(&make_dark_png(15, 200, 100)).unwrap());
    }

    #[test]
    fn black_screen_dark_theme_editor_is_not_black() {
        // bg=15 with 8% bright text pixels: mean ~29.8 — real work
        assert!(!is_black_screen(&make_dark_png(15, 200, 800)).unwrap());
    }

    #[test]
    fn parse_hid_idle_from_ioreg_output() {
        let out = r#"  |   "HIDIdleTime" = 245000000000"#;
        assert_eq!(parse_hid_idle_seconds(out), Some(245.0));
    }

    #[test]
    fn parse_hid_idle_missing_returns_none() {
        assert_eq!(parse_hid_idle_seconds("no idle info here"), None);
    }

    #[test]
    fn parse_console_locked_yes() {
        let out = "  |   \"IOConsoleLocked\" = Yes";
        assert!(parse_console_locked(out));
    }

    #[test]
    fn parse_console_locked_no_or_missing() {
        assert!(!parse_console_locked("  |   \"IOConsoleLocked\" = No"));
        assert!(!parse_console_locked("nothing relevant here"));
    }
}
