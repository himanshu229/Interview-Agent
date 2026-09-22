use tauri::State;

use crate::error::AppResult;
use crate::models::{ChatRequest, ChatResponse, Role};
use crate::services::openai::WireMessage;
use crate::state::AppState;

/// Sends a prompt to ChatGPT within a conversation, persisting both the user
/// prompt and the assistant reply. Creates a new conversation when needed.
#[tauri::command]
pub async fn send_chat(
    request: ChatRequest,
    state: State<'_, AppState>,
) -> AppResult<ChatResponse> {
    let system_prompt = state.settings.read().system_prompt.clone();
    let model = state.settings.read().model.clone();
    let client = state.openai_client()?;

    // --- DB phase 1: persist user message and gather history (no await held) ---
    let (conversation_id, wire_history) = {
        let db = state.db.lock();

        let conversation_id = match request.conversation_id {
            Some(id) => id,
            None => {
                let title = derive_title(&request.prompt);
                db.create_conversation(&title)?.id
            }
        };

        db.insert_message(conversation_id, &Role::User, &request.prompt, &request.source)?;

        let mut wire = vec![WireMessage::new(&Role::System, system_prompt)];
        for msg in db.get_messages(conversation_id)? {
            wire.push(WireMessage::new(&msg.role, msg.content));
        }
        (conversation_id, wire)
    };

    // --- Network phase: no locks held across await ---
    let reply = client.chat(&model, &wire_history).await?;

    // --- DB phase 2: persist assistant reply ---
    let message = {
        let db = state.db.lock();
        db.insert_message(conversation_id, &Role::Assistant, &reply, "chat")?
    };

    Ok(ChatResponse {
        conversation_id,
        message,
    })
}

fn derive_title(prompt: &str) -> String {
    let trimmed = prompt.trim();
    let title: String = trimmed.chars().take(48).collect();
    if title.is_empty() {
        "New conversation".to_string()
    } else if trimmed.chars().count() > 48 {
        format!("{title}…")
    } else {
        title
    }
}
