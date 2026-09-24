import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

// Must stay in sync with src-tauri/tauri.conf.json's window width/height bounds.
const WINDOW_WIDTH = 540;
const MIN_HEIGHT = 200;
const MAX_HEIGHT = 600;

/**
 * Resizes the (fixed-width) window's height to match its rendered content,
 * clamped to the min/max height configured in tauri.conf.json.
 *
 * Returns a ref callback (not a plain ref object) so the observer starts as
 * soon as the element actually mounts, even if that happens after the first
 * render (e.g. behind a loading gate).
 */
export function useAutoWindowHeight(enabled: boolean): (node: HTMLElement | null) => void {
  const [node, setNode] = useState<HTMLElement | null>(null);
  const ref = useCallback((el: HTMLElement | null) => setNode(el), []);

  useEffect(() => {
    if (!enabled || !node) return;

    let lastHeight = -1;

    const applySize = () => {
      const height = Math.round(node.getBoundingClientRect().height);
      const clamped = Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, height));
      if (clamped === lastHeight) return;
      lastHeight = clamped;
      // Routed through a Rust command (instead of window.setSize directly) so
      // every resize attempt is logged to the terminal for debugging.
      void invoke("resize_to_content", { width: WINDOW_WIDTH, height: clamped });
    };

    const observer = new ResizeObserver(applySize);
    observer.observe(node);
    applySize();

    return () => observer.disconnect();
  }, [enabled, node]);

  return ref;
}
