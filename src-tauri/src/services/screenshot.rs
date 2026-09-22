use base64::Engine;
use image::{DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;
use std::path::PathBuf;
use uuid::Uuid;
use xcap::Monitor;

use crate::error::{AppError, AppResult};
use crate::models::{CaptureRegion, ScreenshotResult};

/// Captures the primary monitor in full.
pub fn capture_full_screen() -> AppResult<ScreenshotResult> {
    let monitor = primary_monitor()?;
    let image = monitor
        .capture_image()
        .map_err(|e| AppError::Screenshot(e.to_string()))?;
    persist(image)
}

/// Captures a rectangular region. Coordinates are expressed in the same
/// coordinate space as the transparent selection overlay (logical pixels of
/// the primary monitor), then scaled to physical pixels for cropping.
pub fn capture_region(region: &CaptureRegion) -> AppResult<ScreenshotResult> {
    let monitor = primary_monitor()?;
    let scale = monitor
        .scale_factor()
        .map_err(|e| AppError::Screenshot(e.to_string()))?;

    let full = monitor
        .capture_image()
        .map_err(|e| AppError::Screenshot(e.to_string()))?;

    let x = ((region.x as f32) * scale).max(0.0) as u32;
    let y = ((region.y as f32) * scale).max(0.0) as u32;
    let w = ((region.width as f32) * scale) as u32;
    let h = ((region.height as f32) * scale) as u32;

    let max_w = full.width().saturating_sub(x);
    let max_h = full.height().saturating_sub(y);
    let crop_w = w.min(max_w).max(1);
    let crop_h = h.min(max_h).max(1);

    let cropped = image::imageops::crop_imm(&full, x, y, crop_w, crop_h).to_image();
    persist(cropped)
}

fn primary_monitor() -> AppResult<Monitor> {
    let mut monitors = Monitor::all().map_err(|e| AppError::Screenshot(e.to_string()))?;
    if monitors.is_empty() {
        return Err(AppError::Screenshot("no monitor found".into()));
    }
    // Prefer the primary monitor, else fall back to the first available one.
    let idx = monitors
        .iter()
        .position(|m| m.is_primary().unwrap_or(false))
        .unwrap_or(0);
    Ok(monitors.swap_remove(idx))
}

fn persist(image: RgbaImage) -> AppResult<ScreenshotResult> {
    let width = image.width();
    let height = image.height();

    let mut buffer = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| AppError::Screenshot(e.to_string()))?;
    let bytes = buffer.into_inner();

    let path = screenshot_dir()?.join(format!("shot-{}.png", Uuid::new_v4()));
    std::fs::write(&path, &bytes)?;

    let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    Ok(ScreenshotResult {
        path: path.to_string_lossy().to_string(),
        base64,
        width,
        height,
    })
}

fn screenshot_dir() -> AppResult<PathBuf> {
    let dir = dirs::cache_dir()
        .ok_or_else(|| AppError::Screenshot("no cache dir".into()))?
        .join("ai-desktop-assistant")
        .join("screenshots");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
