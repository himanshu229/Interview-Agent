use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::Builder as ShortcutBuilder;

/// Builds the global-shortcut plugin. Shortcut actions are registered after
/// settings load because their accelerators are user-configurable.
pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    ShortcutBuilder::new().build()
}

/// Registers one shortcut that toggles the application hidden/visible state.
pub fn register(app: &AppHandle, shortcut: &str) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    if let Err(e) = gs.on_shortcut(shortcut, |app, _, event| {
        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            toggle_main(app);
        }
    }) {
        log::warn!("failed to register application shortcut '{shortcut}': {e}");
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
