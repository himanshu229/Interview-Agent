use tauri::{AppHandle, Manager};

use crate::error::AppResult;

/// Raises the window's native level above `NSFloatingWindowLevel` so it stays
/// visible above every other app's windows, even ones that are themselves
/// always-on-top, regardless of which app is currently focused.
#[cfg(target_os = "macos")]
pub(crate) fn pin_above_everything(window: &tauri::WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    if let Ok(ptr) = window.ns_window() {
        unsafe {
            let ns_window = ptr as *mut AnyObject;
            if let Some(ns_window) = ns_window.as_ref() {
                // One level below the screen-capture "shield" level: effectively topmost.
                let level: isize = 2_147_483_630;
                let _: () = msg_send![ns_window, setLevel: level];
            }
        }
    }
}

/// Forces the window to the front of the z-order via `SetWindowPos`, so it
/// wins over other apps' always-on-top windows instead of just ordinary ones.
#[cfg(target_os = "windows")]
pub(crate) fn pin_above_everything(window: &tauri::WebviewWindow) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };

    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

/// Re-applies GTK's "keep above" hint. Only effective under X11; Wayland
/// compositors intentionally don't let clients control their own stacking.
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub(crate) fn pin_above_everything(window: &tauri::WebviewWindow) {
    use gtk::prelude::GtkWindowExt;

    if let Ok(gtk_window) = window.gtk_window() {
        gtk_window.set_keep_above(true);
    }
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
pub(crate) fn pin_above_everything(_window: &tauri::WebviewWindow) {}


#[tauri::command]
pub async fn set_always_on_top(app: AppHandle, enabled: bool) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(enabled)?;
        // Keep the window pinned across Spaces / full-screen apps while it's on top.
        window.set_visible_on_all_workspaces(enabled)?;
        if enabled {
            pin_above_everything(&window);
        }
    }
    Ok(())
}

/// Resizes the main window's height to fit its rendered content (width stays
/// fixed). Logged so the terminal shows exactly what's requested vs. applied.
#[tauri::command]
pub async fn resize_to_content(app: AppHandle, width: f64, height: f64) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        let before = window.outer_size().ok();
        window.set_size(tauri::LogicalSize::new(width, height))?;
        let after = window.outer_size().ok();
        log::info!(
            "resize_to_content: requested {width}x{height}, size before={before:?} after={after:?}"
        );
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
