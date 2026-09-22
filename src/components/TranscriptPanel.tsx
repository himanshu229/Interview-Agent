import { useEffect, useRef } from "react";
import type { TranscriptController } from "@/hooks/useTranscript";
import { useSettings } from "@/context/SettingsContext";

interface TranscriptPanelProps {
  transcript: TranscriptController;
}

export function TranscriptControls({ transcript }: TranscriptPanelProps) {
  const { settings, update } = useSettings();

  return (
    <div className="listener-controls">
      <div className="panel__head-left">
        <select
          className="mini-select"
          value={settings.ocrLanguage}
          onChange={(e) => update({ ocrLanguage: e.target.value })}
        >
          {LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>
              🌐 {l.label}
            </option>
          ))}
        </select>
        <button
          className={"chip" + (transcript.micActive ? " chip--live" : "")}
          onClick={transcript.toggleMic}
        >
          {transcript.micActive ? "● Mic" : "Mic"}
        </button>
        <button
          className={"chip" + (transcript.systemActive ? " chip--live" : "")}
          onClick={transcript.toggleSystem}
        >
          {transcript.systemActive ? "● System" : "System"}
        </button>
      </div>
      <button className="chip chip--ghost" onClick={transcript.clear}>
        Clear <span className="kbd">⌘⇧⌫</span>
      </button>
    </div>
  );
}

const LANGUAGES = [
  { code: "eng", label: "English" },
  { code: "spa", label: "Spanish" },
  { code: "fra", label: "French" },
  { code: "deu", label: "German" },
  { code: "hin", label: "Hindi" },
  { code: "chi_sim", label: "Chinese" },
];

export default function TranscriptPanel({ transcript }: TranscriptPanelProps) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [transcript.segments]);

  return (
    <section className="glass panel">
      <div className="panel__body panel__body--listener">
        <button className="chip chip--ghost panel__clear" onClick={transcript.clear}>
          Clear <span className="kbd">⌘⇧⌫</span>
        </button>
        {transcript.segments.length === 0 ? (
          <p className="panel__hint">
            Press <b>Listen</b> to transcribe your microphone or system audio.
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
