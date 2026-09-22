import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTheme } from "@/context/ThemeContext";

const appWindow = getCurrentWindow();

export default function TitleBar() {
  const { resolved, setMode } = useTheme();

  return (
    <div className="title-bar" data-tauri-drag-region>
      <div className="title-bar__brand" data-tauri-drag-region>
        <span className="title-bar__logo" aria-hidden>
          ✦
        </span>
        <span className="title-bar__title">AI Desktop Assistant</span>
      </div>

      <div className="title-bar__actions">
        <button
          className="title-bar__btn"
          title="Toggle theme"
          onClick={() => setMode("dark")}
        >
          {resolved === "dark" ? "☀" : "☾"}
        </button>
        <button
          className="title-bar__btn"
          title="Minimize"
          onClick={() => appWindow.minimize()}
        >
          –
        </button>
        <button
          className="title-bar__btn"
          title="Maximize"
          onClick={() => appWindow.toggleMaximize()}
        >
          ▢
        </button>
        <button
          className="title-bar__btn title-bar__btn--close"
          title="Close to tray"
          onClick={() => appWindow.hide()}
        >
          ✕
        </button>
      </div>
    </div>
  );
}
