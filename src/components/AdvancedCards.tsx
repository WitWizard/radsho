import { Activity, Check, ChevronRight, Gauge, ScanLine, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { ModalId } from "../types/vpn";
import type { TKey } from "../i18n/translations";

interface CardDefinition {
  id: "scanner" | "check" | "speed";
  icon: LucideIcon;
  color: string;
  title: TKey;
  action: TKey;
  modal: ModalId;
}

const CARDS: CardDefinition[] = [
  { id: "scanner", icon: ScanLine, color: "#06B6D4", title: "scannerTitle", action: "scannerAction", modal: "scanner" },
  { id: "check", icon: Activity, color: "#EC4899", title: "checkTitle", action: "checkAction", modal: "check" },
  { id: "speed", icon: Gauge, color: "#EAB308", title: "speedTitle", action: "speedAction", modal: "speed" },
];

export function AdvancedCards() {
  const { t, snapshot, setModal, isMobile } = useApp();
  const active = snapshot.state === "RUNNING" || snapshot.state === "PAUSED";

  const subtitle = (id: CardDefinition["id"]) => {
    if (id === "scanner") {
      return (
        <span className="mt-0.5 inline-flex items-center gap-1.5 text-[15px] text-fg2">
          <span
            className={cn("grid h-5 w-5 place-items-center rounded-full text-white", active ? "bg-mint" : "bg-[#94a3b8]")}
          >
            <Check size={12} strokeWidth={3} />
          </span>
          {active ? t("scannerRunning") : t("scannerIdle")}
        </span>
      );
    }
    return <span className="mt-0.5 text-[15px] text-fg2">{id === "check" ? t("checkSub") : t("speedSub")}</span>;
  };

  return (
    <div className={cn("grid gap-4", isMobile ? "grid-cols-1" : "grid-cols-3")}>
      {CARDS.map((card) => {
        const Icon = card.icon;
        return (
          <button
            key={card.id}
            type="button"
            onClick={() => setModal(card.modal)}
            className={cn(
              "group border border-line bg-card text-start shadow-[var(--shadow)] transition-transform hover:-translate-y-0.5",
              isMobile ? "flex items-center gap-4 rounded-[28px] p-4" : "flex flex-col rounded-[28px] p-5",
            )}
          >
            <div className="flex flex-1 items-center gap-4">
              <span
                className="grid h-[58px] w-[58px] shrink-0 place-items-center rounded-full border-2 bg-white/0"
                style={{ borderColor: card.color, color: "#0f172a" }}
              >
                <Icon size={24} className="text-fg" strokeWidth={1.8} />
              </span>
              <span className="flex min-w-0 flex-col">
                <span className="text-[21px] font-semibold text-fg">{t(card.title)}</span>
                {subtitle(card.id)}
              </span>
            </div>
            {isMobile ? (
              <span className="grid h-12 w-12 shrink-0 place-items-center rounded-full bg-chip text-fg">
                <ChevronRight size={20} className="rtl:rotate-180" />
              </span>
            ) : (
              <span className="mt-5 flex h-12 items-center justify-between rounded-full bg-chip ps-5 pe-1.5 text-[16px] font-medium text-fg">
                {t(card.action)}
                <span className="grid h-9 w-9 place-items-center rounded-full bg-black/5 transition-colors group-hover:bg-mint group-hover:text-[#06251a]">
                  <ChevronRight size={18} className="rtl:rotate-180" />
                </span>
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
