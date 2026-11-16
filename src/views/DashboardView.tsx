import { Bell, Bolt, Menu } from "lucide-react";
import { useApp } from "../state/AppContext";
import { ConnectionHub } from "../components/ConnectionHub";
import { OptionCards } from "../components/OptionCards";
import { AdvancedCards } from "../components/AdvancedCards";
import { Logo } from "../components/Logo";

export function DashboardView() {
  const { t, isMobile, setView, setOptionsOpen, updateInfo, setModal, notify } = useApp();

  if (!isMobile) {
    return (
      <div className="scroll-y h-full pe-3">
        <h1 className="m-0 mb-5 text-[40px] font-semibold tracking-tight text-fg">{t("vpnConnection")}</h1>
        <section className="surface-dark grid grid-cols-1 gap-5 rounded-[36px] bg-hero p-5 text-fg lg:grid-cols-2">
          <div className="p-4">
            <ConnectionHub />
          </div>
          <OptionCards variant="inline" />
        </section>
        <h2 className="m-0 mb-5 mt-9 text-[40px] font-semibold tracking-tight text-fg">{t("advancedOptions")}</h2>
        <div className="pb-6">
          <AdvancedCards />
        </div>
      </div>
    );
  }

  return (
    <div
      className="scroll-y h-full"
      style={{
        paddingBottom: "calc(max(env(safe-area-inset-bottom, 0px), var(--android-inset-bottom, 12px)) + 112px)",
      }}
    >
      <section className="surface-dark rounded-b-[40px] bg-hero px-5 pb-5 pt-5 text-fg">
        <div className="mb-6 flex items-center justify-between">
          <button
            type="button"
            aria-label={t("navSettings")}
            onClick={() => setView("settings")}
            className="grid h-[52px] w-[52px] place-items-center rounded-full border-0 bg-white/10 text-fg"
          >
            <Menu size={22} />
          </button>
          <Logo size={40} />
          <button
            type="button"
            aria-label={t("notifications")}
            onClick={() => (updateInfo ? setModal("updater") : notify(t("updaterUpToDate")))}
            className="relative grid h-[52px] w-[52px] place-items-center rounded-full border-0 bg-white/10 text-fg"
          >
            <Bell size={22} className="fill-current" />
            {updateInfo && <span className="absolute end-3 top-3 h-2.5 w-2.5 rounded-full bg-mint" />}
          </button>
        </div>
        <ConnectionHub />
        <button
          type="button"
          onClick={() => setOptionsOpen(true)}
          className="mx-auto mt-5 flex items-center gap-2 border-0 bg-transparent text-[16px] font-medium text-fg2"
        >
          <Bolt size={22} className="fill-fg2 text-fg2" />
          {t("moreVpnOptions")}
        </button>
      </section>
      <div className="px-5">
        <h2 className="m-0 mb-4 mt-9 text-center text-[36px] font-semibold tracking-tight text-fg">{t("advancedOptions")}</h2>
        <AdvancedCards />
      </div>
    </div>
  );
}