use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            _ => Role::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: Role,
    pub content: String,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub conversation_id: Option<i64>,
    pub prompt: String,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "chat".to_string()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub conversation_id: i64,
    pub message: ChatMessage,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotResult {
    pub path: String,
    pub base64: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
    pub screenshot_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub id: String,
    pub text: String,
    pub source: String,
    pub created_at: String,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub model: String,
    pub whisper_model: String,
    pub base_url: String,
    pub theme: String,
    pub launch_at_startup: bool,
    #[serde(default = "default_global_shortcut")]
    pub global_shortcut: String,
    pub ocr_language: String,
    pub system_prompt: String,
    pub auto_send_ocr: bool,
    #[serde(default = "default_true")]
    pub content_protection: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub has_api_key: bool,
}

fn default_true() -> bool {
    true
}

fn default_opacity() -> f64 {
    0.85
}

pub fn default_global_shortcut() -> String {
    // Avoid "Command/Control+H": that's macOS's own system-wide "hide app"
    // gesture, so it was too easy to accidentally hide the whole window.
    "CommandOrControl+Shift+K".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            model: "gpt-4o-mini".to_string(),
            whisper_model: "whisper-1".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            theme: "dark".to_string(),
            launch_at_startup: false,
            global_shortcut: default_global_shortcut(),
            ocr_language: "eng".to_string(),
            system_prompt:
                "You are a helpful AI desktop assistant. Answer concisely and clearly."
                    .to_string(),
            auto_send_ocr: true,
            content_protection: true,
            opacity: 0.85,
            has_api_key: false,
        }
    }
}
