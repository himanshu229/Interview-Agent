/**
 * Thin, typed wrappers around Tauri IPC commands defined in the Rust backend.
 * All backend interaction flows through this module so components never call
 * `invoke` directly.
 */
import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  CaptureRegion,
  ChatRequest,
  ChatResponse,
  ChatMessage,
  Conversation,
  OcrResult,
  ScreenshotResult,
} from "@/types";

// ---------------------------------------------------------------------------
// Chat
// ---------------------------------------------------------------------------

export async function sendChat(request: ChatRequest): Promise<ChatResponse> {
  return invoke<ChatResponse>("send_chat", { request });
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

export async function listConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>("list_conversations");
}

export async function getMessages(
  conversationId: number,
): Promise<ChatMessage[]> {
  return invoke<ChatMessage[]>("get_messages", { conversationId });
}

export async function createConversation(title: string): Promise<Conversation> {
  return invoke<Conversation>("create_conversation", { title });
}

export async function deleteConversation(conversationId: number): Promise<void> {
  return invoke("delete_conversation", { conversationId });
}

export async function renameConversation(
  conversationId: number,
  title: string,
): Promise<void> {
  return invoke("rename_conversation", { conversationId, title });
}

// ---------------------------------------------------------------------------
// Screenshot + OCR
// ---------------------------------------------------------------------------

export async function captureFullScreen(): Promise<ScreenshotResult> {
  return invoke<ScreenshotResult>("capture_full_screen");
}

export async function captureRegion(
  region: CaptureRegion,
): Promise<ScreenshotResult> {
  return invoke<ScreenshotResult>("capture_region", { region });
}

/** Opens the transparent overlay window for mouse-driven area selection. */
export async function startRegionSelection(): Promise<void> {
  return invoke("start_region_selection");
}

export async function runOcr(
  screenshotPath: string,
  language: string,
): Promise<OcrResult> {
  return invoke<OcrResult>("run_ocr", { screenshotPath, language });
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

/** Extracts plain text from a PDF file (used for resume parsing). */
export async function extractPdfText(path: string): Promise<string> {
  return invoke<string>("extract_pdf_text", { path });
}

// ---------------------------------------------------------------------------
// Audio / Transcription
// ---------------------------------------------------------------------------

export async function startMicTranscription(): Promise<void> {
  return invoke("start_mic_transcription");
}

export async function stopMicTranscription(): Promise<void> {
  return invoke("stop_mic_transcription");
}

export async function startSystemAudioTranscription(): Promise<void> {
  return invoke("start_system_audio_transcription");
}

export async function stopSystemAudioTranscription(): Promise<void> {
  return invoke("stop_system_audio_transcription");
}

// ---------------------------------------------------------------------------
// Settings + secure key storage
// ---------------------------------------------------------------------------

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

/** Stores the OpenAI API key in the OS keychain. Never persisted in plaintext. */
export async function setApiKey(apiKey: string): Promise<void> {
  return invoke("set_api_key", { apiKey });
}

export async function clearApiKey(): Promise<void> {
  return invoke("clear_api_key");
}

export async function testApiKey(): Promise<boolean> {
  return invoke<boolean>("test_api_key");
}

// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------

/** Hides/shows the window from screen capture, recording, and screen sharing. */
export async function setContentProtection(enabled: boolean): Promise<void> {
  return invoke("set_content_protection", { enabled });
}
