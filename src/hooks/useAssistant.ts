import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { MessageSource } from "@/types";
import { sendChat } from "@/lib/api";

export interface AssistantController {
  question: string;
  answer: string;
  loading: boolean;
  error: string | null;
  ask: (question: string, opts?: { format?: boolean; source?: MessageSource }) => Promise<void>;
  clear: () => void;
}

const ANSWER_INSTRUCTION =
  "Answer the following question concisely as 3 short, clear bullet points. " +
  "Do not add a preamble.";

export function useAssistant(): AssistantController {
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const conversationId = useRef<number | null>(null);

  const ask = useCallback<AssistantController["ask"]>(
    async (q, opts) => {
      const text = q.trim();
      if (!text) return;

      const format = opts?.format ?? true;
      const source = opts?.source ?? "chat";

      setQuestion(text);
      setAnswer("");
      setError(null);
      setLoading(true);

      const prompt = format ? `${ANSWER_INSTRUCTION}\n\n${text}` : text;

      try {
        const res = await sendChat({
          conversationId: conversationId.current,
          prompt,
          source,
        });
        conversationId.current = res.conversationId;
        setAnswer(res.message.content);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  // OCR auto-send and other backend-initiated prompts.
  useEffect(() => {
    const unlisten = listen<{ prompt: string; source: MessageSource }>(
      "chat-message",
      (event) => {
        void ask(event.payload.prompt, { format: false, source: event.payload.source });
      },
    );
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [ask]);

  const clear = useCallback(() => {
    setQuestion("");
    setAnswer("");
    setError(null);
    conversationId.current = null;
  }, []);

  return { question, answer, loading, error, ask, clear };
}
