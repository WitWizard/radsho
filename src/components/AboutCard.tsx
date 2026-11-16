import { useState } from "react";
import { ShieldCheck, Terminal } from "lucide-react";
import { useApp } from "../state/AppContext";
import { APP_VERSION } from "../lib/config";
import type { TKey } from "../i18n/translations";
import { Logo } from "./Logo";

export interface CreatorProfile {
  name: string;
  roleKey: TKey;
  avatarUrl: string;
  gradient: string;
  accent: string;
  kind: "lilmmd" | "lfnightt";
}

export const CREATORS: CreatorProfile[] = [
  {
    name: "Lilmmd",
    roleKey: "roleLilmmd",
    avatarUrl: "/lilmmd.png",
    gradient: "from-[#00e599] via-[#10b981] to-[#0ea5e9]",
    accent: "#00E599",
    kind: "lilmmd",
  },
  {
    name: "LFnighTT",
    roleKey: "roleLfnightt",
    avatarUrl: "/lfnightt.png",
    gradient: "from-[#38bdf8] via-[#6366f1] to-[#00e599]",
    accent: "#38BDF8",
    kind: "lfnightt",
  },
];

export function CreatorAvatar({ creator, size = 48 }: { creator: CreatorProfile; size?: number }) {
  const [imgFailed, setImgFailed] = useState(false);

  return (
    <span
      className={`relative inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-gradient-to-br ${creator.gradient} p-[2px] shadow-md`}
      style={{ width: size, height: size }}
    >
      <span className="relative flex h-full w-full items-center justify-center overflow-hidden rounded-full bg-[#0b1211]">
        {!imgFailed ? (
          <img
            src={creator.avatarUrl}
            alt={creator.name}
            onError={() => setImgFailed(true)}
            className="h-full w-full object-cover"
          />
        ) : creator.kind === "lilmmd" ? (
          <svg viewBox="0 0 64 64" className="h-full w-full">
            <defs>
              <linearGradient id="lilmmd-bg" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stopColor="#071914" />
                <stop offset="100%" stopColor="#0f2922" />
              </linearGradient>
            </defs>
            <rect width="64" height="64" fill="url(#lilmmd-bg)" />
            <circle cx="32" cy="24" r="11" fill="#00e599" opacity="0.95" />
            <path d="M14 56c2-11 10-17 18-17s16 6 18 17" fill="#00e599" opacity="0.4" />
            <path d="M18 21h28l-2.5 6.5H20.5z" fill="#0b1211" />
            <rect x="22.5" y="23" width="7.5" height="2.8" rx="1.4" fill="#00e599" />
            <rect x="34" y="23" width="7.5" height="2.8" rx="1.4" fill="#00e599" />
          </svg>
        ) : (
          <svg viewBox="0 0 64 64" className="h-full w-full">
            <defs>
              <linearGradient id="lfn-bg" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stopColor="#091526" />
                <stop offset="100%" stopColor="#0d233a" />
              </linearGradient>
            </defs>
            <rect width="64" height="64" fill="url(#lfn-bg)" />
            <polygon points="32,8 48,18 48,38 32,48 16,38 16,18" fill="#38bdf8" opacity="0.2" stroke="#38bdf8" strokeWidth="2" />
            <circle cx="32" cy="25" r="10" fill="#38bdf8" opacity="0.95" />
            <path d="M15 56c2.5-10.5 9.5-16 17-16s14.5 5.5 17 16" fill="#00e599" opacity="0.5" />
            <path d="M23 24h18" stroke="#091526" strokeWidth="3.2" strokeLinecap="round" />
          </svg>
        )}
      </span>
    </span>
  );
}

export function AboutCard() {
  const { t } = useApp();
  return (
    <section className="surface-dark relative overflow-hidden rounded-[32px] bg-hero p-6 text-fg">
      <div className="relative flex flex-wrap items-center gap-5">
        <span className="grid h-[76px] w-[76px] place-items-center rounded-[24px] bg-rail border border-white/8">
          <Logo size={46} />
        </span>
        <div className="min-w-0 flex-1">
          <div className="text-[12px] font-bold uppercase tracking-widest text-mint">{t("aboutTitle")}</div>
          <h2 className="m-0 mt-1 text-[28px] font-bold leading-tight">
            {t("appName")}
          </h2>
          <div className="mt-1 text-[14px] text-fg2">
            {t("aboutBy")} <span className="font-bold text-fg">WitWizard</span>
          </div>
        </div>
        <span className="mono rounded-full border border-white/12 px-3.5 py-1.5 text-[12px] font-semibold text-mint">
          v{APP_VERSION}
        </span>
      </div>

      <p className="relative m-0 mt-4 text-[13.5px] leading-relaxed text-fg2">{t("aboutBody")}</p>

      <div className="relative mt-5">
        <div className="mb-3 text-[11.5px] font-bold uppercase tracking-wider text-fg2">{t("aboutCreators")}</div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {CREATORS.map((creator) => (
            <div
              key={creator.name}
              className="flex items-center gap-3.5 rounded-2xl border border-white/10 bg-[#151f1d] p-3 transition-colors hover:border-mint/40"
            >
              <CreatorAvatar creator={creator} size={50} />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                  <span className="truncate text-[15px] font-bold text-fg">{creator.name}</span>
                  {creator.kind === "lilmmd" ? (
                    <Terminal size={13} className="shrink-0 text-mint" />
                  ) : (
                    <ShieldCheck size={14} className="shrink-0 text-[#38bdf8]" />
                  )}
                </div>
                <div className="mt-0.5 truncate text-[12px] text-fg2">{t(creator.roleKey)}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}