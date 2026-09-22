import { currentMonitor, getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";

export type SnapPosition =
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-center"
  | "bottom-right";

export const SNAP_POSITIONS: SnapPosition[] = [
  "top-left",
  "top-center",
  "top-right",
  "bottom-left",
  "bottom-center",
  "bottom-right",
];

type Row = "top" | "bottom";
type Col = "left" | "center" | "right";

function splitPosition(pos: SnapPosition): [Row, Col] {
  const [row, col] = pos.split("-") as [Row, Col];
  return [row, col];
}

function joinPosition(row: Row, col: Col): SnapPosition {
  return `${row}-${col}` as SnapPosition;
}

/** Moves the window to one of six screen anchor points (corners + edge centers). */
export async function snapWindowTo(pos: SnapPosition) {
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

/** Finds the grid anchor closest to the window's current position. */
async function closestSnapPosition(): Promise<SnapPosition> {
  const win = getCurrentWindow();
  const monitor = await currentMonitor();
  const winPos = await win.outerPosition();
  if (!monitor) return "top-center";

  const { position: monPos, size: monSize } = monitor;
  const relX = (winPos.x - monPos.x) / monSize.width;
  const relY = (winPos.y - monPos.y) / monSize.height;

  const col: Col = relX < 1 / 3 ? "left" : relX < 2 / 3 ? "center" : "right";
  const row: Row = relY < 0.5 ? "top" : "bottom";
  return joinPosition(row, col);
}

const COLS: Col[] = ["left", "center", "right"];

/** Cycles the window through the 3x2 snap grid using an arrow-key direction. */
export async function snapWindowInDirection(direction: "up" | "down" | "left" | "right") {
  const current = await closestSnapPosition();
  const [row, col] = splitPosition(current);

  if (direction === "up" || direction === "down") {
    const nextRow: Row = row === "top" ? "bottom" : "top";
    await snapWindowTo(joinPosition(nextRow, col));
    return;
  }

  const colIndex = COLS.indexOf(col);
  const delta = direction === "right" ? 1 : -1;
  const nextCol = COLS[(colIndex + delta + COLS.length) % COLS.length];
  await snapWindowTo(joinPosition(row, nextCol));
}
