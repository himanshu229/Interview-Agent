import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/App";
import { ThemeProvider } from "@/context/ThemeContext";
import { SettingsProvider } from "@/context/SettingsContext";
import "@/styles/global.css";

try {
  const savedOpacity = Number(
    window.localStorage.getItem("ai-desktop-assistant.opacity"),
  );
  if (Number.isFinite(savedOpacity) && savedOpacity >= 0.2 && savedOpacity <= 1) {
    document.documentElement.style.setProperty(
      "--panel-alpha",
      String(savedOpacity),
    );
  }
} catch {
  // Local storage is optional; the default CSS opacity remains in effect.
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SettingsProvider>
      <ThemeProvider>
        <App />
      </ThemeProvider>
    </SettingsProvider>
  </React.StrictMode>,
);
