import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { exit } from "@tauri-apps/plugin-process";
import { extractPdfText } from "@/lib/api";
import { MODEL_OPTIONS } from "@/lib/session";
import { useSettings } from "@/context/SettingsContext";
import SettingsModal from "@/components/SettingsModal";
import type { SessionConfig } from "@/types";

interface SessionSetupProps {
  onCreate: (config: SessionConfig) => void;
}

const LANGUAGES = [
  "English",
  "Spanish",
  "French",
  "German",
  "Hindi",
  "Chinese",
  "Japanese",
  "Portuguese",
  "Arabic",
];

const MAX_WORDS = 5000;

function wordCount(text: string): number {
  const t = text.trim();
  return t ? t.split(/\s+/).length : 0;
}

export default function SessionSetup({ onCreate }: SessionSetupProps) {
  const { settings } = useSettings();

  const [company, setCompany] = useState("");
  const [jobDescription, setJobDescription] = useState("");
  const [model, setModel] = useState(settings.model || "gpt-4o-mini");
  const [language, setLanguage] = useState("English");
  const [easyLanguage, setEasyLanguage] = useState(true);
  const [resumeName, setResumeName] = useState<string | null>(null);
  const [resumeText, setResumeText] = useState("");
  const [instructions, setInstructions] = useState("");
  const [instrOpen, setInstrOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canCreate = company.trim() !== "" && jobDescription.trim() !== "";
  const instrWords = useMemo(() => wordCount(instructions), [instructions]);

  const pickResume = async () => {
    setError(null);
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (typeof selected === "string") {
        setBusy(true);
        const text = await extractPdfText(selected);
        setResumeName(selected.split(/[/\\]/).pop() ?? "resume.pdf");
        setResumeText(text);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  const onInstructionsChange = (value: string) => {
    // Enforce the 5,000-word limit.
    if (wordCount(value) <= MAX_WORDS) {
      setInstructions(value);
    } else {
      setInstructions(value.trim().split(/\s+/).slice(0, MAX_WORDS).join(" "));
    }
  };

  const create = () => {
    if (!canCreate) return;
    onCreate({
      company: company.trim(),
      jobDescription: jobDescription.trim(),
      model,
      language,
      easyLanguage,
      resumeName,
      resumeText,
      instructions: instructions.trim(),
    });
  };

  return (
    <div className="setup">
      <div className="setup__card glass">
        <header className="setup__head" data-tauri-drag-region>
          <h1 data-tauri-drag-region>New Session</h1>
          <span className="setup__head-spacer" data-tauri-drag-region />
          <div className="setup__head-actions">
            <button
              className="toolbar__icon"
              onClick={() => setSettingsOpen(true)}
              title="Settings"
            >
              ⚙
            </button>
            <button
              className="toolbar__icon"
              onClick={() => void exit(0)}
              title="Close application"
            >
              ✕
            </button>
          </div>
        </header>

        <div className="setup__body">
          <label className="field">
            <span className="field__label">
              🏢 Company <b className="req">*</b>
            </span>
            <input
              value={company}
              onChange={(e) => setCompany(e.target.value)}
              placeholder="Microsoft"
              autoFocus
            />
          </label>

          <label className="field">
            <span className="field__label">
              📝 Job Description <b className="req">*</b>
            </span>
            <textarea
              rows={4}
              value={jobDescription}
              onChange={(e) => setJobDescription(e.target.value)}
              placeholder="Software Engineer versed in Python, SQL, and AWS…"
            />
          </label>

          <div className="setup__group">
            <span className="setup__group-title">Output Settings</span>
            <div className="setup__row">
              <label className="field field--inline">
                <span className="field__label">✨ Model</span>
                <select value={model} onChange={(e) => setModel(e.target.value)}>
                  {MODEL_OPTIONS.map((m) => (
                    <option key={m.value} value={m.value}>
                      {m.label}
                    </option>
                  ))}
                </select>
              </label>

              <label className="field field--inline">
                <span className="field__label">🌐 Language</span>
                <select
                  value={language}
                  onChange={(e) => setLanguage(e.target.value)}
                >
                  {LANGUAGES.map((l) => (
                    <option key={l} value={l}>
                      {l}
                    </option>
                  ))}
                </select>
              </label>

              <button
                type="button"
                className={"star-toggle" + (easyLanguage ? " is-on" : "")}
                onClick={() => setEasyLanguage((v) => !v)}
                title="Answer in simple, easy language"
              >
                {easyLanguage ? "★" : "☆"} Easy language
              </button>
            </div>
          </div>

          <div className="setup__group">
            <span className="setup__group-title">Context</span>
            <div className="setup__row">
              <button
                type="button"
                className="chip chip--outline"
                onClick={pickResume}
                disabled={busy}
              >
                📄 {resumeName ? resumeName : "Add Resume (PDF)"}
              </button>
              <button
                type="button"
                className="chip chip--outline"
                onClick={() => setInstrOpen(true)}
              >
                ＋ {instructions ? `Instructions (${instrWords} words)` : "Add Instructions"}
              </button>
            </div>
            {resumeText && (
              <p className="setup__hint">Resume loaded ✓ ({resumeText.length} chars)</p>
            )}
          </div>

          {error && <p className="setup__error">⚠ {error}</p>}
        </div>

        <footer className="setup__foot">
          <button className="btn-ghost" onClick={() => void exit(0)}>
            Close
          </button>
          <button
            className="btn-primary"
            disabled={!canCreate || busy}
            onClick={create}
          >
            Create Session →
          </button>
        </footer>
      </div>

      {instrOpen && (
        <div className="modal-backdrop" onClick={() => setInstrOpen(false)}>
          <div className="modal glass" onClick={(e) => e.stopPropagation()}>
            <header className="modal__head">
              <h2>Instructions</h2>
              <span
                className={
                  "instr-count" + (instrWords >= MAX_WORDS ? " instr-count--max" : "")
                }
              >
                {instrWords} / {MAX_WORDS} words
              </span>
              <button
                className="toolbar__icon"
                onClick={() => setInstrOpen(false)}
                title="Done"
              >
                ✕
              </button>
            </header>
            <div className="modal__body">
              <textarea
                className="instr-area"
                rows={16}
                value={instructions}
                onChange={(e) => onInstructionsChange(e.target.value)}
                placeholder="Add any instructions for the assistant (tone, focus areas, things to emphasise…). Up to 5,000 words."
              />
              <div className="modal__actions">
                <button className="btn-primary" onClick={() => setInstrOpen(false)}>
                  Done
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      <SettingsModal open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}
