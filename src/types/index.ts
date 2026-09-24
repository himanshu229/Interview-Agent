/**
 * Shared TypeScript types mirrored from the Rust backend (`src-tauri/src`).
 * Keep these in sync with the `serde` structs exposed by Tauri commands.
 */

export type Role = "system" | "user" | "assistant";

export type MessageSource = "chat" | "ocr" | "voice" | "system-audio";

export interface ChatMessage {
  id: number;
  conversationId: number;
  role: Role;
  content: string;
  source: MessageSource;
  createdAt: string; // ISO-8601
}

export interface Conversation {
  id: number;
  title: string;
  createdAt: string;
  updatedAt: string;
}

export interface ChatRequest {
  conversationId: number | null;
  prompt: string;
  source: MessageSource;
}

export interface ChatResponse {
  conversationId: number;
  message: ChatMessage;
}

export interface TranscriptSegment {
  id: string;
  text: string;
  source: "microphone" | "system-audio";
  createdAt: string;
  isFinal: boolean;
}

export interface ScreenshotResult {
  /** Absolute path to the saved PNG on disk. */
  path: string;
  /** Base64-encoded PNG (data URL body) for immediate preview. */
  base64: string;
  width: number;
  height: number;
}

export interface OcrResult {
  text: string;
  confidence: number;
  screenshotPath: string;
}

export interface CaptureRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type ThemeMode = "dark";

export interface AppSettings {
  model: string;
  whisperModel: string;
  baseUrl: string;
  theme: ThemeMode;
  launchAtStartup: boolean;
  globalShortcut: string;
  ocrLanguage: string;
  systemPrompt: string;
  autoSendOcr: boolean;
  contentProtection: boolean;
  opacity: number;
  hasApiKey: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  model: "gpt-4o-mini",
  whisperModel: "whisper-1",
  baseUrl: "https://api.openai.com/v1",
  theme: "dark",
  launchAtStartup: false,
  globalShortcut: "Command+H",
  ocrLanguage: "eng",
  systemPrompt:
    "You are a helpful AI desktop assistant. Answer concisely and clearly.",
  autoSendOcr: true,
  contentProtection: true,
  opacity: 0.85,
  hasApiKey: false,
};

export type TabKey =
  | "chat"
  | "transcript"
  | "screenshot"
  | "history"
  | "settings";

/** Per-session configuration captured on the setup page. */
export interface SessionConfig {
  company: string;
  jobDescription: string;
  model: string;
  language: string;
  easyLanguage: boolean;
  resumeName: string | null;
  resumeText: string;
  instructions: string;
}
