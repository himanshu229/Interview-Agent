import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { AppSettings, DEFAULT_SETTINGS } from "@/types";
import { getSettings, saveSettings, setApiKey } from "@/lib/api";

interface SettingsContextValue {
  settings: AppSettings;
  loading: boolean;
  update: (patch: Partial<AppSettings>) => Promise<void>;
  reload: () => Promise<void>;
}

const SettingsContext = createContext<SettingsContextValue | undefined>(
  undefined,
);

const OPACITY_STORAGE_KEY = "ai-desktop-assistant.opacity";

function readSavedOpacity(): number | null {
  try {
    const value = Number(window.localStorage.getItem(OPACITY_STORAGE_KEY));
    return Number.isFinite(value) && value >= 0.2 && value <= 1 ? value : null;
  } catch {
    return null;
  }
}

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(() => {
    const opacity = readSavedOpacity();
    return opacity === null ? DEFAULT_SETTINGS : { ...DEFAULT_SETTINGS, opacity };
  });
  const [loading, setLoading] = useState(true);
  const settingsRef = useRef(settings);
  const saveQueue = useRef(Promise.resolve());

  const reload = useCallback(async () => {
    try {
      let loaded = await getSettings();
      const environmentApiKey = import.meta.env.VITE_OPENAI_API_KEY?.trim();
      if (!loaded.hasApiKey && environmentApiKey) {
        await setApiKey(environmentApiKey);
        loaded = await getSettings();
      }
      const savedOpacity = readSavedOpacity();
      if (savedOpacity !== null) {
        loaded.opacity = savedOpacity;
      }
      settingsRef.current = loaded;
      setSettings(loaded);
    } catch (err) {
      console.error("Failed to load settings", err);
    } finally {
      setLoading(false);
    }
  }, []);

  const update = useCallback(
    async (patch: Partial<AppSettings>) => {
      const next = { ...settingsRef.current, ...patch };
      settingsRef.current = next;
      setSettings(next);
      if (patch.opacity !== undefined) {
        window.localStorage.setItem(OPACITY_STORAGE_KEY, String(next.opacity));
      }
      saveQueue.current = saveQueue.current.then(() => saveSettings(next));
      await saveQueue.current;
    },
    [],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  return (
    <SettingsContext.Provider value={{ settings, loading, update, reload }}>
      {children}
    </SettingsContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useSettings(): SettingsContextValue {
  const ctx = useContext(SettingsContext);
  if (!ctx) {
    throw new Error("useSettings must be used within a SettingsProvider");
  }
  return ctx;
}
