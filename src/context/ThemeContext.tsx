import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { ThemeMode } from "@/types";
import { useSettings } from "@/context/SettingsContext";

type ResolvedTheme = "light" | "dark";

interface ThemeContextValue {
  mode: ThemeMode;
  resolved: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { settings, update } = useSettings();
  const [resolved] = useState<ResolvedTheme>("dark");

  useEffect(() => {
    if (settings.theme !== "dark") {
      void update({ theme: "dark" });
    }
    document.documentElement.setAttribute("data-theme", "dark");
  }, [settings.theme, update]);

  const mode: ThemeMode = "dark";

  const value = useMemo<ThemeContextValue>(
    () => ({
      mode,
      resolved,
      setMode: () => void update({ theme: "dark" }),
    }),
    [resolved, update],
  );

  return (
    <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return ctx;
}
