import { ArrowDown, ArrowUp, ChevronRight, Loader2, Pause, Play, Search, Star, Zap } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import { FAST_QUICK_IDS, findServer, serverLabel } from "../lib/servers";
import { formatDuration, formatRate } from "../lib/config";
import type { ConnState } from "../types/vpn";
import type { TKey } from "../i18n/translations";
import { Flag } from "./Flag";

const STATUS_KEY: Record<ConnState, TKey> = {
  OFF: "disconnected",
  PREPARING: "preparing",
  CONNECTING: "connecting",
  VERIFYING: "verifying",
  PAUSED: "paused",
  RUNNING: "connectedTo",
  ERROR: "errorState",
};

function isBusy(state: ConnState): boolean {
  return state === "PREPARING" || state === "CONNECTING" || state === "VERIFYING";
}

export function ConnectionHub() {
  const { t, settings, snapshot, server, favorites, toggleFavorite, connect, disconnect, togglePause, selectServer, setView, isMobile } = useApp();
  const state = snapshot.state;
  const busy = isBusy(state);
  const idle = state === "OFF" || state === "ERROR";
  const showTimer = state === "RUNNING" || state === "PAUSED";
  const isFav = favorites.includes(server.id);
  const quick = FAST_QUICK_IDS.filter((id) => id !== server.id)
    .slice(0, 2)
    .map(findServer);

  const onPrimary = () => {
    if (idle) {
      connect();
    } else if (state === "PAUSED") {
      togglePause();
    } else {
      disconnect();
    }
  };

  const onQuick = (id: string) => {
    if (idle) {
      selectServer(id);
    } else {
      connect(id);
    }
  };

  const dotClass = cn(
    "h-[22px] w-[22px] rounded-full border-[5px] border-white/10",
    state === "RUNNING" && "dot-live bg-mint",
    busy && "dot-busy bg-amber-fast",
    state === "PAUSED" && "bg-[#64748b]",
    state === "OFF" && "bg-[#64748b]",
    state === "ERROR" && "bg-red-500",
  );

  return (
    <div className="flex h-full min-w-0 flex-col">
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          <span className={dotClass} />
          <span className={cn("truncate text-[15px] font-medium", state === "OFF" || state === "PAUSED" ? "text-fg2" : "text-fg")}>
            {t(STATUS_KEY[state])}
          </span>
        </div>
        {showTimer && (
          <span className={cn("mono text-[15px]", state === "PAUSED" ? "text-fg2/60" : "text-fg2")}>{formatDuration(snapshot.uptimeSecs)}</span>
        )}
      </div>

      <div className="mt-6 flex items-center gap-4">
        <Flag code={server.flag} size={isMobile ? 52 : 56} />
        <h2 className={cn("m-0 min-w-0 flex-1 truncate font-semibold tracking-tight text-fg", isMobile ? "text-[32px]" : "text-[38px]")}>
          {serverLabel(server, settings.lang)}
          {server.tags.includes("fast") && (
            <span
              className={cn(
                "ms-3 inline-flex translate-y-[-4px] items-center gap-1 rounded-full border border-amber-fast align-middle text-[15px] font-semibold text-amber-fast",
                isMobile ? "h-8 w-8 justify-center" : "px-3 py-1",
              )}
            >
              <Zap size={14} className="fill-amber-fast" />
              {!isMobile && t("fast")}
            </span>
          )}
        </h2>
        <button
          type="button"
          aria-label="favorite"
          aria-pressed={isFav}
          onClick={() => toggleFavorite(server.id)}
          className="grid h-10 w-10 shrink-0 place-items-center rounded-full border-0 bg-transparent"
        >
          <Star size={26} className={isFav ? "fill-star text-star" : "text-[#475569]"} strokeWidth={1.6} />
        </button>
      </div>

      {snapshot.egress && state !== "OFF" && (
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <span className="mono rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-fg2">
            {snapshot.egress.ip} · {snapshot.egress.colo} · {snapshot.egress.warp.toUpperCase() === "ON" ? "WARP" : "WARP+"}
          </span>
          {state === "RUNNING" && (
            <span className="mono inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-fg2">
              <ArrowDown size={12} className="text-mint" />
              {formatRate(snapshot.rxBps)}
              <ArrowUp size={12} className="text-amber-fast" />
              {formatRate(snapshot.txBps)}
            </span>
          )}
        </div>
      )}

      <div className={cn("mt-6 flex gap-3", isMobile ? "flex-col" : "flex-wrap items-center")}>
        <span className="text-[22px] font-medium text-fg">{t("moreFastServers")}</span>
        <div className="flex flex-1 items-center gap-3">
          {quick.map((item) => (
            <button
              key={item.id}
              type="button"
              onClick={() => onQuick(item.id)}
              className="inline-flex items-center gap-2 rounded-full border-0 bg-white/6 py-2 pe-4 ps-2 text-[15px] font-medium text-fg transition-colors hover:bg-white/12"
            >
              <Flag code={item.flag} size={28} />
              {settings.lang === "fa" ? item.cityFa : item.city}
            </button>
          ))}
          {isMobile && (
            <button
              type="button"
              aria-label={t("searchLocation")}
              onClick={() => setView("locations")}
              className="ms-auto grid h-10 w-10 place-items-center rounded-full border-0 bg-transparent text-fg"
            >
              <Search size={24} />
            </button>
          )}
        </div>
      </div>

      {!isMobile && (
        <button
          type="button"
          onClick={() => setView("locations")}
          className="mt-5 flex h-[60px] w-full items-center gap-3 rounded-full border border-white/10 bg-white/3 px-5 text-start text-[16px] font-medium text-fg transition-colors hover:bg-white/6"
        >
          <Search size={22} className="shrink-0" />
          <span className="flex-1">{t("searchLocation")}</span>
          <ChevronRight size={20} className="text-fg2 rtl:rotate-180" />
        </button>
      )}

      <div className="mt-auto flex items-center gap-4 pt-6">
        <button
          type="button"
          onClick={onPrimary}
          className={cn(
            "flex h-[62px] min-w-0 flex-1 items-center justify-center gap-2 rounded-full text-[19px] font-semibold transition-all",
            idle && "border-0 bg-mint text-[#04231a] ring-[7px] ring-mint/12 hover:bg-mint-hover",
            busy && "border-[1.5px] border-amber-fast bg-transparent text-amber-fast",
            state === "RUNNING" && "border-[1.5px] border-mint bg-transparent text-mint hover:bg-mint/10",
            state === "PAUSED" && "border-[1.5px] border-white/25 bg-transparent text-fg2 hover:border-mint hover:text-mint",
          )}
        >
          {busy && <Loader2 size={20} className="spin-slow" />}
          {idle && t("connect")}
          {busy && t(STATUS_KEY[state])}
          {state === "RUNNING" && t("disconnect")}
          {state === "PAUSED" && t("resumeConnection")}
        </button>
        <button
          type="button"
          aria-label={state === "PAUSED" ? t("resume") : t("pause")}
          disabled={state !== "RUNNING" && state !== "PAUSED"}
          onClick={togglePause}
          className={cn(
            "grid h-[58px] w-[58px] shrink-0 place-items-center rounded-full border-[1.5px] bg-transparent transition-all disabled:cursor-not-allowed",
            state === "RUNNING" && "border-white text-white hover:bg-white/10",
            state === "PAUSED" && "border-mint text-mint ring-[7px] ring-mint/15",
            state !== "RUNNING" && state !== "PAUSED" && "border-white/20 text-white/35",
          )}
        >
          <span
            className={cn(
              "grid h-7 w-7 place-items-center rounded-full",
              state === "RUNNING" && "bg-white text-hero",
              state === "PAUSED" && "bg-mint text-hero",
              state !== "RUNNING" && state !== "PAUSED" && "bg-white/35 text-hero",
            )}
          >
            {state === "PAUSED" ? <Play size={13} className="fill-current" /> : <Pause size={13} className="fill-current" />}
          </span>
        </button>
      </div>
      {state === "ERROR" && <p className="mt-3 text-sm text-red-400">{t("errorState")}</p>}
    </div>
  );
}
