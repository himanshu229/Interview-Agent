mod commands;
mod db;
mod error;
mod models;
mod services;
mod state;
mod tray;

#[cfg(desktop)]
mod shortcuts;

use tauri::{Manager, WindowEvent};

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(
                tauri_plugin_log::Builder::new()
                    .level(log::LevelFilter::Info)
                    .build(),
            )
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(shortcuts::plugin());
    }

    builder
        .setup(|app| {
            // Initialise shared state (DB, settings, HTTP client).
            let data_dir = app.path().app_data_dir()?;
            let state = AppState::new(data_dir)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

            let shortcut = state.settings.read().global_shortcut.clone();
            let always_on_top = state.settings.read().always_on_top;
            let content_protection = state.settings.read().content_protection;

            app.manage(state);

            // System tray.
            tray::build_tray(app.handle())?;

            // Global shortcut (desktop only).
            #[cfg(desktop)]
            shortcuts::register(app.handle(), &shortcut);

            // Apply persisted window preferences.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_always_on_top(always_on_top);
                // Stay pinned above other apps' windows even across Spaces / full-screen apps.
                let _ = window.set_visible_on_all_workspaces(true);
                if always_on_top {
                    commands::window::pin_above_everything(&window);
                }
                // Hide the window from screen capture / sharing when enabled.
                let _ = window.set_content_protected(content_protection);

                // Open at the top-center of the primary monitor instead of full-center.
                if let (Ok(Some(monitor)), Ok(size)) =
                    (window.current_monitor(), window.outer_size())
                {
                    let margin: i32 = 16;
                    let mon_pos = monitor.position();
                    let mon_size = monitor.size();
                    let x = mon_pos.x
                        + ((mon_size.width as i32 - size.width as i32) / 2).max(0);
                    let y = mon_pos.y + margin;
                    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                }

                // Window is created hidden (see tauri.conf.json) so it never flashes
                // at the OS default position before the repositioning above applies.
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window hides it to the tray instead of quitting.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
            // Re-assert the topmost z-order whenever focus moves to another app,
            // so a newly-opened app can't bury this window behind it.
            if let WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let app = window.app_handle();
                    let always_on_top = app.state::<AppState>().settings.read().always_on_top;
                    if always_on_top {
                        if let Some(webview) = app.get_webview_window("main") {
                            commands::window::pin_above_everything(&webview);
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::chat::send_chat,
            commands::history::list_conversations,
            commands::history::get_messages,
            commands::history::create_conversation,
            commands::history::rename_conversation,
            commands::history::delete_conversation,
            commands::screenshot::capture_full_screen,
            commands::screenshot::capture_region,
            commands::screenshot::run_ocr,
            commands::screenshot::start_region_selection,
            commands::screenshot::ocr_and_maybe_send,
            commands::documents::extract_pdf_text,
            commands::audio::start_mic_transcription,
            commands::audio::stop_mic_transcription,
            commands::audio::start_system_audio_transcription,
            commands::audio::stop_system_audio_transcription,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::set_api_key,
            commands::settings::clear_api_key,
            commands::settings::test_api_key,
            commands::window::set_always_on_top,
            commands::window::set_content_protection,
            commands::window::toggle_floating_window,
            commands::window::resize_to_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AI Desktop Assistant");
}
