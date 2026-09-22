use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::TranscriptSegment;
use crate::services::audio::{start_capture, AudioSession, CaptureKind};
use crate::services::openai::OpenAiClient;
use crate::state::AppState;

#[tauri::command]
pub async fn start_mic_transcription(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    start(app, &state, CaptureKind::Microphone).await
}

#[tauri::command]
pub async fn stop_mic_transcription(state: State<'_, AppState>) -> AppResult<()> {
    if let Some(session) = state.mic_session.lock().take() {
        session.stop();
    }
    Ok(())
}

#[tauri::command]
pub async fn start_system_audio_transcription(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    start(app, &state, CaptureKind::SystemAudio).await
}

#[tauri::command]
pub async fn stop_system_audio_transcription(state: State<'_, AppState>) -> AppResult<()> {
    if let Some(session) = state.system_session.lock().take() {
        session.stop();
    }
    Ok(())
}

async fn start(
    app: AppHandle,
    state: &AppState,
    kind: CaptureKind,
) -> AppResult<()> {
    // Already running? No-op.
    {
        let slot = match kind {
            CaptureKind::Microphone => state.mic_session.lock(),
            CaptureKind::SystemAudio => state.system_session.lock(),
        };
        if slot.as_ref().map(|s| s.is_running()).unwrap_or(false) {
            return Ok(());
        }
    }

    let client = state.openai_client()?;
    let model = state.settings.read().whisper_model.clone();

    let session = Arc::new(start_capture(kind)?);

    match kind {
        CaptureKind::Microphone => *state.mic_session.lock() = Some(session.clone()),
        CaptureKind::SystemAudio => *state.system_session.lock() = Some(session.clone()),
    }

    spawn_transcription_loop(app, session, client, model, kind);
    Ok(())
}

/// Background loop: repeatedly drains audio chunks and transcribes them,
/// emitting `transcript-segment` events to the frontend.
fn spawn_transcription_loop(
    app: AppHandle,
    session: Arc<AudioSession>,
    client: OpenAiClient,
    model: String,
    kind: CaptureKind,
) {
    tauri::async_runtime::spawn(async move {
        let source = kind.source_label().to_string();

        while session.is_running() {
            match session.take_chunk_wav() {
                Some(wav) => {
                    transcribe_and_emit(&app, &client, &model, wav, &source).await;
                }
                None => tokio::time::sleep(Duration::from_millis(400)).await,
            }
        }

        // Flush trailing audio after stop.
        if let Some(wav) = session.take_remaining_wav() {
            transcribe_and_emit(&app, &client, &model, wav, &source).await;
        }
    });
}

async fn transcribe_and_emit(
    app: &AppHandle,
    client: &OpenAiClient,
    model: &str,
    wav: Vec<u8>,
    source: &str,
) {
    match client.transcribe(model, wav).await {
        Ok(text) if !text.trim().is_empty() => {
            let segment = TranscriptSegment {
                id: Uuid::new_v4().to_string(),
                text: text.trim().to_string(),
                source: source.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                is_final: true,
            };
            if let Err(e) = app.emit("transcript-segment", segment) {
                log::error!("failed to emit transcript segment: {e}");
            }
        }
        Ok(_) => {}
        Err(e) => log::warn!("transcription failed: {e}"),
    }
}
