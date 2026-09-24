use std::sync::Arc;
use std::time::Duration;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::TranscriptSegment;
use crate::services::audio::{start_capture, AudioSession, CaptureKind};
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

    let realtime_connection = state.openai_client()?.realtime_connection()?;

    let session = Arc::new(start_capture(kind)?);

    match kind {
        CaptureKind::Microphone => *state.mic_session.lock() = Some(session.clone()),
        CaptureKind::SystemAudio => *state.system_session.lock() = Some(session.clone()),
    }

    spawn_realtime_transcription_loop(app, session, realtime_connection, kind);
    Ok(())
}

/// Streams microphone/system PCM to one OpenAI Realtime WebSocket instead of
/// creating a new HTTP transcription request for every audio chunk.
fn spawn_realtime_transcription_loop(
    app: AppHandle,
    session: Arc<AudioSession>,
    realtime_connection: (String, String),
    kind: CaptureKind,
) {
    tauri::async_runtime::spawn(async move {
        let source = kind.source_label().to_string();
        let (url, api_key) = realtime_connection;
        let mut request = match url.into_client_request() {
            Ok(request) => request,
            Err(e) => {
                log::warn!("could not build Realtime request: {e}");
                return;
            }
        };
        let headers = request.headers_mut();
        let Ok(authorization) = HeaderValue::from_str(&format!("Bearer {api_key}")) else {
            log::warn!("could not encode Realtime authorization header");
            return;
        };
        headers.insert("Authorization", authorization);
        headers.insert("OpenAI-Beta", HeaderValue::from_static("realtime=v1"));

        let (socket, _) = match connect_async(request).await {
            Ok(connection) => connection,
            Err(e) => {
                log::warn!("could not connect to OpenAI Realtime: {e}");
                return;
            }
        };
        let (mut writer, mut reader) = socket.split();
        let configuration = json!({
            "type": "session.update",
            "session": {
                "modalities": ["text"],
                "input_audio_format": "pcm16",
                "input_audio_transcription": { "model": "gpt-4o-mini-transcribe" },
                "turn_detection": { "type": "server_vad" }
            }
        });
        if let Err(e) = writer.send(Message::Text(configuration.to_string())).await {
            log::warn!("could not configure OpenAI Realtime: {e}");
            return;
        }

        let segment_id = Uuid::new_v4().to_string();
        let mut partial = String::new();
        let mut interval = tokio::time::interval(Duration::from_millis(80));

        while session.is_running() {
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(pcm) = session.take_realtime_pcm16() {
                        let packet = json!({
                            "type": "input_audio_buffer.append",
                            "audio": base64::engine::general_purpose::STANDARD.encode(pcm),
                        });
                        if let Err(e) = writer.send(Message::Text(packet.to_string())).await {
                            log::warn!("OpenAI Realtime audio stream disconnected: {e}");
                            break;
                        }
                    }
                }
                event = reader.next() => {
                    let Some(event) = event else { break; };
                    match event {
                        Ok(Message::Text(text)) => emit_realtime_transcript(&app, &source, &segment_id, &mut partial, &text),
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            log::warn!("OpenAI Realtime receive failed: {e}");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        let _ = writer.send(Message::Close(None)).await;
    });
}

fn emit_realtime_transcript(
    app: &AppHandle,
    source: &str,
    segment_id: &str,
    partial: &mut String,
    event_text: &str,
) {
    let Ok(event) = serde_json::from_str::<Value>(event_text) else {
        return;
    };
    let event_type = event["type"].as_str().unwrap_or_default();
    let (text, is_final) = match event_type {
        "conversation.item.input_audio_transcription.delta" => {
            let delta = event["delta"].as_str().unwrap_or_default();
            partial.push_str(delta);
            (partial.clone(), false)
        }
        "conversation.item.input_audio_transcription.completed" => {
            let completed = event["transcript"].as_str().unwrap_or_default();
            if !completed.is_empty() {
                partial.clear();
                partial.push_str(completed);
            }
            (partial.clone(), true)
        }
        "error" => {
            log::warn!("OpenAI Realtime error: {}", event["error"]);
            return;
        }
        _ => return,
    };
    if text.trim().is_empty() {
        return;
    }
    let segment = TranscriptSegment {
        id: if is_final { Uuid::new_v4().to_string() } else { segment_id.to_string() },
        text,
        source: source.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        is_final,
    };
    if let Err(e) = app.emit("transcript-segment", segment) {
        log::error!("failed to emit Realtime transcript segment: {e}");
    }
}
