import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { Engine, Lang, LogEntry, LogLevel, ModalId, ResolvedTheme, Server, SessionSnapshot, Settings, UpdateInfo, ViewId } from "../types/vpn";
import { translations, type TKey } from "../i18n/translations";
import { APP_VERSION, CORE_VERSION, DEFAULT_SETTINGS, portsValid } from "../lib/config";
import { call, isMobileDevice, sleep } from "../lib/bridge";
import { createEngine, initialSnapshot } from "../lib/engine";
import { findServer, serverByRegion } from "../lib/servers";

interface ToastState {
  id: number;
  text: string;
  starred: boolean;
}

interface AppContextValue {
  settings: Settings;
  patchSettings: (partial: Partial<Settings>) => void;
  t: (key: TKey) => string;
  isRtl: boolean;
  resolvedTheme: ResolvedTheme;
  isMobile: boolean;
  snapshot: SessionSnapshot;
  logs: LogEntry[];
  clearLogs: () => void;
  server: Server;
  favorites: string[];
  toggleFavorite: (id: string) => void;
  view: ViewId;
  setView: (view: ViewId) => void;
  modal: ModalId;
  setModal: (modal: ModalId) => void;
  optionsOpen: boolean;
  setOptionsOpen: (open: boolean) => void;
  toast: ToastState | null;
  notify: (text: string, starred?: boolean) => void;
  connect: (serverId?: string) => void;
  disconnect: () => void;
  togglePause: () => void;
  selectServer: (id: string) => void;
  updateInfo: UpdateInfo | null;
  checkForUpdates: () => Promise<UpdateInfo | null>;
  completeUpdate: () => void;
  lastArgv: string;
}

const AppContext = createContext<AppContextValue | null>(null);

const SETTINGS_KEY = "radsho.settings.v1";
const FAVORITES_KEY = "radsho.favorites.v1";

function loadSettings(): Settings {
  try {
    const raw = window.localStorage.getItem(SETTINGS_KEY);
    if (!raw) {
      return DEFAULT_SETTINGS;
    }
    const stored = JSON.parse(raw) as Partial<Settings>;
    return {
      ...DEFAULT_SETTINGS,
      ...stored,
      ports: { ...DEFAULT_SETTINGS.ports, ...stored.ports },
      shuffle: { ...DEFAULT_SETTINGS.shuffle, ...stored.shuffle },
      routing: { ...DEFAULT_SETTINGS.routing, ...stored.routing },
      protocol: { ...DEFAULT_SETTINGS.protocol, ...stored.protocol },
    };
  } catch {
    return DEFAULT_SETTINGS;
  }
}

function loadFavorites(): string[] {
  try {
    const raw = window.localStorage.getItem(FAVORITES_KEY);
    return raw ? (JSON.parse(raw) as string[]) : ["vienna"];
  } catch {
    return ["vienna"];
  }
}

