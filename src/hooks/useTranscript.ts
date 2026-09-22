import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { TranscriptSegment } from "@/types";
import {
  startMicTranscription,
  startSystemAudioTranscription,
  stopMicTranscription,
  stopSystemAudioTranscription,
} from "@/lib/api";

export interface TranscriptController {
  segments: TranscriptSegment[];
  micActive: boolean;
  systemActive: boolean;
  error: string | null;
  fullText: string;
  toggleMic: () => Promise<void>;
  toggleSystem: () => Promise<void>;
  clear: () => void;
}

export function useTranscript(): TranscriptController {
  const [segments, setSegments] = useState<TranscriptSegment[]>([]);
  const [micActive, setMicActive] = useState(false);
  const [systemActive, setSystemActive] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<TranscriptSegment>("transcript-segment", (event) => {
      setSegments((prev) => {
        const idx = prev.findIndex(
          (s) => !s.isFinal && s.source === event.payload.source,
        );
        if (idx >= 0) {
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

  const toggleMic = useCallback(async () => {
    setError(null);
    try {
      if (micActive) {
        await stopMicTranscription();
        setMicActive(false);
      } else {
        await startMicTranscription();
        setMicActive(true);
      }
    } catch (err) {
      setError(String(err));
    }
  }, [micActive]);

  const toggleSystem = useCallback(async () => {
    setError(null);
    try {
      if (systemActive) {
        await stopSystemAudioTranscription();
        setSystemActive(false);
      } else {
        await startSystemAudioTranscription();
        setSystemActive(true);
      }
    } catch (err) {
      setError(String(err));
    }
  }, [systemActive]);

  const clear = useCallback(() => setSegments([]), []);

  const fullText = useMemo(
    () => segments.map((s) => s.text).join(" ").trim(),
    [segments],
  );

  return {
    segments,
    micActive,
    systemActive,
    error,
    fullText,
    toggleMic,
    toggleSystem,
    clear,
  };
}
