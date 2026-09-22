import { useState, type ReactNode } from "react";
import { currentMonitor, getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";

export interface ToolbarProps {
  micActive: boolean;
  systemActive: boolean;
  onToggleMic: () => void;
  onToggleSystem: () => void;
  onChat: () => void;
  onNew: () => void;
  onExpand: () => void;
  onEnd: () => void;
  elapsed: string;
  menu: ReactNode;
}

function Kbd({ children }: { children: ReactNode }) {
  return <span className="kbd">{children}</span>;
}

type SnapPosition =
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-center"
  | "bottom-right";

const SNAP_POSITIONS: SnapPosition[] = [
  "top-left",
  "top-center",
  "top-right",
  "bottom-left",
  "bottom-center",
  "bottom-right",
];

/** Moves the window to one of six screen anchor points (corners + edge centers). */
async function snapWindowTo(pos: SnapPosition) {
  const win = getCurrentWindow();
  const monitor = await currentMonitor();
  if (!monitor) return;

  const size = await win.outerSize();
  const margin = 16;
  const { position: monPos, size: monSize } = monitor;

  const minX = monPos.x + margin;
  const maxX = monPos.x + monSize.width - size.width - margin;
  const centerX = monPos.x + Math.round((monSize.width - size.width) / 2);
  const minY = monPos.y + margin;
  const maxY = monPos.y + monSize.height - size.height - margin;

  const coords: Record<SnapPosition, [number, number]> = {
    "top-left": [minX, minY],
    "top-center": [centerX, minY],
    "top-right": [maxX, minY],
    "bottom-left": [minX, maxY],
    "bottom-center": [centerX, maxY],
    "bottom-right": [maxX, maxY],
  };
  const [x, y] = coords[pos];
  await win.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));
}

export default function Toolbar({
  micActive,
  systemActive,
  onToggleMic,
  onToggleSystem,
  onChat,
  onNew,
  onExpand,
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

      <button className="toolbar__icon" onClick={onExpand} title="Resize window (⌘⇧F)">
        ⤢
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
