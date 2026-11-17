import { useEffect, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { ExternalLink, Globe, Pause, Play, Power, Shield } from "lucide-react";
import { AppProvider, useApp } from "./state/AppContext";
import { Titlebar } from "./components/Titlebar";
import { MobileNav, Sidebar } from "./components/Navigation";
import { OptionsDrawer } from "./components/OptionsDrawer";
import { OptionModals } from "./components/OptionModals";
import { AdvancedModals } from "./components/AdvancedModals";
import { UpdaterModal } from "./components/UpdaterModal";
import { Toast } from "./components/ui";
import { DashboardView } from "./views/DashboardView";
import { LocationsView } from "./views/LocationsView";
import { LogsView } from "./views/LogsView";
import { SettingsView } from "./views/SettingsView";
import { SpeedView } from "./views/SimpleViews";
import { Flag } from "./components/Flag";
import { Logo } from "./components/Logo";
import { call } from "./lib/bridge";
import { formatDuration } from "./lib/config";
import { serverLabel } from "./lib/servers";
import { cn } from "./utils/cn";

const IS_TRAY_WINDOW = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("tray") === "1";

function CurrentView(): ReactNode {
  const { view } = useApp();
  switch (view) {
    case "locations":
      return <LocationsView />;
    case "logs":
      return <LogsView />;
    case "speed":
      return <SpeedView />;
    case "settings":
      return <SettingsView />;
    default:
      return <DashboardView />;
  }
}

function AnimatedView() {
  const { view } = useApp();
  return (
    <AnimatePresence mode="wait" initial={false}>
      <motion.div
        key={view}
        className="h-full"
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.16 }}
      >
        <CurrentView />
      </motion.div>
    </AnimatePresence>
  );
}

function TrayPopup() {
  const { t, settings, patchSettings, snapshot, server, connect, disconnect, togglePause } = useApp();
  const state = snapshot.state;
  const idle = state === "OFF" || state === "ERROR";
  const running = state === "RUNNING";
  const paused = state === "PAUSED";
  const busy = state === "PREPARING" || state === "CONNECTING" || state === "VERIFYING";

  const handlePrimary = () => {
    if (idle) {
      connect();
    } else if (paused) {
      togglePause();
    } else {
      disconnect();
    }
  };

  return (
    <div className="surface-dark flex h-full w-full flex-col justify-between overflow-hidden rounded-[24px] border border-white/14 bg-[#0e1514] p-4 text-fg select-none">
      <div className="flex items-center justify-between border-b border-white/8 pb-3">
        <div className="flex items-center gap-2.5">
          <span className="grid h-9 w-9 place-items-center rounded-xl bg-[#151f1d] border border-white/8">
            <Logo size={24} />
          </span>
          <div>
            <div className="text-[15px] font-bold leading-tight text-fg">{t("appName")}</div>
            <div className="text-[11px] font-medium text-mint">WitWizard</div>
          </div>
        </div>
        <div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1">
          <span
            className={cn(
              "h-2 w-2 rounded-full",
              running && "dot-live bg-mint",
              busy && "dot-busy bg-amber-fast",
              (idle || paused) && "bg-[#64748b]",
            )}
          />
          <span className="mono text-[11px] font-semibold text-fg2">
            {running || paused ? formatDuration(snapshot.uptimeSecs) : t("disconnected")}
          </span>
        </div>
      </div>

      <div className="my-2.5 flex items-center gap-3 rounded-2xl border border-white/8 bg-[#151e1c] p-3">
        <Flag code={server.flag} size={36} />
        <div className="min-w-0 flex-1">
          <div className="truncate text-[14px] font-bold text-fg">{serverLabel(server, settings.lang)}</div>
          <div className="mono truncate text-[11px] text-fg2">
            {snapshot.egress ? `${snapshot.egress.ip} · ${snapshot.egress.colo}` : `Region ${server.region}`}
          </div>
        </div>
        {(running || paused) && (
          <button
            type="button"
            onClick={togglePause}
            title={paused ? t("resume") : t("pause")}
            className="grid h-8 w-8 shrink-0 place-items-center rounded-full border border-white/15 bg-white/5 text-fg hover:border-mint hover:text-mint"
          >
            {paused ? <Play size={13} className="fill-current" /> : <Pause size={13} className="fill-current" />}
          </button>
        )}
      </div>

      <button
        type="button"
        onClick={handlePrimary}
        className={cn(
          "flex h-11 w-full items-center justify-center gap-2 rounded-full text-[14px] font-bold transition-all",
          idle && "border-0 bg-mint text-[#04231a] hover:bg-mint-hover",
          busy && "border border-amber-fast bg-transparent text-amber-fast",
          running && "border border-mint bg-mint/10 text-mint hover:bg-mint/20",
          paused && "border border-white/25 bg-transparent text-fg hover:border-mint hover:text-mint",
        )}
      >
        <Shield size={16} />
        {idle && t("connect")}
        {busy && t("connecting")}
        {running && t("disconnect")}
        {paused && t("resumeConnection")}
      </button>

      <div className="mt-2.5 flex flex-col gap-1 border-t border-white/8 pt-2.5">
        <button
          type="button"
          onClick={() => void call("tray_open_main", {}, () => undefined)}
          className="flex h-9 w-full items-center gap-2.5 rounded-xl border-0 bg-transparent px-3 text-[13px] font-medium text-fg hover:bg-white/8"
        >
          <ExternalLink size={15} className="text-mint" />
          <span>{t("trayShow")}</span>
        </button>
        <button
          type="button"
          onClick={() => {
            const next = settings.lang === "en" ? "fa" : "en";
            patchSettings({ lang: next });
          }}
          className="flex h-9 w-full items-center gap-2.5 rounded-xl border-0 bg-transparent px-3 text-[13px] font-medium text-fg hover:bg-white/8"
        >
          <Globe size={15} className="text-[#38bdf8]" />
          <span>{t("traySwitchLang")}</span>
        </button>
        <button
          type="button"
          onClick={() => void call("tray_quit_app", {}, () => undefined)}
          className="flex h-9 w-full items-center gap-2.5 rounded-xl border-0 bg-transparent px-3 text-[13px] font-semibold text-red-400 hover:bg-red-500/15"
        >
          <Power size={15} />
          <span>{t("trayQuit")}</span>
        </button>
      </div>
    </div>
  );
}

function Shell() {
  const { isMobile, view, settings } = useApp();

  useEffect(() => {
    const title = settings.lang === "fa" ? "ردشو" : "Radsho";
    document.title = title;
    void call("sync_locale", { lang: settings.lang }, () => undefined);
  }, [settings.lang]);

  if (IS_TRAY_WINDOW) {
    return <TrayPopup />;
  }

  if (isMobile) {
    return (
      <div
        className="relative h-full overflow-hidden bg-bg"
        style={{ paddingTop: "var(--safe-top, env(safe-area-inset-top, 0px))" }}
      >
        <AnimatedView />
        {view !== "locations" && <MobileNav />}
      </div>
    );
  }

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-[26px] border border-line bg-bg">
      <Titlebar />
      <div className="flex min-h-0 flex-1 gap-5 px-5 pb-5">
        <Sidebar />
        <main className="min-w-0 flex-1">
          <AnimatedView />
        </main>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <AppProvider>
      <Shell />
      {!IS_TRAY_WINDOW && (
        <>
          <OptionsDrawer />
          <OptionModals />
          <AdvancedModals />
          <UpdaterModal />
          <Toast />
        </>
      )}
    </AppProvider>
  );
}