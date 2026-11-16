import { Gauge, House, Settings, ShieldCheck, Terminal, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { ViewId } from "../types/vpn";
import type { TKey } from "../i18n/translations";
import { Logo } from "./Logo";

interface NavItem {
  id: ViewId;
  icon: LucideIcon;
  label: TKey;
}

const MAIN_ITEMS: NavItem[] = [
  { id: "dashboard", icon: House, label: "navDashboard" },
  { id: "locations", icon: ShieldCheck, label: "navLocations" },
  { id: "logs", icon: Terminal, label: "navLogs" },
  { id: "speed", icon: Gauge, label: "navSpeed" },
];

const FOOTER_ITEMS: NavItem[] = [
  { id: "settings", icon: Settings, label: "navSettings" },
];

const MOBILE_ITEMS: NavItem[] = [...MAIN_ITEMS, { id: "settings", icon: Settings, label: "navSettings" }];

interface NavButtonProps {
  item: NavItem;
  size: number;
}

function NavButton({ item, size }: NavButtonProps) {
  const { view, setView, t, snapshot } = useApp();
  const Icon = item.icon;
  const active = view === item.id;
  const live = item.id === "dashboard" && snapshot.state === "RUNNING";
  return (
    <button
      type="button"
      title={t(item.label)}
      aria-label={t(item.label)}
      aria-current={active ? "page" : undefined}
      onClick={() => setView(item.id)}
      className={cn(
        "relative grid shrink-0 place-items-center rounded-full border-0 transition-all",
        active ? "bg-mint text-[#04231a]" : "bg-transparent text-[#8a9a96] hover:bg-white/8 hover:text-white",
      )}
      style={{ width: size, height: size }}
    >
      <Icon size={size * 0.42} strokeWidth={active ? 2.4 : 2} className={active ? "fill-[#04231a]/20" : ""} />
      {(live) && (
        <span className="absolute end-[2px] top-[2px] h-3 w-3 rounded-full border-2 border-[#111817] bg-[#7cff00]" />
      )}
    </button>
  );
}

export function Sidebar() {
  return (
    <nav className="flex w-[88px] shrink-0 flex-col items-center rounded-[32px] bg-rail py-7">
      <Logo size={44} />
      <div className="mt-16 flex flex-col items-center gap-5">
        {MAIN_ITEMS.map((item) => (
          <NavButton key={item.id} item={item} size={64} />
        ))}
      </div>
      <div className="mt-auto flex flex-col items-center gap-5">
        {FOOTER_ITEMS.map((item) => (
          <NavButton key={item.id} item={item} size={56} />
        ))}
      </div>
    </nav>
  );
}

export function MobileNav() {
  return (
    <nav className="absolute inset-x-0 bottom-0 z-30 rounded-t-[36px] bg-hero px-5 pb-3 pt-4 shadow-[0_-12px_40px_rgba(0,0,0,0.25)]">
      <div className="flex items-center justify-between">
        {MOBILE_ITEMS.map((item) => (
          <NavButton key={item.id} item={item} size={62} />
        ))}
      </div>
    </nav>
  );
}