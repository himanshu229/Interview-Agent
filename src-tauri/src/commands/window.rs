use tauri::{AppHandle, Manager};
use std::sync::{Mutex, OnceLock};

use crate::error::AppResult;

#[derive(Clone, Copy)]
struct SnapGridCell {
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    row: i64,
    column: i64,
}

impl SnapGridCell {
    fn matches_monitor(&self, x: i32, y: i32, width: u32, height: u32) -> bool {
        self.monitor_x == x
            && self.monitor_y == y
            && self.monitor_width == width
            && self.monitor_height == height
    }
}

static SNAP_GRID_CELL: OnceLock<Mutex<Option<SnapGridCell>>> = OnceLock::new();

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

/// Moves the main window through a 3x3 grid on its current monitor. This runs
/// natively because global shortcuts can fire while the webview has no focus.
pub(crate) fn snap_window_in_direction(app: &AppHandle, direction: &'static str) {
    dispatch(app, move |app| {
        let Some(window) = app.get_webview_window("main") else {
            return;
        };
        let (Ok(Some(monitor)), Ok(window_position), Ok(window_size)) = (
            window.current_monitor(),
            window.outer_position(),
            window.outer_size(),
        ) else {
            log::warn!("window snap skipped because monitor geometry was unavailable");
            return;
        };

        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        let margin = 16_i64;
        let min_x = i64::from(monitor_position.x) + margin;
        let max_x = (i64::from(monitor_position.x) + i64::from(monitor_size.width)
            - i64::from(window_size.width)
            - margin)
            .max(min_x);
        let min_y = i64::from(monitor_position.y) + margin;
        let max_y = (i64::from(monitor_position.y) + i64::from(monitor_size.height)
            - i64::from(window_size.height)
            - margin)
            .max(min_y);
        let center_x = i64::from(monitor_position.x)
            + (i64::from(monitor_size.width) - i64::from(window_size.width)) / 2;
        let center_y = i64::from(monitor_position.y)
            + (i64::from(monitor_size.height) - i64::from(window_size.height)) / 2;

        let cells = SNAP_GRID_CELL.get_or_init(|| Mutex::new(None));
        let mut cells = cells.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let (row, column) = if let Some(cell) = *cells {
            if cell.matches_monitor(
                monitor_position.x,
                monitor_position.y,
                monitor_size.width,
                monitor_size.height,
            ) {
                (cell.row, cell.column)
            } else {
                grid_cell_from_position(
                    &window_position,
                    &monitor_position,
                    &monitor_size,
                    &window_size,
                )
            }
        } else {
            grid_cell_from_position(
                &window_position,
                &monitor_position,
                &monitor_size,
                &window_size,
            )
        };

        let Some((next_row, next_column)) = adjacent_grid_cell(row, column, direction) else {
            log::warn!("window snap received invalid direction: {direction}");
            return;
        };
        let x = match next_column {
            0 => min_x,
            1 => center_x,
            _ => max_x,
        };
        let y = match next_row {
            0 => min_y,
            1 => center_y,
            _ => max_y,
        };

        if let Err(e) = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32)) {
            log::warn!("window snap failed: {e}");
        } else {
            *cells = Some(SnapGridCell {
                monitor_x: monitor_position.x,
                monitor_y: monitor_position.y,
                monitor_width: monitor_size.width,
                monitor_height: monitor_size.height,
                row: next_row,
                column: next_column,
            });
            log::info!("window snap: direction={direction}, position={x}x{y}");
        }
    });
}

/// Records a window move from either a mouse drag or a native position change.
/// The next arrow shortcut will start at the nearest 3x3 cell for that position.
pub(crate) fn sync_snap_grid_cell(
    window: &tauri::Window,
    window_position: tauri::PhysicalPosition<i32>,
) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let Ok(window_size) = window.outer_size() else {
        return;
    };
    let (row, column) = grid_cell_from_position(
        &window_position,
        &monitor_position,
        &monitor_size,
        &window_size,
    );
    let cells = SNAP_GRID_CELL.get_or_init(|| Mutex::new(None));
    let mut cells = cells.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    *cells = Some(SnapGridCell {
        monitor_x: monitor_position.x,
        monitor_y: monitor_position.y,
        monitor_width: monitor_size.width,
        monitor_height: monitor_size.height,
        row,
        column,
    });
}

fn grid_cell_from_position(
    window_position: &tauri::PhysicalPosition<i32>,
    monitor_position: &tauri::PhysicalPosition<i32>,
    monitor_size: &tauri::PhysicalSize<u32>,
    window_size: &tauri::PhysicalSize<u32>,
) -> (i64, i64) {
    let margin = 16_i64;
    let min_x = i64::from(monitor_position.x) + margin;
    let max_x = (i64::from(monitor_position.x) + i64::from(monitor_size.width)
        - i64::from(window_size.width)
        - margin)
        .max(min_x);
    let min_y = i64::from(monitor_position.y) + margin;
    let max_y = (i64::from(monitor_position.y) + i64::from(monitor_size.height)
        - i64::from(window_size.height)
        - margin)
        .max(min_y);
    let center_x = i64::from(monitor_position.x)
        + (i64::from(monitor_size.width) - i64::from(window_size.width)) / 2;
    let center_y = i64::from(monitor_position.y)
        + (i64::from(monitor_size.height) - i64::from(window_size.height)) / 2;

    let column = nearest_anchor(i64::from(window_position.x), [min_x, center_x, max_x]);
    let row = nearest_anchor(i64::from(window_position.y), [min_y, center_y, max_y]);
    (row, column)
}

fn nearest_anchor(position: i64, anchors: [i64; 3]) -> i64 {
    anchors
        .iter()
        .enumerate()
        .min_by_key(|(_, anchor)| (position - **anchor).abs())
        .map(|(index, _)| index as i64)
        .unwrap_or(0)
}

fn adjacent_grid_cell(row: i64, column: i64, direction: &str) -> Option<(i64, i64)> {
    match direction {
        "up" => Some(((row - 1).max(0), column)),
        "down" => Some(((row + 1).min(2), column)),
        "left" => Some((row, (column - 1).max(0))),
        "right" => Some((row, (column + 1).min(2))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{adjacent_grid_cell, nearest_anchor};

    #[test]
    fn moves_to_an_adjacent_cell_without_changing_the_other_axis() {
        assert_eq!(adjacent_grid_cell(1, 1, "right"), Some((1, 2)));
        assert_eq!(adjacent_grid_cell(1, 1, "left"), Some((1, 0)));
        assert_eq!(adjacent_grid_cell(1, 1, "up"), Some((0, 1)));
        assert_eq!(adjacent_grid_cell(1, 1, "down"), Some((2, 1)));
    }

    #[test]
    fn clamps_at_each_grid_edge_instead_of_wrapping() {
        assert_eq!(adjacent_grid_cell(0, 0, "up"), Some((0, 0)));
        assert_eq!(adjacent_grid_cell(0, 0, "left"), Some((0, 0)));
        assert_eq!(adjacent_grid_cell(2, 2, "down"), Some((2, 2)));
        assert_eq!(adjacent_grid_cell(2, 2, "right"), Some((2, 2)));
    }

    #[test]
    fn finds_the_nearest_real_snap_anchor() {
        let anchors = [16, 972, 1928];
        assert_eq!(nearest_anchor(840, anchors), 1);
        assert_eq!(nearest_anchor(1700, anchors), 2);
        assert_eq!(nearest_anchor(100, anchors), 0);
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

