import { useState, type ReactNode } from "react";
import { SNAP_POSITIONS, snapWindowTo } from "@/lib/windowSnap";

export interface ToolbarProps {
  micActive: boolean;
  systemActive: boolean;
  onToggleMic: () => void;
  onToggleSystem: () => void;
  onChat: () => void;
  onNew: () => void;
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
  onNew,
  onEnd,
  elapsed,
  menu,
}: ToolbarProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [positionOpen, setPositionOpen] = useState(false);

  return (
    <div className="toolbar">
      {/* Drag handle */}
      <span className="toolbar__grip" data-tauri-drag-region title="Drag to move">
        ⠿
      </span>

      <button
        className={"toolbar__btn toolbar__listen" + (micActive ? " is-active" : "")}
        onClick={onToggleMic}
        title="Start / stop microphone (⌘⇧L)"
      >
        <span className={"dot" + (micActive ? " dot--live" : "")} />
        Mic
      </button>

      <button
        className={"toolbar__btn toolbar__listen" + (systemActive ? " is-active" : "")}
        onClick={onToggleSystem}
        title="Start / stop system audio"
      >
        <span className={"dot" + (systemActive ? " dot--live" : "")} />
        System
      </button>

      <button className="toolbar__btn" onClick={onChat} title="Ask a question">
        Chat <Kbd>⌘/</Kbd>
      </button>

      <button className="toolbar__icon" onClick={onNew} title="New session (⌘N)">
        ＋
      </button>

      {/* Draggable spacer */}
      <span className="toolbar__spacer" data-tauri-drag-region />

      <button className="toolbar__timer" onClick={onEnd} title="Click to end session">
        {elapsed}
      </button>

      <div className="toolbar__menu-wrap">
        <button
          className={"toolbar__icon" + (positionOpen ? " is-active" : "")}
          onClick={() => setPositionOpen((v) => !v)}
          title="Snap window position"
        >
          ⊞
        </button>
        {positionOpen && (
          <>
            <div className="menu-backdrop" onClick={() => setPositionOpen(false)} />
            <div className="position-dropdown">
              <div className="position-grid">
                {SNAP_POSITIONS.map((pos) => (
                  <button
                    key={pos}
                    className="position-cell"
                    title={pos.replace("-", " ")}
                    onClick={() => {
                      void snapWindowTo(pos);
                      setPositionOpen(false);
                    }}
                  >
                    <span className={"position-cell__mark position-cell__mark--" + pos} />
                  </button>
                ))}
              </div>
            </div>
          </>
        )}
      </div>

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
