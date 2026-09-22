import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import Toolbar from "@/components/Toolbar";
import TranscriptPanel from "@/components/TranscriptPanel";
import AnswerPanel from "@/components/AnswerPanel";
import Menu from "@/components/Menu";
import SettingsModal from "@/components/SettingsModal";
import { useTranscript } from "@/hooks/useTranscript";
import { useAssistant } from "@/hooks/useAssistant";
import { useSettings } from "@/context/SettingsContext";
import { captureFullScreen, runOcr } from "@/lib/api";
import { buildSystemPrompt } from "@/lib/session";
import type { SessionConfig } from "@/types";

const appWindow = getCurrentWindow();

function useElapsed(): string {
  const [seconds, setSeconds] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setSeconds((s) => s + 1), 1000);
    return () => clearInterval(id);
  }, []);
  const mm = Math.floor(seconds / 60);
  const ss = seconds % 60;
  return `${mm}:${ss.toString().padStart(2, "0")}`;
}

interface AssistantProps {
  session: SessionConfig;
  onEnd: () => void;
}

export default function Assistant({ session, onEnd }: AssistantProps) {
  const { settings, update } = useSettings();
  const transcript = useTranscript();
  const assistant = useAssistant();
  const elapsed = useElapsed();

  const [composeOpen, setComposeOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);

  // Model and easy-language can be changed mid-session from the ⋮ menu.
  const [model, setModel] = useState(session.model);
  const [easyLanguage, setEasyLanguage] = useState(session.easyLanguage);

  const applySessionPatch = useCallback(
    async (patch: Partial<Pick<SessionConfig, "model" | "easyLanguage">>) => {
      const nextModel = patch.model ?? model;
      const nextEasyLanguage = patch.easyLanguage ?? easyLanguage;
      setModel(nextModel);
      setEasyLanguage(nextEasyLanguage);
      const nextConfig: SessionConfig = {
        ...session,
        model: nextModel,
        easyLanguage: nextEasyLanguage,
      };
      await update({ model: nextModel, systemPrompt: buildSystemPrompt(nextConfig) });
    },
    [model, easyLanguage, session, update],
  );

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--panel-alpha",
      String(settings.opacity),
    );
  }, [settings.opacity]);

  const onAnswer = useCallback(() => {
    const context = transcript.fullText || assistant.question;
    if (context) void assistant.ask(context, { format: true });
  }, [transcript.fullText, assistant]);

  const onScreenshot = useCallback(async () => {
    try {
      const shot = await captureFullScreen();
      const ocr = await runOcr(shot.path, settings.ocrLanguage);
      if (ocr.text.trim()) {
        void assistant.ask(ocr.text, { format: true, source: "ocr" });
      }
    } catch (err) {
      console.error("screenshot failed", err);
    }
  }, [assistant, settings.ocrLanguage]);

  const onNew = useCallback(() => {
    transcript.clear();
    assistant.clear();
  }, [transcript, assistant]);

  const onExpand = useCallback(() => {
    void appWindow.toggleMaximize();
  }, []);

  const onChat = useCallback(() => setComposeOpen((v) => !v), []);

  // Stopping the session timer ends the session and returns to the setup page.
  const endSession = useCallback(() => {
    if (transcript.micActive) void transcript.toggleMic();
    if (transcript.systemActive) void transcript.toggleSystem();
    onEnd();
  }, [transcript, onEnd]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      const key = e.key.toLowerCase();

      if (key === "enter") {
        e.preventDefault();
        onAnswer();
      } else if (e.shiftKey && key === "s") {
        e.preventDefault();
        void onScreenshot();
      } else if (key === "/") {
        e.preventDefault();
        onChat();
      } else if (e.shiftKey && key === "l") {
        e.preventDefault();
        void transcript.toggleMic();
      } else if (key === "n") {
        e.preventDefault();
        onNew();
      } else if (e.shiftKey && (key === "backspace" || key === "delete")) {
        e.preventDefault();
        onNew();
      } else if (e.shiftKey && key === "f") {
        e.preventDefault();
        onExpand();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onAnswer, onScreenshot, onChat, onNew, onExpand, transcript]);

  useEffect(() => {
    const unlisten = listen<string>("navigate-tab", (event) => {
      if (event.payload === "settings") setSettingsOpen(true);
      if (event.payload === "screenshot") void onScreenshot();
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [onScreenshot]);

  return (
    <div className="overlay">
      <Toolbar
        micActive={transcript.micActive}
        systemActive={transcript.systemActive}
        onToggleMic={transcript.toggleMic}
        onToggleSystem={transcript.toggleSystem}
        onChat={onChat}
        onNew={onNew}
        onExpand={onExpand}
        onEnd={endSession}
        elapsed={elapsed}
        menu={
          <Menu
            easyLanguage={easyLanguage}
            onToggleEasyLanguage={() => void applySessionPatch({ easyLanguage: !easyLanguage })}
            model={model}
            onChangeModel={(m) => void applySessionPatch({ model: m })}
          />
        }
      />

      <div className="panels">
        <TranscriptPanel transcript={transcript} />
        <AnswerPanel assistant={assistant} composeOpen={composeOpen} />
      </div>

      <SettingsModal open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}
