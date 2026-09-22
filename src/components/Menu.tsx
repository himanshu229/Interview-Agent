import { useSettings } from "@/context/SettingsContext";
import { MODEL_OPTIONS } from "@/lib/session";

interface MenuProps {
  easyLanguage: boolean;
  onToggleEasyLanguage: () => void;
  model: string;
  onChangeModel: (model: string) => void;
}

export default function Menu({
  easyLanguage,
  onToggleEasyLanguage,
  model,
  onChangeModel,
}: MenuProps) {
  const { settings, update } = useSettings();
  const languages = [
    ["eng", "English"],
    ["spa", "Spanish"],
    ["fra", "French"],
    ["deu", "German"],
    ["hin", "Hindi"],
    ["chi_sim", "Chinese"],
  ];

  return (
    <div className="menu">
      <div className="menu__section">
        <label className="menu__row">
          <span>Transparency</span>
          <span className="menu__value">{Math.round(settings.opacity * 100)}%</span>
        </label>
        <input
          type="range"
          min={0.2}
          max={1}
          step={0.05}
          value={settings.opacity}
          onChange={(e) => update({ opacity: Number(e.target.value) })}
          className="menu__slider"
          aria-label="Transparency"
        />
      </div>

      <div className="menu__divider" />

      <label className="menu__toggle">
        <span>Easy language</span>
        <input
          type="checkbox"
          checked={easyLanguage}
          onChange={onToggleEasyLanguage}
        />
      </label>

      <div className="menu__divider" />

      <div className="menu__section">
        <label className="menu__row">
          <span>AI model</span>
        </label>
        <select
          className="menu__select"
          value={model}
          onChange={(e) => onChangeModel(e.target.value)}
        >
          {MODEL_OPTIONS.map((m) => (
            <option key={m.value} value={m.value}>
              {m.label}
            </option>
          ))}
        </select>
      </div>

      <div className="menu__section">
        <label className="menu__row">
          <span>Language</span>
        </label>
        <select
          className="menu__select"
          value={settings.ocrLanguage}
          onChange={(e) => void update({ ocrLanguage: e.target.value })}
        >
          {languages.map(([code, label]) => (
            <option key={code} value={code}>
              {label}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
