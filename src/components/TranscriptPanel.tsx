import { useEffect, useRef } from "react";
import type { TranscriptController } from "@/hooks/useTranscript";

interface TranscriptPanelProps {
  transcript: TranscriptController;
}

export default function TranscriptPanel({ transcript }: TranscriptPanelProps) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [transcript.segments]);

  return (
    <section className="glass panel">
      <div className="panel__body panel__body--listener">
        <button className="chip chip--ghost panel__clear panel__clear-action" onClick={transcript.clear} title="Clear transcript">
          Clear
        </button>
        {transcript.segments.length === 0 ? (
          <p className="panel__hint">
            {transcript.micActive
              ? "Listening to your microphone…"
              : "Microphone is paused. Use the microphone icon to start listening."}
          </p>
        ) : (
          transcript.segments.map((s) => (
            <p
              key={s.id}
              className={"line" + (s.isFinal ? "" : " line--interim")}
            >
              <span className="line__who">You:</span> {s.text}
            </p>
          ))
        )}
        {transcript.error && <p className="line line--error">⚠ {transcript.error}</p>}
        <div ref={endRef} />
      </div>
    </section>
  );
}
