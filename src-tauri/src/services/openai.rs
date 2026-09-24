use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::models::Role;

/// A single chat message in the OpenAI wire format.
#[derive(Debug, Clone, Serialize)]
pub struct WireMessage {
    pub role: String,
    pub content: String,
}

impl WireMessage {
    pub fn new(role: &Role, content: impl Into<String>) -> Self {
        WireMessage {
            role: role.as_str().to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: &'a [WireMessage],
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

/// Stateless client for the OpenAI REST API. Reuses a shared `reqwest::Client`.
#[derive(Clone)]
pub struct OpenAiClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl OpenAiClient {
    pub fn new(http: reqwest::Client, base_url: String, api_key: String) -> Self {
        OpenAiClient {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    /// Returns the authenticated WebSocket endpoint used for a persistent
    /// OpenAI Realtime transcription session.
    pub fn realtime_connection(&self) -> AppResult<(String, String)> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }

        let websocket_base = self
            .base_url
            .trim_end_matches("/v1")
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1);
        Ok((
            format!(
                "{websocket_base}/v1/realtime?model=gpt-4o-mini-realtime-preview"
            ),
            self.api_key.clone(),
        ))
    }

    /// Sends a chat completion request and returns the assistant reply text.
    pub async fn chat(&self, model: &str, messages: &[WireMessage]) -> AppResult<String> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }

        let body = ChatCompletionRequest {
            model,
            messages,
            temperature: 0.7,
        };

        let resp = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            let message = serde_json::from_str::<ApiErrorBody>(&text)
                .map(|b| b.error.message)
                .unwrap_or_else(|_| text.clone());
            return Err(AppError::OpenAi(format!("{status}: {message}")));
        }

        let parsed: ChatCompletionResponse = serde_json::from_str(&text)?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| AppError::OpenAi("empty response from model".into()))
    }

    /// Lightweight validation call used by the "Test" button in Settings.
    pub async fn validate(&self) -> AppResult<bool> {
        if self.api_key.is_empty() {
            return Err(AppError::MissingApiKey);
        }
        let resp = self
            .http
            .get(format!("{}/models", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
}
