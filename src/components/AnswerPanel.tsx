import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { AssistantController } from "@/hooks/useAssistant";

interface AnswerPanelProps {
  assistant: AssistantController;
  composeOpen: boolean;
}

export default function AnswerPanel({ assistant, composeOpen }: AnswerPanelProps) {
  const [input, setInput] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (composeOpen) inputRef.current?.focus();
  }, [composeOpen]);

  useEffect(() => {
    bodyRef.current?.scrollTo({ top: bodyRef.current.scrollHeight, behavior: "smooth" });
  }, [assistant.answer, assistant.loading]);

  const submit = () => {
    const text = input.trim();
    if (!text) return;
    void assistant.ask(text, { format: true });
    setInput("");
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") submit();
  };

  return (
    <section className="glass panel answer-panel">
      {composeOpen && (
        <header className="panel__head">
          <div className="qbar">
            <span className="qbar__icon">💬</span>
            <input
              ref={inputRef}
              className="qbar__input"
              placeholder="Ask a question…"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={onKeyDown}
            />
          </div>
          <button className="chip chip--ghost panel__clear-action" onClick={assistant.clear} title="Clear chat">
            Clear
          </button>
        </header>
      )}

      <div className="panel__body answer-panel__body" ref={bodyRef}>
        {!assistant.question && !assistant.loading && (
          <p className="panel__hint">
            Ask a question, press <b>Answer</b> to solve the transcript, or capture
            a <b>Screenshot</b>.
          </p>
        )}

        {assistant.question && (
          <p className="qline">
            <span className="qline__tag">★ Question:</span> {assistant.question}
          </p>
        )}

        {assistant.loading ? (
          <div className="answer-loading">
            <span className="typing-dot" />
            <span className="typing-dot" />
            <span className="typing-dot" />
          </div>
        ) : (
          assistant.answer && (
            <div className="answer">
              <div className="answer__tag">Answer</div>
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {assistant.answer}
              </ReactMarkdown>
            </div>
          )
        )}

        {assistant.error && <p className="line line--error">⚠ {assistant.error}</p>}
      </div>
    </section>
  );
}
