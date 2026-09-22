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
        settings.has_api_key = SecretStore::has_api_key();

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .user_agent("ai-desktop-assistant/1.0")
            .build()?;

        Ok(AppState {
            db: Mutex::new(db),
            settings: RwLock::new(settings),
            http,
            mic_session: Mutex::new(None),
            system_session: Mutex::new(None),
            config_path,
        })
    }

    pub fn settings_snapshot(&self) -> AppSettings {
        let mut s = self.settings.read().clone();
        s.has_api_key = SecretStore::has_api_key();
        s
    }

    pub fn update_settings(&self, mut settings: AppSettings) -> AppResult<()> {
        settings.has_api_key = SecretStore::has_api_key();
        {
            let mut guard = self.settings.write();
            *guard = settings.clone();
        }
        persist_settings(&self.config_path, &settings)
    }

    /// Builds an OpenAI client using the current settings and the API key from
    /// the OS keychain.
    pub fn openai_client(&self) -> AppResult<OpenAiClient> {
        let api_key = SecretStore::get_api_key()?.ok_or(AppError::MissingApiKey)?;
        let base_url = self.settings.read().base_url.clone();
        Ok(OpenAiClient::new(self.http.clone(), base_url, api_key))
    }
}

fn load_settings(path: &PathBuf) -> AppSettings {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let mut settings: AppSettings = serde_json::from_str(&text).unwrap_or_default();
            if settings.global_shortcut == "CommandOrControl+Shift+Space" {
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
