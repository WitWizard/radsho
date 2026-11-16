import { useMemo, useState } from "react";
import { ArrowLeft, Bot, ListFilter, Network, Play, Search, ShieldCheck, Star, Zap, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import { CATEGORY_ORDER, SERVERS, serverLabel } from "../lib/servers";
import { categoryLabelKey } from "../i18n/translations";
import type { Server, ServerCategory, SortMode } from "../types/vpn";
import { Flag } from "../components/Flag";

const CATEGORY_ICONS: Record<ServerCategory, LucideIcon> = {
  fast: Zap,
  streaming: Play,
  gaming: Bot,
  antisanction: ShieldCheck,
  doublehop: Network,
};

function pingColor(ping: number): string {
  if (ping < 100) {
    return "bg-mint";
  }
  return ping < 170 ? "bg-amber-fast" : "bg-red-400";
}

export function LocationsView() {
  const { t, settings, snapshot, favorites, toggleFavorite, connect, disconnect, setView, isMobile } = useApp();
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<ServerCategory | null>("fast");
  const [sort, setSort] = useState<SortMode>("none");

  const connectedId = snapshot.state === "OFF" || snapshot.state === "ERROR" ? null : settings.serverId;

  const featured = useMemo(() => SERVERS.filter((server) => (category ? server.tags.includes(category) : true)), [category]);

  const list = useMemo(() => {
    const needle = query.trim().toLowerCase();
    let items = SERVERS.filter((server) => {
      if (!needle) {
        return true;
      }
      return [server.city, server.cityFa, server.tag, server.region].some((field) => field.toLowerCase().includes(needle));
    });
    if (sort === "fav") {
      items = items.filter((server) => favorites.includes(server.id));
    }
    if (sort === "az") {
      items = [...items].sort((a, b) => a.city.localeCompare(b.city));
    }
    if (sort === "za") {
      items = [...items].sort((a, b) => b.city.localeCompare(a.city));
    }
    return items;
  }, [query, sort, favorites]);

  const toggleSort = (mode: SortMode) => setSort((previous) => (previous === mode ? "none" : mode));

  const action = (server: Server) => (connectedId === server.id ? disconnect() : connect(server.id));

  const pill = (active: boolean) =>
    cn(
      "inline-flex h-10 items-center justify-center rounded-full border text-[15px] font-medium transition-colors",
      active ? "border-white bg-white text-[#0e1514]" : "border-white/10 bg-transparent text-fg2 hover:text-fg",
    );

  return (
    <div className={cn("surface-dark flex h-full min-h-0 flex-col overflow-hidden bg-hero text-fg", !isMobile && "rounded-[36px]")}>
      <div className="mx-auto flex w-full max-w-4xl shrink-0 flex-col px-5 pt-5">
        <div className="relative flex h-12 items-center justify-center">
          <button
            type="button"
            aria-label={t("back")}
            onClick={() => setView("dashboard")}
            className="absolute start-0 grid h-11 w-11 place-items-center rounded-full border-0 bg-transparent text-fg"
          >
            <ArrowLeft size={24} className="rtl:rotate-180" />
          </button>
          <h1 className="m-0 text-[22px] font-medium">{t("vpnLocations")}</h1>
        </div>

        <label className="mt-4 flex h-[58px] items-center gap-3 rounded-full border border-white/10 bg-white/3 px-5 focus-within:border-white focus-within:ring-2 focus-within:ring-white/20">
          <Search size={22} className="shrink-0 text-fg" />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("searchLocation")}
            className="min-w-0 flex-1 border-0 bg-transparent text-[16px] text-fg outline-none placeholder:text-fg2 focus-visible:outline-none"
          />
        </label>

        {!query && (
          <>
            <div className="scroll-x mt-4 flex gap-3">
              {CATEGORY_ORDER.map((id) => {
                const Icon = CATEGORY_ICONS[id];
                const active = category === id;
                return (
                  <button
                    key={id}
                    type="button"
                    onClick={() => setCategory(active ? null : id)}
                    className={cn(pill(active), "shrink-0 gap-2 ps-1.5 pe-4", !active && "border-transparent bg-white/5")}
                  >
                    <span className={cn("grid h-7 w-7 place-items-center rounded-full", active ? "bg-[#0e1514] text-white" : "border border-white/25")}>
                      <Icon size={14} />
                    </span>
                    {t(categoryLabelKey[id])}
                  </button>
                );
              })}
            </div>

            <div className="scroll-x -mx-5 mt-5 flex gap-3 px-5 pb-1 pt-2" aria-label={t("featured")}>
              {featured.map((server) => {
                const current = connectedId === server.id;
                return (
                  <div
                    key={server.id}
                    className={cn(
                      "flex w-[190px] shrink-0 flex-col gap-3 rounded-[24px] border bg-card p-3 transition-all",
                      current ? "border-mint shadow-[0_0_0_5px_rgba(0,229,153,0.14)]" : "border-white/8 opacity-80",
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <Flag code={server.flag} size={40} ring={current} />
                      <span className="truncate text-[16px] font-medium">{serverLabel(server, settings.lang)}</span>
                    </div>
                    <button
                      type="button"
                      onClick={() => action(server)}
                      className={cn(
                        "h-11 rounded-full border bg-transparent text-[15px] font-medium",
                        current ? "border-mint text-mint" : "border-white/25 text-fg2 hover:border-mint hover:text-mint",
                      )}
                    >
                      {current ? t("disconnect") : t("connect")}
                    </button>
                  </div>
                );
              })}
            </div>
          </>
        )}
      </div>

      <div className={cn("mx-auto mt-4 flex min-h-0 w-full max-w-4xl flex-1 flex-col bg-card px-5 pb-4 pt-3", isMobile ? "rounded-t-[32px]" : "rounded-[32px]")}>
        <div className="mx-auto h-1 w-14 shrink-0 rounded-full bg-white/70" />
        <div className="mt-4 flex shrink-0 items-center gap-3">
          <ListFilter size={22} className="text-fg2" />
          <span className="flex-1 text-[19px] font-medium">{t("sortBy")}</span>
          <button type="button" onClick={() => toggleSort("az")} className={cn(pill(sort === "az"), "min-w-[60px] px-4 text-[14px]")}>
            {t("sortAz")}
          </button>
          <button type="button" onClick={() => toggleSort("za")} className={cn(pill(sort === "za"), "min-w-[60px] px-4 text-[14px]")}>
            {t("sortZa")}
          </button>
          <button type="button" aria-label="favorites" aria-pressed={sort === "fav"} onClick={() => toggleSort("fav")} className={cn(pill(sort === "fav"), "w-10")}>
            <Star size={17} className={sort === "fav" ? "fill-current" : ""} />
          </button>
        </div>

        <ul className="scroll-y m-0 mt-3 flex min-h-0 flex-1 list-none flex-col p-0">
          {list.length === 0 && <li className="py-10 text-center text-fg2">{t("noResults")}</li>}
          {list.map((server) => {
            const current = connectedId === server.id;
            const fav = favorites.includes(server.id);
            return (
              <li key={server.id} className="flex items-center gap-3 py-[10px]">
                <Flag code={server.flag} size={42} />
                <div className="flex min-w-0 flex-1 flex-col">
                  <span className="truncate text-[18px] font-medium">{serverLabel(server, settings.lang)}</span>
                  <span className="mono flex items-center gap-1.5 text-[12px] text-fg2">
                    <span className={cn("h-1.5 w-1.5 rounded-full", pingColor(server.ping))} />
                    {server.ping} {t("msUnit")}
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => action(server)}
                  className={cn(
                    "h-10 min-w-[96px] rounded-full border bg-transparent px-4 text-[14px] font-medium transition-colors",
                    current ? "border-mint text-mint" : "border-white/25 text-fg2 hover:border-mint hover:text-mint",
                  )}
                >
                  {current ? t("disconnect") : t("connect")}
                </button>
                <button
                  type="button"
                  aria-label="favorite"
                  aria-pressed={fav}
                  onClick={() => toggleFavorite(server.id)}
                  className="grid h-10 w-10 place-items-center rounded-full border-0 bg-transparent"
                >
                  <Star size={24} strokeWidth={1.6} className={fav ? "fill-star text-star" : "text-fg2"} />
                </button>
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
}
