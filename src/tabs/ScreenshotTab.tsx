import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import type { OcrResult, ScreenshotResult } from "@/types";
import {
  captureFullScreen,
  runOcr,
  startRegionSelection,
} from "@/lib/api";
import { useSettings } from "@/context/SettingsContext";

export default function ScreenshotTab() {
  const { settings } = useSettings();
  const [shot, setShot] = useState<ScreenshotResult | null>(null);
  const [ocr, setOcr] = useState<OcrResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Receive the result of a region selection made in the overlay window.
  useEffect(() => {
    const capturedP = listen<ScreenshotResult>("region-captured", (event) => {
      setShot(event.payload);
      setOcr(null);
      setBusy(false);
    });
    const errorP = listen<string>("region-error", (event) => {
      setError(event.payload);
      setBusy(false);
    });
    return () => {
      void capturedP.then((fn) => fn());
      void errorP.then((fn) => fn());
    };
  }, []);

  const capture = async (mode: "full" | "region") => {
    setError(null);
    setOcr(null);
    setBusy(true);
    try {
      if (mode === "region") {
        // Overlay window handles selection then emits `region-captured`.
        await startRegionSelection();
        setBusy(false);
        return;
      }
      const result = await captureFullScreen();
      setShot(result);
    } catch (err) {
      setError(String(err));
    } finally {
      if (mode === "full") setBusy(false);
    }
  };

  const extractText = async (autoSend: boolean) => {
    if (!shot) return;
    setBusy(true);
    setError(null);
    try {
      const result = await runOcr(shot.path, settings.ocrLanguage);
      setOcr(result);
      if (autoSend && result.text.trim()) {
        await emit("chat-message", {
          prompt: `Please analyse this text captured from my screen:\n\n${result.text}`,
          source: "ocr",
        });
        await emit("ocr-sent", {});
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="screenshot-tab">
      <div className="screenshot-tab__controls">
        <button className="btn btn--primary" onClick={() => capture("full")} disabled={busy}>
          📷 Capture full screen
        </button>
        <button className="btn btn--primary" onClick={() => capture("region")} disabled={busy}>
          ▭ Select area
        </button>
        <button
          className="btn btn--secondary"
          onClick={() => extractText(false)}
          disabled={!shot || busy}
        >
          🔤 Extract text (OCR)
        </button>
        <button
          className="btn btn--secondary"
          onClick={() => extractText(true)}
          disabled={!shot || busy}
        >
          ⚡ OCR → Ask ChatGPT
        </button>
      </div>

      {error && <div className="screenshot-tab__error">⚠ {error}</div>}

      <div className="screenshot-tab__grid">
        <div className="screenshot-tab__preview">
          {shot ? (
            <img
              src={`data:image/png;base64,${shot.base64}`}
              alt="Screenshot preview"
            />
          ) : (
            <div className="empty-state">
              <h2>No screenshot yet</h2>
              <p>Capture the full screen or select an area to begin.</p>
            </div>
          )}
        </div>

        <div className="screenshot-tab__ocr">
          <h3>Extracted text</h3>
          {ocr ? (
            <>
              <div className="screenshot-tab__confidence">
                Confidence: {ocr.confidence.toFixed(1)}%
              </div>
              <textarea readOnly value={ocr.text} rows={16} />
            </>
          ) : (
            <p className="muted">Run OCR to see extracted text.</p>
          )}
        </div>
      </div>
    </div>
  );
}
