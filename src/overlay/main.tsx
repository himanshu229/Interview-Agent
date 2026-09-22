import { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { captureRegion } from "@/lib/api";
import type { CaptureRegion } from "@/types";

const appWindow = getCurrentWindow();

interface Point {
  x: number;
  y: number;
}

function RegionOverlay() {
  const [start, setStart] = useState<Point | null>(null);
  const [current, setCurrent] = useState<Point | null>(null);
  const capturing = useRef(false);

  const cancel = useCallback(async () => {
    await appWindow.close();
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") void cancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cancel]);

  const onMouseDown = (e: React.MouseEvent) => {
    setStart({ x: e.clientX, y: e.clientY });
    setCurrent({ x: e.clientX, y: e.clientY });
  };

  const onMouseMove = (e: React.MouseEvent) => {
    if (start) setCurrent({ x: e.clientX, y: e.clientY });
  };

  const onMouseUp = async () => {
    if (!start || !current || capturing.current) return;
    capturing.current = true;

    const region: CaptureRegion = {
      x: Math.round(Math.min(start.x, current.x)),
      y: Math.round(Math.min(start.y, current.y)),
      width: Math.round(Math.abs(current.x - start.x)),
      height: Math.round(Math.abs(current.y - start.y)),
    };

    if (region.width < 4 || region.height < 4) {
      await cancel();
      return;
    }

    // Hide before capturing so the overlay isn't part of the screenshot.
    await appWindow.hide();
    try {
      const result = await captureRegion(region);
      await emit("region-captured", result);
    } catch (err) {
      await emit("region-error", String(err));
    } finally {
      await appWindow.close();
    }
  };

  const box =
    start && current
      ? {
          left: Math.min(start.x, current.x),
          top: Math.min(start.y, current.y),
          width: Math.abs(current.x - start.x),
          height: Math.abs(current.y - start.y),
        }
      : null;

  return (
    <div
      onMouseDown={onMouseDown}
      onMouseMove={onMouseMove}
      onMouseUp={onMouseUp}
      style={{ width: "100vw", height: "100vh", position: "relative" }}
    >
      {box && (
        <div
          style={{
            position: "absolute",
            left: box.left,
            top: box.top,
            width: box.width,
            height: box.height,
            border: "2px solid #4c8bf5",
            background: "rgba(76, 139, 245, 0.15)",
            boxShadow: "0 0 0 9999px rgba(0,0,0,0.25)",
          }}
        />
      )}
      <div
        style={{
          position: "absolute",
          top: 16,
          left: "50%",
          transform: "translateX(-50%)",
          color: "#fff",
          font: "14px system-ui, sans-serif",
          background: "rgba(0,0,0,0.6)",
          padding: "6px 12px",
          borderRadius: 8,
          pointerEvents: "none",
        }}
      >
        Drag to select an area · Esc to cancel
      </div>
    </div>
  );
}

createRoot(document.getElementById("overlay-root") as HTMLElement).render(
  <RegionOverlay />,
);
