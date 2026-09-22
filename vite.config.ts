import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";
import { fileURLToPath } from "url";

const rootDir = fileURLToPath(new URL(".", import.meta.url));

// @tauri-apps/cli sets this env var when running `tauri dev`/`tauri build`.
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],
  envDir: resolve(rootDir, "src-tauri"),
  resolve: {
    alias: {
      "@": resolve(rootDir, "src"),
    },
  },

  // Prevent Vite from obscuring Rust errors.
  clearScreen: false,

  // Tauri expects a fixed port, fail if that port is not available.
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tell Vite to ignore watching `src-tauri`.
      ignored: ["**/src-tauri/**"],
    },
  },

  // Produce output tuned to the Tauri app + WebView target.
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux.
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    // Don't minify for debug builds.
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    // Produce sourcemaps for debug builds.
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // Multiple HTML entry points: main window + region-selection overlay.
    rollupOptions: {
      input: {
        main: resolve(rootDir, "index.html"),
        overlay: resolve(rootDir, "overlay.html"),
      },
    },
  },

  // Env variables starting with the item of `envPrefix` will be exposed
  // to the client source code via `import.meta.env`.
  envPrefix: ["VITE_", "TAURI_ENV_"],
}));
