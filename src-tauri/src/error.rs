use serde::Serialize;

/// Application-wide error type. Implements `Serialize` so it can be returned
/// directly from Tauri commands and surfaced to the frontend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("no OpenAI API key configured. Add one in Settings.")]
    MissingApiKey,

    #[error("OpenAI API error: {0}")]
    OpenAi(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("keychain error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("screenshot error: {0}")]
    Screenshot(String),

    #[error("OCR error: {0}")]
    Ocr(String),

    #[error("audio error: {0}")]
    Audio(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("window error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Other(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
