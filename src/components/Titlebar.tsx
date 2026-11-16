import { Bell, Maximize2, Minus, X } from "lucide-react";
import { useApp } from "../state/AppContext";
import { windowAction } from "../lib/bridge";
import { CREATORS, CreatorAvatar } from "./AboutCard";

export function Titlebar() {
  const { t, updateInfo, setModal, setView, notify } = useApp();

  return (
    <header data-tauri-drag-region className="drag-region flex h-[68px] shrink-0 items-center gap-3 px-5">
      <button
        type="button"
        onClick={() => setView("settings")}
        className="no-drag flex items-center gap-3 rounded-full border border-line bg-card py-1.5 pe-5 ps-2 shadow-[var(--shadow)] transition-colors hover:border-mint/40"
      >
        <span className="flex items-center -space-x-2 rtl:space-x-reverse">
          {CREATORS.map((creator) => (
            <CreatorAvatar key={creator.name} creator={creator} size={34} />
          ))}
        </span>
        <span className="text-[14.5px] font-semibold text-fg">{t("badgeCredit")}</span>
      </button>

      <div className="flex-1" />

      {updateInfo && (
        <button
          type="button"
          onClick={() => setModal("updater")}
          className="no-drag inline-flex items-center gap-2 rounded-full border border-mint/40 bg-mint/12 px-4 py-2 text-[13px] font-bold text-[#00a870] dark:text-mint"
        >
          <span className="dot-live h-2 w-2 rounded-full bg-mint" />
          {t("updateAvailable")} · v{updateInfo.appLatest}
        </button>
      )}

      <button
        type="button"
        aria-label={t("notifications")}
        onClick={() => (updateInfo ? setModal("updater") : notify(t("updaterUpToDate")))}
        className="no-drag grid h-10 w-10 place-items-center rounded-full border border-line bg-card text-fg shadow-[var(--shadow)]"
      >
        <Bell size={18} className="fill-current" />
      </button>

      <div className="no-drag ms-2 flex items-center gap-1">
        <button
          type="button"
          title={t("minimize")}
          aria-label={t("minimize")}
          onClick={() => windowAction("minimize")}
          className="grid h-9 w-9 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:bg-chip hover:text-fg"
        >
          <Minus size={16} />
        </button>
        <button
          type="button"
          title={t("maximize")}
          aria-label={t("maximize")}
          onClick={() => windowAction("maximize")}
          className="grid h-9 w-9 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:bg-chip hover:text-fg"
        >
          <Maximize2 size={14} />
        </button>
        <button
          type="button"
          title={t("closeWindow")}
          aria-label={t("closeWindow")}
          onClick={() => windowAction("close")}
          className="grid h-9 w-9 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:bg-red-500 hover:text-white"
        >
          <X size={16} />
        </button>
      </div>
    </header>
  );
}