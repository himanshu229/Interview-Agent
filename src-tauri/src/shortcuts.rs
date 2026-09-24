use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::Builder as ShortcutBuilder;

/// Builds the global-shortcut plugin.
pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    ShortcutBuilder::new().build()
}

/// Registers the fixed shortcut that collapses the application to its on-screen
/// icon or restores it,
/// plus the fixed Cmd/Ctrl+Arrow shortcuts that snap the window across the
/// 3x3 grid of screen anchor points. Called again on every settings save, so
/// the snap shortcuts are re-registered here too (`unregister_all` wipes them).
pub fn register(app: &AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let shortcut = crate::models::default_global_shortcut();
    if let Err(e) = gs.on_shortcut(shortcut.as_str(), |app, _, event| {
        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            toggle_main(app);
        }
    }) {
        log::warn!("failed to register application shortcut '{shortcut}': {e}");
    }

    register_window_snap_shortcuts(app);
    register_clear_shortcuts(app);
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

/// Clear shortcuts are global so they work regardless of which app control had
/// keyboard focus when the user pressed them.
fn register_clear_shortcuts(app: &AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gs = app.global_shortcut();
    let shortcuts = [
        ("CommandOrControl+Backspace", "clear-transcript"),
        ("CommandOrControl+Delete", "clear-transcript"),
        ("CommandOrControl+Shift+Backspace", "clear-chat"),
        ("CommandOrControl+Shift+Delete", "clear-chat"),
    ];
    for (accelerator, event_name) in shortcuts {
        if let Err(e) = gs.on_shortcut(accelerator, move |app, _, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let _ = app.emit(event_name, ());
            }
        }) {
            log::warn!("failed to register clear shortcut '{accelerator}': {e}");
        }
    }
}

fn toggle_main<R: tauri::Runtime>(app: &AppHandle<R>) {
    crate::commands::window::toggle_compact_window(app);
}
