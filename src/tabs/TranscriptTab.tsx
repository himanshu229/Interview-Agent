import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { TranscriptSegment } from "@/types";
import {
  startMicTranscription,
  startSystemAudioTranscription,
  stopMicTranscription,
  stopSystemAudioTranscription,
} from "@/lib/api";

type Capture = "microphone" | "system-audio";

export default function TranscriptTab() {
  const [segments, setSegments] = useState<TranscriptSegment[]>([]);
  const [micActive, setMicActive] = useState(false);
  const [systemActive, setSystemActive] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = listen<TranscriptSegment>("transcript-segment", (event) => {
      setSegments((prev) => {
        // Replace a non-final segment from the same source, else append.
        const idx = prev.findIndex(
          (s) => !s.isFinal && s.source === event.payload.source,
        );
        if (idx >= 0 && !event.payload.isFinal) {
          const copy = [...prev];
          copy[idx] = event.payload;
          return copy;
        }
        if (idx >= 0 && event.payload.isFinal) {
          const copy = [...prev];
          copy[idx] = event.payload;
          return copy;
        }
        return [...prev, event.payload];
      });
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [segments]);

  const toggle = useCallback(
    async (capture: Capture) => {
      setError(null);
      try {
        if (capture === "microphone") {
          if (micActive) {
            await stopMicTranscription();
            setMicActive(false);
          } else {
            await startMicTranscription();
            setMicActive(true);
          }
        } else {
          if (systemActive) {
            await stopSystemAudioTranscription();
            setSystemActive(false);
          } else {
            await startSystemAudioTranscription();
            setSystemActive(true);
          }
        }
      } catch (err) {
        setError(String(err));
      }
    },
    [micActive, systemActive],
  );

  return (
    <div className="transcript-tab">
      <div className="transcript-tab__controls">
        <button
          className={"btn " + (micActive ? "btn--danger" : "btn--primary")}
          onClick={() => toggle("microphone")}
        >
          {micActive ? "■ Stop mic" : "● Record mic"}
        </button>
        <button
          className={"btn " + (systemActive ? "btn--danger" : "btn--primary")}
          onClick={() => toggle("system-audio")}
        >
          {systemActive ? "■ Stop system audio" : "● Capture system audio"}
        </button>
        <button
          className="btn btn--ghost"
          onClick={() => setSegments([])}
          disabled={segments.length === 0}
        >
          Clear
        </button>
      </div>

      {error && <div className="transcript-tab__error">⚠ {error}</div>}

      <div className="transcript-tab__panel">
        {segments.length === 0 ? (
          <div className="empty-state">
            <h2>Live transcript</h2>
            <p>Start the microphone or capture system audio to see transcription here.</p>
          </div>
        ) : (
          segments.map((s) => (
            <div
              key={s.id}
              className={
                "transcript-line" + (s.isFinal ? "" : " transcript-line--interim")
              }
            >
              <span className={"transcript-line__badge badge--" + s.source}>
                {s.source === "microphone" ? "MIC" : "SYS"}
              </span>
              <span className="transcript-line__text">{s.text}</span>
            </div>
          ))
        )}
        <div ref={endRef} />
      </div>
    </div>
  );
}
