import { useEffect, useState } from "react";
import { platform } from "@tauri-apps/plugin-os";
import { useSettings } from "@/context/SettingsContext";
import { setContentProtection } from "@/lib/api";

export default function SettingsTab() {
  const { settings, update } = useSettings();
  const [platformName] = useState(() => platform());
  const modifier = platformName === "macos" ? "Command" : "Control";

  // Local draft so partial keystrokes don't register invalid/empty shortcuts.
  const [shortcutDraft, setShortcutDraft] = useState(settings.globalShortcut);
  useEffect(() => setShortcutDraft(settings.globalShortcut), [settings.globalShortcut]);

  const commitShortcut = () => {
    const value = shortcutDraft.trim();
    if (!value) {
      setShortcutDraft(settings.globalShortcut);
      return;
    }
    if (value !== settings.globalShortcut) void update({ globalShortcut: value });
  };

  return (
    <div className="settings-tab">
      <section className="settings-section">
        <div className="settings-control">
          <div className="settings-control__head">
            <span>Transparency</span>
            <strong>{Math.round(settings.opacity * 100)}%</strong>
          </div>
          <input
            className="settings-control__range"
            type="range"
            min={0.2}
            max={1}
            step={0.05}
            value={settings.opacity}
            onChange={(e) => update({ opacity: Number(e.target.value) })}
            aria-label="Transparency"
          />
        </div>
        <label className="settings-toggle settings-toggle--switch">
          <span>
            Keep the assistant invisible to Teams, Zoom, Google Meet, and other
            screen recordings while it remains visible to you.
          </span>
          <input
            type="checkbox"
            checked={settings.contentProtection}
            onChange={async (e) => {
              await update({ contentProtection: e.target.checked });
              await setContentProtection(e.target.checked);
            }}
          />
        </label>
      </section>
      <section className="settings-section">
        <h3>Shortcuts</h3>
        <div className="settings-field settings-field--shortcut">
          <label>Hide / show application ({modifier} + Shift + K)</label>
          <input
            value={shortcutDraft}
            onChange={(e) => setShortcutDraft(e.target.value)}
            onBlur={commitShortcut}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.currentTarget.blur();
              }
            }}
            placeholder={`${modifier}+Shift+K`}
          />
        </div>
      </section>
    </div>
  );
}
