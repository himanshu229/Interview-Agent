use tauri::{AppHandle, State};

use crate::error::AppResult;
use crate::models::AppSettings;
use crate::services::secrets::SecretStore;
use crate::state::AppState;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    Ok(state.settings_snapshot())
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.update_settings(settings.clone())?;
    #[cfg(desktop)]
    crate::shortcuts::register(&app, &settings.global_shortcut);
    Ok(())
}

#[tauri::command]
pub async fn set_api_key(api_key: String, state: State<'_, AppState>) -> AppResult<()> {
    SecretStore::set_api_key(api_key.trim())?;
    let mut settings = state.settings_snapshot();
    settings.has_api_key = true;
    state.update_settings(settings)
}

#[tauri::command]
pub async fn clear_api_key(state: State<'_, AppState>) -> AppResult<()> {
    SecretStore::clear_api_key()?;
    let mut settings = state.settings_snapshot();
    settings.has_api_key = false;
    state.update_settings(settings)
}

#[tauri::command]
pub async fn test_api_key(state: State<'_, AppState>) -> AppResult<bool> {
    let client = state.openai_client()?;
    client.validate().await
}
