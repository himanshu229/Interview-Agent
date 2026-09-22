import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ChatMessage, MessageSource } from "@/types";
import { getMessages, sendChat } from "@/lib/api";

export interface ChatController {
  messages: ChatMessage[];
  conversationId: number | null;
  sending: boolean;
  error: string | null;
  send: (prompt: string, source?: MessageSource) => Promise<void>;
  loadConversation: (id: number) => Promise<void>;
  newConversation: () => void;
}

/**
 * Central chat state manager. Also listens for `chat-message` events emitted by
 * the backend (e.g. when OCR text is auto-sent to ChatGPT from the tray).
 */
export function useChat(): ChatController {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [conversationId, setConversationId] = useState<number | null>(null);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const conversationRef = useRef<number | null>(null);

  useEffect(() => {
    conversationRef.current = conversationId;
  }, [conversationId]);

  const send = useCallback(
    async (prompt: string, source: MessageSource = "chat") => {
      const text = prompt.trim();
      if (!text) return;

      setSending(true);
      setError(null);

      // Optimistic user message.
      const optimistic: ChatMessage = {
        id: Date.now(),
        conversationId: conversationRef.current ?? -1,
        role: "user",
        content: text,
        source,
        createdAt: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, optimistic]);

      try {
        const res = await sendChat({
          conversationId: conversationRef.current,
          prompt: text,
          source,
        });
        setConversationId(res.conversationId);
        setMessages((prev) => [...prev, res.message]);
      } catch (err) {
        setError(String(err));
      } finally {
        setSending(false);
      }
    },
    [],
  );

  const loadConversation = useCallback(async (id: number) => {
    setError(null);
    try {
      const history = await getMessages(id);
      setConversationId(id);
      setMessages(history);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const newConversation = useCallback(() => {
    setConversationId(null);
    setMessages([]);
    setError(null);
  }, []);

  // Backend-initiated messages (OCR auto-send, voice commands).
  useEffect(() => {
    const unlisten = listen<{ prompt: string; source: MessageSource }>(
      "chat-message",
      (event) => {
        void send(event.payload.prompt, event.payload.source);
      },
    );
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [send]);

  return {
    messages,
    conversationId,
    sending,
    error,
    send,
    loadConversation,
    newConversation,
  };
}
