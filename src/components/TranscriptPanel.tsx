import type { TranscriptController } from "@/hooks/useTranscript";

interface TranscriptPanelProps {
  transcript: TranscriptController;
}

export default function TranscriptPanel({ transcript }: TranscriptPanelProps) {
  const latestSegment = transcript.segments[transcript.segments.length - 1];

  return (
    <section className="glass panel">
      <div className="panel__body panel__body--listener">
        <button className="chip chip--ghost panel__clear panel__clear-action" onClick={transcript.clear} title="Clear transcript">
          Clear
        </button>
        {!latestSegment ? (
          <p className="panel__hint">
            {transcript.micActive
              ? "Listening to your microphone…"
              : "Microphone is paused. Use the microphone icon to start listening."}
          </p>
        ) : (
          <p className={"listener-ticker" + (latestSegment.isFinal ? "" : " is-interim")}>
            {latestSegment.text}
          </p>
        )}
        {transcript.error && <p className="listener-ticker listener-ticker--error">⚠ {transcript.error}</p>}
      </div>
    </section>
  );
}
