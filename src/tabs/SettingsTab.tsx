import { useSettings } from "@/context/SettingsContext";
import { setContentProtection } from "@/lib/api";

export default function SettingsTab() {
  const { settings, update } = useSettings();

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
        <div className="settings-shortcut-list" aria-label="Window movement shortcuts">
          <div>
            <span>Collapse / restore application</span>
            <kbd>Cmd/Ctrl + H</kbd>
          </div>
          <div>
            <span>Show / hide chat question</span>
            <kbd>Cmd/Ctrl + /</kbd>
          </div>
          <div>
            <span>Answer latest live question</span>
            <kbd>Cmd/Ctrl + Enter</kbd>
          </div>
          <div>
            <span>Clear transcript</span>
            <kbd>Cmd/Ctrl + Backspace/Delete</kbd>
          </div>
          <div>
            <span>Clear chat</span>
            <kbd>Cmd/Ctrl + Shift + Backspace/Delete</kbd>
          </div>
          <div>
            <span>Move application left</span>
            <kbd>Cmd/Ctrl + Left</kbd>
          </div>
          <div>
            <span>Move application right</span>
            <kbd>Cmd/Ctrl + Right</kbd>
          </div>
          <div>
            <span>Move application up</span>
            <kbd>Cmd/Ctrl + Up</kbd>
          </div>
          <div>
            <span>Move application down</span>
            <kbd>Cmd/Ctrl + Down</kbd>
          </div>
        </div>
      </section>
    </div>
  );
}
