use tauri::{AppHandle, Manager};

use crate::error::AppResult;

/// Raises the window's native level above `NSFloatingWindowLevel` so it stays
/// visible above every other app's windows, even ones that are themselves
/// always-on-top, regardless of which app is currently focused.
#[cfg(target_os = "macos")]
pub(crate) fn pin_above_everything(window: &tauri::WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    match window.ns_window() {
        Ok(ptr) => unsafe {
            let ns_window = ptr as *mut AnyObject;
            if let Some(ns_window) = ns_window.as_ref() {
                // One level below the screen-capture "shield" level: effectively topmost.
                let level: isize = 2_147_483_630;
                let _: () = msg_send![ns_window, setLevel: level];
                let confirmed: isize = msg_send![ns_window, level];
                log::info!("pin_above_everything: requested level={level}, confirmed={confirmed}");
            } else {
                log::warn!("pin_above_everything: ns_window pointer was null");
            }
        },
        Err(e) => log::warn!("pin_above_everything: failed to get ns_window: {e}"),
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

/// Runs `f` on the OS main thread. AppKit/GTK window calls (`pin_above_everything`,
/// `set_size`, `set_position`, ...) can abort the process if invoked off the
/// main thread, but Tauri's `async fn` commands run on a background (tokio)
/// thread. Every command below that touches a `WebviewWindow` is funneled
/// through here instead of calling window APIs directly.
fn dispatch(app: &AppHandle, f: impl FnOnce(&AppHandle) + Send + 'static) {
    let app_clone = app.clone();
    if let Err(e) = app.run_on_main_thread(move || f(&app_clone)) {
        log::warn!("failed to schedule window task on main thread: {e}");
    }
}

/// Resizes the main window's height to fit its rendered content (width stays
/// fixed). Logged so the terminal shows exactly what's requested vs. applied.
#[tauri::command]
pub async fn resize_to_content(app: AppHandle, width: f64, height: f64) -> AppResult<()> {
    dispatch(&app, move |app| {
        if let Some(window) = app.get_webview_window("main") {
            let before = window.outer_size().ok();
            let _ = window.set_size(tauri::LogicalSize::new(width, height));
            let after = window.outer_size().ok();
            log::info!(
                "resize_to_content: requested {width}x{height}, size before={before:?} after={after:?}"
            );
        }
    });
    Ok(())
}

/// Hides/shows the window from screen capture, recording, and screen sharing
/// (Teams, Zoom, Google Meet). On macOS this sets `NSWindowSharingNone`; on
/// Windows it uses `WDA_EXCLUDEFROMCAPTURE`. The window stays visible to the
/// local user.
#[tauri::command]
pub async fn set_content_protection(app: AppHandle, enabled: bool) -> AppResult<()> {
    dispatch(&app, move |app| {
        for label in ["main", "overlay"] {
            if let Some(window) = app.get_webview_window(label) {
                let _ = window.set_content_protected(enabled);
            }
        }
    });
    Ok(())
}

