use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::error::{AppError, AppResult};
use crate::models::{CaptureRegion, OcrResult, ScreenshotResult};
use crate::services::{ocr, screenshot};
use crate::state::AppState;

#[tauri::command]
pub async fn capture_full_screen() -> AppResult<ScreenshotResult> {
    tauri::async_runtime::spawn_blocking(screenshot::capture_full_screen)
        .await
        .map_err(|e| AppError::Screenshot(e.to_string()))?
}

#[tauri::command]
pub async fn capture_region(region: CaptureRegion) -> AppResult<ScreenshotResult> {
    tauri::async_runtime::spawn_blocking(move || screenshot::capture_region(&region))
        .await
        .map_err(|e| AppError::Screenshot(e.to_string()))?
}

#[tauri::command]
pub async fn run_ocr(
    screenshot_path: String,
    language: String,
) -> AppResult<OcrResult> {
    tauri::async_runtime::spawn_blocking(move || ocr::extract_text(&screenshot_path, &language))
        .await
        .map_err(|e| AppError::Ocr(e.to_string()))?
}

/// Opens a transparent, full-screen overlay window that lets the user drag to
/// select a screen region. The overlay captures the region and emits
/// `region-captured` back to the main window.
#[tauri::command]
pub async fn start_region_selection(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("Select area")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .fullscreen(true)
        .resizable(false)
        .content_protected(true)
        .build()?;

    Ok(())
}

/// Runs OCR on a screenshot and, when `auto_send_ocr` is enabled, forwards the
/// extracted text to the chat flow via the `chat-message` event.
#[tauri::command]
pub async fn ocr_and_maybe_send(
    app: AppHandle,
    screenshot_path: String,
    state: State<'_, AppState>,
) -> AppResult<OcrResult> {
    let (language, auto_send) = {
        let s = state.settings.read();
        (s.ocr_language.clone(), s.auto_send_ocr)
    };

    let path = screenshot_path.clone();
    let result: OcrResult =
        tauri::async_runtime::spawn_blocking(move || ocr::extract_text(&path, &language))
            .await
            .map_err(|e| AppError::Ocr(e.to_string()))??;

    if auto_send && !result.text.trim().is_empty() {
        use tauri::Emitter;
        let payload = serde_json::json!({
            "prompt": format!(
                "Please analyse this text captured from my screen:\n\n{}",
                result.text
            ),
            "source": "ocr",
        });
        app.emit("chat-message", payload)?;
        app.emit("ocr-sent", ())?;
    }

    Ok(result)
}
