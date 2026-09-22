import type { SessionConfig } from "@/types";

/** Shared list of selectable chat models (session setup + in-session menu). */
export const MODEL_OPTIONS = [
  { value: "gpt-4o-mini", label: "GPT-4o Mini" },
  { value: "gpt-4o", label: "GPT-4o" },
  { value: "gpt-4.1-mini", label: "GPT-4.1 Mini" },
  { value: "gpt-4.1", label: "GPT-4.1" },
];

/** Builds the assistant system prompt from the session configuration. */
export function buildSystemPrompt(cfg: SessionConfig): string {
  const parts = [
    "You are an AI assistant helping the user during a live interview or call.",
    `Target company: ${cfg.company}.`,
    `Job description:\n${cfg.jobDescription}`,
  ];
  if (cfg.resumeText.trim()) {
    parts.push(`Candidate resume:\n${cfg.resumeText}`);
  }
  if (cfg.instructions.trim()) {
    parts.push(`Additional instructions:\n${cfg.instructions}`);
  }
  parts.push(`Always respond in ${cfg.language}.`);
  if (cfg.easyLanguage) {
    parts.push("Use simple, clear, easy-to-understand language.");
  }
  parts.push("Answer concisely and helpfully.");
  return parts.join("\n\n");
}
