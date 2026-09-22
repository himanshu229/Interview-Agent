use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

/// Builds the system-tray icon with a context menu and click handlers.
pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Assistant", true, None::<&str>)?;
    let screenshot =
        MenuItem::with_id(app, "screenshot", "Capture Screenshot", true, None::<&str>)?;
    let transcript =
        MenuItem::with_id(app, "transcript", "Live Transcript", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show, &screenshot, &transcript, &settings, &separator, &quit],
    )?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("app must have a default window icon");

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("AI Desktop Assistant")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => reveal_main(app),
            "screenshot" => {
                reveal_main(app);
                let _ = app.emit("navigate-tab", "screenshot");
            }
            "transcript" => {
                reveal_main(app);
                let _ = app.emit("navigate-tab", "transcript");
            }
            "settings" => {
                reveal_main(app);
                let _ = app.emit("navigate-tab", "settings");
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                reveal_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn reveal_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
