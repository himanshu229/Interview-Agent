import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import appIcon from "../assets/icon.svg";
import SessionSetup from "@/components/SessionSetup";
import Assistant from "@/components/Assistant";
import { useSettings } from "@/context/SettingsContext";
import { buildSystemPrompt } from "@/lib/session";
import { useAutoWindowHeight } from "@/lib/autosize";
import type { SessionConfig } from "@/types";
import "@/styles/overlay.css";

export default function App() {
  const { settings, loading, update } = useSettings();
  const [session, setSession] = useState<SessionConfig | null>(null);
  const [compact, setCompact] = useState(false);
  const containerRef = useAutoWindowHeight(!compact);

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--panel-alpha",
      String(settings.opacity),
    );
  }, [settings.opacity]);

  useEffect(() => {
    const unlisten = listen<boolean>("compact-window", (event) => {
      setCompact(event.payload);
    });
    return () => void unlisten.then((fn) => fn());
  }, []);

  const onCreate = async (config: SessionConfig) => {
    // Apply the session's model and composed system prompt for this session.
    await update({ model: config.model, systemPrompt: buildSystemPrompt(config) });
    setSession(config);
  };

  return (
    <>
      {compact && (
        <button
          className="compact-app-icon"
          data-tauri-drag-region
          onDoubleClick={() => void invoke("restore_compact_window")}
          title="Double-click to restore AI Desktop Assistant (Cmd/Ctrl+H)"
        >
          <img data-tauri-drag-region src={appIcon} alt="Double-click to restore AI Desktop Assistant" />
        </button>
      )}
      {/* Keep this mounted while compact so typed setup fields and session state persist. */}
      <div ref={containerRef} className={compact ? "app-content--compact" : undefined}>
        {loading ? null : session ? (
          <Assistant session={session} onEnd={() => setSession(null)} />
        ) : (
          <SessionSetup onCreate={onCreate} />
        )}
      </div>
    </>
  );
}
