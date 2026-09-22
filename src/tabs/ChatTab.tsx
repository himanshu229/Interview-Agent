import { useEffect, useRef } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { ChatController } from "@/hooks/useChat";
import type { ChatMessage } from "@/types";

interface ChatTabProps {
  chat: ChatController;
}

function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === "user";
  return (
    <div className={"bubble bubble--" + (isUser ? "user" : "assistant")}>
      <div className="bubble__meta">
        <span className="bubble__role">{isUser ? "You" : "Assistant"}</span>
        {message.source !== "chat" && (
          <span className="bubble__tag">{message.source}</span>
        )}
      </div>
      <div className="bubble__content">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>
          {message.content}
        </ReactMarkdown>
      </div>
    </div>
  );
}

export default function ChatTab({ chat }: ChatTabProps) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [chat.messages]);

  return (
    <div className="chat-tab">
      <div className="chat-tab__toolbar">
        <button className="btn btn--ghost" onClick={chat.newConversation}>
          + New chat
        </button>
      </div>

      <div className="chat-tab__messages">
        {chat.messages.length === 0 && (
          <div className="empty-state">
            <h2>Ask me anything</h2>
            <p>
              Type a question above, capture a screenshot for OCR, or start a
              live transcription.
            </p>
          </div>
        )}

        {chat.messages.map((m) => (
          <MessageBubble key={m.id} message={m} />
        ))}

        {chat.sending && (
          <div className="bubble bubble--assistant bubble--typing">
            <span className="typing-dot" />
            <span className="typing-dot" />
            <span className="typing-dot" />
          </div>
        )}

        {chat.error && <div className="chat-tab__error">⚠ {chat.error}</div>}
        <div ref={endRef} />
      </div>
    </div>
  );
}
