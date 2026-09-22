import { useState } from "react";
import { platform } from "@tauri-apps/plugin-os";
import { useSettings } from "@/context/SettingsContext";
import { setContentProtection } from "@/lib/api";

export default function SettingsTab() {
  const { settings, update } = useSettings();
  const [platformName] = useState(() => platform());
  const modifier = platformName === "macos" ? "Command" : "Control";

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
          <label>Hide / show application ({modifier} + H)</label>
          <input
            value={settings.globalShortcut}
            onChange={(e) => update({ globalShortcut: e.target.value })}
            placeholder={`${modifier}+H`}
          />
        </div>
      </section>
    </div>
  );
}
