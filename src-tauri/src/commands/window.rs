use tauri::{AppHandle, Manager};

use crate::error::AppResult;

#[tauri::command]
pub async fn set_always_on_top(app: AppHandle, enabled: bool) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(enabled)?;
    }
    Ok(())
}

/// Hides/shows the window from screen capture, recording, and screen sharing
/// (Teams, Zoom, Google Meet). On macOS this sets `NSWindowSharingNone`; on
/// Windows it uses `WDA_EXCLUDEFROMCAPTURE`. The window stays visible to the
/// local user.
#[tauri::command]
pub async fn set_content_protection(app: AppHandle, enabled: bool) -> AppResult<()> {
    for label in ["main", "overlay"] {
        if let Some(window) = app.get_webview_window(label) {
            window.set_content_protected(enabled)?;
        }
    }
    Ok(())
}

/// Shows/hides the main window, mimicking a floating assistant toggle.
#[tauri::command]
pub async fn toggle_floating_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide()?;
        } else {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}
