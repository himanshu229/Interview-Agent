use tauri::State;

use crate::error::AppResult;
use crate::models::{ChatMessage, Conversation};
use crate::state::AppState;

#[tauri::command]
pub async fn list_conversations(state: State<'_, AppState>) -> AppResult<Vec<Conversation>> {
    let db = state.db.lock();
    db.list_conversations()
}

#[tauri::command]
pub async fn get_messages(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> AppResult<Vec<ChatMessage>> {
    let db = state.db.lock();
    db.get_messages(conversation_id)
}

#[tauri::command]
pub async fn create_conversation(
    title: String,
    state: State<'_, AppState>,
) -> AppResult<Conversation> {
    let db = state.db.lock();
    db.create_conversation(&title)
}

#[tauri::command]
pub async fn rename_conversation(
    conversation_id: i64,
    title: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let db = state.db.lock();
    db.rename_conversation(conversation_id, &title)
}

#[tauri::command]
pub async fn delete_conversation(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let db = state.db.lock();
    db.delete_conversation(conversation_id)
}
