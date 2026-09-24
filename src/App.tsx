import { useEffect, useState } from "react";
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
  const containerRef = useAutoWindowHeight();

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--panel-alpha",
      String(settings.opacity),
    );
  }, [settings.opacity]);

  const onCreate = async (config: SessionConfig) => {
    // Apply the session's model and composed system prompt for this session.
    await update({ model: config.model, systemPrompt: buildSystemPrompt(config) });
    setSession(config);
  };

  // Always render the container (even while loading) so the ref attaches on
  // the very first render — otherwise the auto-resize observer never starts.
  return (
    <div ref={containerRef}>
      {loading ? null : session ? (
        <Assistant session={session} onEnd={() => setSession(null)} />
      ) : (
        <SessionSetup onCreate={onCreate} />
      )}
    </div>
  );
}
