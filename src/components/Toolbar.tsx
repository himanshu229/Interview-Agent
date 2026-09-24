import { useState, type ReactNode } from "react";
import { collapseCompactWindow } from "@/lib/api";

export interface ToolbarProps {
  micActive: boolean;
  systemActive: boolean;
  onToggleMic: () => void;
  onToggleSystem: () => void;
  onChat: () => void;
  onEnd: () => void;
  elapsed: string;
  menu: ReactNode;
}

function Kbd({ children }: { children: ReactNode }) {
  return <span className="kbd">{children}</span>;
}

export default function Toolbar({
  micActive,
  systemActive,
  onToggleMic,
  onToggleSystem,
  onChat,
  onEnd,
  elapsed,
  menu,
}: ToolbarProps) {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div className="toolbar">
      {/* Drag handle */}
      <span className="toolbar__grip" data-tauri-drag-region title="Drag to move">
        ⠿
      </span>

      <button
        className={"toolbar__icon toolbar__listen" + (micActive ? " is-active" : "")}
        onClick={onToggleMic}
        title={micActive ? "Stop microphone" : "Start microphone"}
      >
        🎙
      </button>

      <button
        className={"toolbar__icon toolbar__listen" + (systemActive ? " is-active" : "")}
        onClick={onToggleSystem}
        title={systemActive ? "Stop system audio" : "Start system audio"}
      >
        🔊
      </button>

      <button className="toolbar__btn" onClick={onChat} title="Ask a question">
        Chat <Kbd>⌘/</Kbd>
      </button>

      {/* Draggable spacer */}
      <span className="toolbar__spacer" data-tauri-drag-region />

      <button className="toolbar__timer" onClick={onEnd} title="Click to end session">
        {elapsed}
      </button>

      <button
        className="toolbar__icon"
        onClick={() => void collapseCompactWindow()}
        title="Collapse to app icon (Cmd/Ctrl+H)"
      >
        ⊖
      </button>

      <div className="toolbar__menu-wrap">
        <button
          className={"toolbar__icon" + (menuOpen ? " is-active" : "")}
          onClick={() => setMenuOpen((v) => !v)}
          title="Settings"
        >
          ⋮
        </button>
        {menuOpen && (
          <>
            <div className="menu-backdrop" onClick={() => setMenuOpen(false)} />
            <div className="menu-dropdown">{menu}</div>
          </>
        )}
      </div>
    </div>
  );
}