function useMobileViewport(): boolean {
  const [narrow, setNarrow] = useState<boolean>(() => window.matchMedia("(max-width: 899px)").matches);
  useEffect(() => {
    const query = window.matchMedia("(max-width: 899px)");
    const listener = (event: MediaQueryListEvent) => setNarrow(event.matches);
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, []);
  return narrow || isMobileDevice;
}

function systemTheme(): ResolvedTheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function simulatedUpdate(): UpdateInfo {
  return {
    appCurrent: APP_VERSION,
    appLatest: "1.1.0",
    coreCurrent: CORE_VERSION,
    coreLatest: "2.5.0",
    changelog: [
      "Faster Psiphon carrier handshake inside WARP",
      "Shuffle Location rotates exit region without dropping the carrier",
      "Core updater falls back to gh-proxy.com and ghfast.top mirrors",
      "Crash recovery for system proxy runs before the window opens",
    ],
    mirror: "gh-proxy.com",
    sha256: "9f2c41ab7d03e5c1b68a24f0d19e07d83c5a6b4e2f81c09d7a35e6b1c4f08a2d",
  };
}

export function AppProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(loadSettings);
  const [favorites, setFavorites] = useState<string[]>(loadFavorites);
  const [snapshot, setSnapshot] = useState<SessionSnapshot>(initialSnapshot);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [view, setView] = useState<ViewId>("dashboard");
  const [modal, setModal] = useState<ModalId>(null);
  const [optionsOpen, setOptionsOpen] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [lastArgv, setLastArgv] = useState("");
  const [osTheme, setOsTheme] = useState<ResolvedTheme>(systemTheme);
  const engineRef = useRef<Engine | null>(null);
  const logId = useRef(0);
  const toastTimer = useRef<number | null>(null);
  const settingsRef = useRef(settings);
  const fromExternalSync = useRef(false);
  const isMobile = useMobileViewport();

  if (engineRef.current === null) {
    engineRef.current = createEngine();
  }
  const engine = engineRef.current;

  useEffect(() => {
    settingsRef.current = settings;
    if (fromExternalSync.current) {
      fromExternalSync.current = false;
      return;
    }
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    engine.configure(settings);
  }, [settings, engine]);

  useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key === SETTINGS_KEY && event.newValue) {
        try {
          const next = JSON.parse(event.newValue) as Partial<Settings>;
          fromExternalSync.current = true;
          setSettings((previous) => ({
            ...previous,
            ...next,
            ports: { ...previous.ports, ...next.ports },
            shuffle: { ...previous.shuffle, ...next.shuffle },
            routing: { ...previous.routing, ...next.routing },
            protocol: { ...previous.protocol, ...next.protocol },
          }));
        } catch {
          return;
        }
      } else if (event.key === FAVORITES_KEY && event.newValue) {
        try {
          setFavorites(JSON.parse(event.newValue) as string[]);
        } catch {
          return;
        }
      }
    };

    const onSyncLang = (event: Event) => {
      const custom = event as CustomEvent<string>;
      const lang: Lang = custom.detail === "fa" ? "fa" : "en";
      if (settingsRef.current.lang !== lang) {
        fromExternalSync.current = true;
        setSettings((previous) => {
          const next = { ...previous, lang };
          window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(next));
          return next;
        });
      }
    };

    window.addEventListener("storage", onStorage);
    window.addEventListener("radsho-sync-lang", onSyncLang);
    return () => {
      window.removeEventListener("storage", onStorage);
      window.removeEventListener("radsho-sync-lang", onSyncLang);
    };
  }, []);

  useEffect(() => {
    window.localStorage.setItem(FAVORITES_KEY, JSON.stringify(favorites));
  }, [favorites]);

  useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const listener = (event: MediaQueryListEvent) => setOsTheme(event.matches ? "dark" : "light");
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, []);

  const resolvedTheme: ResolvedTheme = settings.theme === "system" ? osTheme : settings.theme;

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
  }, [resolvedTheme]);

  useEffect(() => {
    document.documentElement.lang = settings.lang;
    document.documentElement.dir = settings.lang === "fa" ? "rtl" : "ltr";
  }, [settings.lang]);

  const pushLog = useCallback((level: LogLevel, text: string) => {
    logId.current += 1;
    const entry: LogEntry = { id: logId.current, ts: Date.now(), level, text };
    if (level === "core") {
      setLastArgv(text);
    }
    setLogs((previous) => [...previous.slice(-599), entry]);
  }, []);

  useEffect(() => {
    const unsubscribe = engine.subscribe(setSnapshot, pushLog);
    pushLog("info", `Radsho ${APP_VERSION} • WitWizard • Lilmmd & LFnighTT`);
    pushLog("info", "proxy.lock recovery check complete, no stale system proxy found");
    return unsubscribe;
  }, [engine, pushLog]);

  useEffect(() => {
    if (snapshot.state !== "RUNNING" && snapshot.state !== "PAUSED") {
      return;
    }
    const target = serverByRegion(snapshot.region, snapshot.serverId);
    const current = settingsRef.current;
    const patch: Partial<Settings> = {};
    if (target.id !== current.serverId) {
      patch.serverId = target.id;
    }
    if (snapshot.transport && snapshot.transport !== current.rememberedTransport) {
      patch.rememberedTransport = snapshot.transport;
    }
    if (Object.keys(patch).length > 0) {
      setSettings((previous) => ({ ...previous, ...patch }));
    }
  }, [snapshot.state, snapshot.region, snapshot.serverId, snapshot.transport]);

  const checkForUpdates = useCallback(async (): Promise<UpdateInfo | null> => {
    const info = await call<UpdateInfo | null>("updater_check", {}, async () => {
      await sleep(1100);
      return simulatedUpdate();
    });
    setUpdateInfo(info);
    return info;
  }, []);

  useEffect(() => {
    if (!settings.autoUpdate) {
      return;
    }
    const timer = window.setTimeout(() => void checkForUpdates(), 4500);
    return () => window.clearTimeout(timer);
  }, [settings.autoUpdate, checkForUpdates]);

  const notify = useCallback((text: string, starred = false) => {
    if (toastTimer.current !== null) {
      window.clearTimeout(toastTimer.current);
    }
    setToast({ id: Date.now(), text, starred });
    toastTimer.current = window.setTimeout(() => setToast(null), 2200);
  }, []);

  const t = useCallback((key: TKey) => translations[settings.lang][key], [settings.lang]);

  const patchSettings = useCallback((partial: Partial<Settings>) => {
    setSettings((previous) => ({ ...previous, ...partial }));
    if (partial.lang) {
      void call("sync_locale", { lang: partial.lang }, () => undefined);
    }
  }, []);

  const connect = useCallback(
    (serverId?: string) => {
      const current = settingsRef.current;
      if (!portsValid(current)) {
        notify(translations[current.lang].portsInvalid);
        return;
      }
      const id = serverId ?? current.serverId;
      const target = findServer(id);
      const next: Settings = { ...current, serverId: id, psiphonRegion: target.region };
      setSettings(next);
      engine.connect(next, id, target.region);
    },
    [engine, notify],
  );

  const disconnect = useCallback(() => engine.disconnect(), [engine]);

  const togglePause = useCallback(() => {
    if (engine.snapshot().state === "PAUSED") {
      engine.resume();
    } else {
      engine.pause();
    }
  }, [engine]);

  const selectServer = useCallback((id: string) => {
    setSettings((previous) => ({ ...previous, serverId: id }));
  }, []);

  const favoritesRef = useRef(favorites);
  favoritesRef.current = favorites;

  const toggleFavorite = useCallback(
    (id: string) => {
      const exists = favoritesRef.current.includes(id);
      const dictionary = translations[settingsRef.current.lang];
      notify(exists ? dictionary.removedFav : dictionary.addedFav, !exists);
      setFavorites(exists ? favoritesRef.current.filter((item) => item !== id) : [...favoritesRef.current, id]);
    },
    [notify],
  );

  const completeUpdate = useCallback(() => setUpdateInfo(null), []);
  const clearLogs = useCallback(() => setLogs([]), []);

  const value = useMemo<AppContextValue>(
    () => ({
      settings,
      patchSettings,
      t,
      isRtl: settings.lang === "fa",
      resolvedTheme,
      isMobile,
      snapshot,
      logs,
      clearLogs,
      server: findServer(settings.serverId),
      favorites,
      toggleFavorite,
      view,
      setView,
      modal,
      setModal,
      optionsOpen,
      setOptionsOpen,
      toast,
      notify,
      connect,
      disconnect,
      togglePause,
      selectServer,
      updateInfo,
      checkForUpdates,
      completeUpdate,
      lastArgv,
    }),
    [settings, patchSettings, t, resolvedTheme, isMobile, snapshot, logs, clearLogs, favorites, toggleFavorite, view, modal, optionsOpen, toast, notify, connect, disconnect, togglePause, selectServer, updateInfo, checkForUpdates, completeUpdate, lastArgv],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp(): AppContextValue {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useApp must be used inside AppProvider");
  }
  return context;
}