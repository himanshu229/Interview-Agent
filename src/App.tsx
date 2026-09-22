import { useEffect, useState } from "react";
import SessionSetup from "@/components/SessionSetup";
import Assistant from "@/components/Assistant";
import { useSettings } from "@/context/SettingsContext";
import { buildSystemPrompt } from "@/lib/session";
import type { SessionConfig } from "@/types";
import "@/styles/overlay.css";

export default function App() {
  const { settings, loading, update } = useSettings();
  const [session, setSession] = useState<SessionConfig | null>(null);

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

  if (loading) {
    return null;
  }

  if (!session) {
    return <SessionSetup onCreate={onCreate} />;
  }

  return <Assistant session={session} onEnd={() => setSession(null)} />;
}
