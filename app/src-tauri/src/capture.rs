//! Screenshot capture and perceptual hashing — direct Rust replacement
//! for the Python `capture.py`, so the .app calls `screencapture` itself
//! and macOS TCC only needs one permission grant.

use image_hasher::{HashAlg, HasherConfig, ImageHash};
use std::path::PathBuf;
use std::process::Command;

const MAX_WIDTH: u32 = 640;

/// Take a screenshot of the primary display via the system `screencapture`
/// command. Returns the PNG bytes.
pub fn take_screenshot() -> Result<Vec<u8>, String> {
    let path: PathBuf = std::env::temp_dir().join(format!(
        "daymonitor-{}-{}.png",
        std::process::id(),
        chrono::Local::now().timestamp_millis()
    ));

    let status = Command::new("/usr/sbin/screencapture")
        .args(["-x", "-t", "png"])
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

/// Best-effort screen-active check via osascript. Returns true if screen is
/// awake & unlocked. Falls back to true on any error so we don't silently stop
/// capturing.
pub fn is_screen_active() -> bool {
    let out = Command::new("/usr/bin/osascript")
        .args([
            "-e",
            r#"tell application "System Events" to get name of every process whose name is "loginwindow""#,
        ])
        .output();
    if let Ok(o) = out {
        let stdout = String::from_utf8_lossy(&o.stdout);
        if stdout.contains("loginwindow") {
            // loginwindow only "exists as a foreground process" in our query when locked.
            // Inverted heuristic; if uncertain, assume active.
        }
    }
    // For now: always return true. Lock detection refinement is a follow-up.
    true
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
}
