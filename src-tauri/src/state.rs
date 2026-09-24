use parking_lot::{Mutex, RwLock};
use std::path::PathBuf;
use std::sync::Arc;

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::services::audio::AudioSession;
use crate::services::openai::OpenAiClient;
use crate::services::secrets::SecretStore;

/// Global, thread-safe application state shared across all Tauri commands.
pub struct AppState {
    pub db: Mutex<Database>,
    pub settings: RwLock<AppSettings>,
    api_key: RwLock<Option<String>>,
    pub http: reqwest::Client,
    pub mic_session: Mutex<Option<Arc<AudioSession>>>,
    pub system_session: Mutex<Option<Arc<AudioSession>>>,
    config_path: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&data_dir)?;

        let db = Database::open(&data_dir.join("assistant.db"))?;
        let config_path = data_dir.join("settings.json");
        let mut settings = load_settings(&config_path);
        // Read the keychain once per app lifetime. Re-reading for every audio
        // chunk triggers repeated macOS Keychain permission prompts.
        let api_key = SecretStore::get_api_key().ok().flatten();
        settings.has_api_key = api_key.is_some();

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .user_agent("ai-desktop-assistant/1.0")
            .build()?;

        Ok(AppState {
            db: Mutex::new(db),
            settings: RwLock::new(settings),
            api_key: RwLock::new(api_key),
            http,
            mic_session: Mutex::new(None),
            system_session: Mutex::new(None),
            config_path,
        })
    }

    pub fn settings_snapshot(&self) -> AppSettings {
        let mut s = self.settings.read().clone();
        s.has_api_key = self.api_key.read().is_some();
        s
    }

    pub fn update_settings(&self, mut settings: AppSettings) -> AppResult<()> {
        settings.has_api_key = self.api_key.read().is_some();
        {
            let mut guard = self.settings.write();
            *guard = settings.clone();
        }
        persist_settings(&self.config_path, &settings)
    }

    /// Builds an OpenAI client using the current settings and the API key from
    /// the in-memory cache populated from the OS keychain at startup.
    pub fn openai_client(&self) -> AppResult<OpenAiClient> {
        let api_key = self.api_key.read().clone().ok_or(AppError::MissingApiKey)?;
        let base_url = self.settings.read().base_url.clone();
        Ok(OpenAiClient::new(self.http.clone(), base_url, api_key))
    }

    pub fn cache_api_key(&self, api_key: Option<String>) {
        *self.api_key.write() = api_key;
    }
}

fn load_settings(path: &PathBuf) -> AppSettings {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let mut settings: AppSettings = serde_json::from_str(&text).unwrap_or_default();
            let legacy_defaults = [
                "CommandOrControl+Shift+Space",
                "CommandOrControl+Shift+K",
            ];
            if settings.global_shortcut.trim().is_empty()
                || legacy_defaults.contains(&settings.global_shortcut.as_str())
            {
                settings.global_shortcut = crate::models::default_global_shortcut();
            }
            settings
        }
        Err(_) => AppSettings::default(),
    }
}

fn persist_settings(path: &PathBuf, settings: &AppSettings) -> AppResult<()> {
    let text = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, text)?;
    Ok(())
}
