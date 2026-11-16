import { ChevronRight, Bolt, RefreshCw, Shuffle, Split, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { ModalId } from "../types/vpn";
import type { TKey } from "../i18n/translations";
import { Toggle } from "./ui";

interface OptionCardsProps {
  variant: "inline" | "drawer";
}

interface OptionDefinition {
  id: "shuffle" | "paths" | "protocol";
  icon: LucideIcon;
  title: TKey;
  desc: TKey;
  edit: TKey;
  modal: ModalId;
}

const OPTIONS: OptionDefinition[] = [
  { id: "shuffle", icon: Shuffle, title: "shuffleTitle", desc: "shuffleDesc", edit: "editShuffle", modal: "shuffle" },
  { id: "paths", icon: Split, title: "multiTitle", desc: "multiDesc", edit: "editMulti", modal: "paths" },
  { id: "protocol", icon: RefreshCw, title: "protocolTitle", desc: "protocolDesc", edit: "editProtocol", modal: "protocol" },
];

export function OptionCards({ variant }: OptionCardsProps) {
  const { t, settings, patchSettings, setModal, setOptionsOpen, notify } = useApp();

  const valueOf = (id: OptionDefinition["id"]): boolean => {
    if (id === "shuffle") {
      return settings.shuffle.enabled;
    }
    if (id === "paths") {
      return settings.routing.enabled;
    }
    return settings.protocol.auto;
  };

  const setValue = (id: OptionDefinition["id"], value: boolean) => {
    if (id === "shuffle") {
      patchSettings({ shuffle: { ...settings.shuffle, enabled: value } });
      if (value && !settings.psiphon) {
        notify(t("shuffleNeedsPsiphon"));
      }
    } else if (id === "paths") {
      patchSettings({ routing: { ...settings.routing, enabled: value } });
    } else {
      patchSettings({ protocol: { ...settings.protocol, auto: value } });
    }
  };

  const openEdit = (modal: ModalId) => {
    setOptionsOpen(false);
    window.setTimeout(() => setModal(modal), variant === "drawer" ? 220 : 0);
  };

  return (
    <div className={cn("flex flex-col gap-3", variant === "inline" && "h-full justify-between")}>
      {OPTIONS.map((option) => {
        const Icon = option.icon;
        const checked = valueOf(option.id);
        return (
          <div
            key={option.id}
            className={cn(
              "rounded-[24px] border border-white/6 bg-hero-in",
              variant === "inline" ? "flex flex-1 items-center gap-4 p-5" : "p-4",
            )}
          >
            {variant === "inline" ? (
              <>
                <span className="grid h-[58px] w-[58px] shrink-0 place-items-center rounded-full border border-white/25 text-white">
                  <Icon size={24} strokeWidth={1.6} />
                </span>
                <div className="min-w-0 flex-1">
                  <h3 className="m-0 text-[19px] font-semibold text-fg">{t(option.title)}</h3>
                  <p className="m-0 mt-1 text-[14px] leading-snug text-fg2">{t(option.desc)}</p>
                </div>
                <div className="flex shrink-0 flex-col items-center gap-4 self-stretch">
                  <Toggle checked={checked} onChange={(value) => setValue(option.id, value)} label={t(option.title)} />
                  <button
                    type="button"
                    aria-label={t(option.edit)}
                    onClick={() => openEdit(option.modal)}
                    className="grid h-8 w-8 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:text-mint"
                  >
                    <Bolt size={22} strokeWidth={1.6} />
                  </button>
                </div>
              </>
            ) : (
              <>
                <div className="flex items-center gap-3">
                  <span className="grid h-12 w-12 shrink-0 place-items-center rounded-full border border-white/25 text-white">
                    <Icon size={22} strokeWidth={1.6} />
                  </span>
                  <h3 className="m-0 flex-1 text-[18px] font-semibold text-fg">{t(option.title)}</h3>
                  <Toggle checked={checked} onChange={(value) => setValue(option.id, value)} label={t(option.title)} />
                </div>
                <p className="m-0 mt-3 text-[14px] leading-snug text-fg2">{t(option.desc)}</p>
                <button
                  type="button"
                  onClick={() => openEdit(option.modal)}
                  className="mt-3 flex h-12 w-full items-center justify-center gap-1 rounded-full border border-white/10 bg-transparent text-[15px] font-medium text-fg hover:bg-white/5"
                >
                  {t(option.edit)}
                  <ChevronRight size={16} className="text-fg2 rtl:rotate-180" />
                </button>
              </>
            )}
          </div>
        );
      })}
    </div>
  );
}
