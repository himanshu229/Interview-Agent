use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::Builder as ShortcutBuilder;

/// Builds the global-shortcut plugin. Shortcut actions are registered after
/// settings load because their accelerators are user-configurable.
pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    ShortcutBuilder::new().build()
}

/// Registers one shortcut that toggles the application hidden/visible state,
/// plus the fixed Cmd/Ctrl+Arrow shortcuts that snap the window across the
/// 3x3 grid of screen anchor points. Called again on every settings save, so
/// the snap shortcuts are re-registered here too (`unregister_all` wipes them).
pub fn register(app: &AppHandle, shortcut: &str) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let shortcut = shortcut.trim();
    if !shortcut.is_empty() {
        if let Err(e) = gs.on_shortcut(shortcut, |app, _, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                toggle_main(app);
            }
        }) {
            log::warn!("failed to register application shortcut '{shortcut}': {e}");
        }
    }

    register_window_snap_shortcuts(app);
}

/// Grid-snap shortcuts work globally (not just when the window has focus)
/// since this app deliberately never steals focus on its own. Each one only
/// moves the native main window, so it doesn't depend on webview focus/event
/// delivery and all other key combinations are untouched.
fn register_window_snap_shortcuts(app: &AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gs = app.global_shortcut();
    let directions = [
        ("CommandOrControl+Up", "up"),
        ("CommandOrControl+Down", "down"),
        ("CommandOrControl+Left", "left"),
        ("CommandOrControl+Right", "right"),
    ];
    for (accelerator, direction) in directions {
        if let Err(e) = gs.on_shortcut(accelerator, move |app, _, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                crate::commands::window::snap_window_in_direction(app, direction);
            }
        }) {
            log::warn!("failed to register window-snap shortcut '{accelerator}': {e}");
        }
    }
}

fn toggle_main<R: tauri::Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        let focused = window.is_focused().unwrap_or(false);
        if visible && focused {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
            let _ = app.emit("navigate-tab", "chat");
        }
    }
}
