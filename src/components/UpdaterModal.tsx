import { useState } from "react";
import { BadgeCheck, Download, Loader2, RotateCw } from "lucide-react";
import { useApp } from "../state/AppContext";
import { APP_VERSION, CORE_VERSION } from "../lib/config";
import { call, sleep } from "../lib/bridge";
import { Button, Modal } from "./ui";

type Phase = "idle" | "checking" | "ready" | "downloading" | "verifying" | "verified" | "installing" | "current";

export function UpdaterBody() {
  const { t, updateInfo, checkForUpdates, completeUpdate, setModal } = useApp();
  const [phase, setPhase] = useState<Phase>(updateInfo ? "ready" : "idle");
  const [progress, setProgress] = useState(0);

  const check = async () => {
    setPhase("checking");
    const info = await checkForUpdates();
    setPhase(info ? "ready" : "current");
  };

  const download = async () => {
    setPhase("downloading");
    setProgress(0);
    await call<void>("updater_download", {}, async () => {
      for (let value = 0; value <= 100; value += 4) {
        setProgress(value);
        await sleep(70);
      }
    });
    setProgress(100);
    setPhase("verifying");
    await sleep(1200);
    setPhase("verified");
  };

  const install = async () => {
    setPhase("installing");
    await call<void>("updater_install", {}, () => sleep(1800));
    completeUpdate();
    setPhase("current");
    setModal(null);
  };

  const rows = [
    { label: t("updaterApp"), current: APP_VERSION, latest: updateInfo?.appLatest ?? APP_VERSION },
    { label: t("updaterCore"), current: CORE_VERSION, latest: updateInfo?.coreLatest ?? CORE_VERSION },
  ];

  return (
    <div>
      <div className="grid gap-3 sm:grid-cols-2">
        {rows.map((row) => (
          <div key={row.label} className="rounded-2xl border border-white/8 bg-white/4 p-4">
            <div className="text-[13px] font-semibold text-fg2">{row.label}</div>
            <div className="mono mt-2 flex items-center justify-between text-[14px]">
              <span className="text-fg2">
                {t("updaterCurrent")} v{row.current}
              </span>
              <span className={row.latest !== row.current ? "font-bold text-mint" : "text-fg2"}>
                {t("updaterLatest")} v{row.latest}
              </span>
            </div>
          </div>
        ))}
      </div>

      {phase === "current" && (
        <p className="m-0 mt-4 flex items-center gap-2 text-[14px] font-semibold text-mint">
          <BadgeCheck size={18} />
          {t("updaterUpToDate")}
        </p>
      )}

      {updateInfo && phase !== "current" && (
        <div className="mt-4">
          <div className="mb-2 text-[13px] font-semibold text-fg2">{t("updaterChangelog")}</div>
          <ul className="m-0 flex list-none flex-col gap-1.5 p-0 text-[14px]">
            {updateInfo.changelog.map((line) => (
              <li key={line} className="flex gap-2">
                <span className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-mint" />
                {line}
              </li>
            ))}
          </ul>
          <div className="mono mt-3 break-all text-[11px] text-fg2">
            {t("updaterMirror")}: {updateInfo.mirror} · sha256:{updateInfo.sha256}
          </div>
        </div>
      )}

      {(phase === "downloading" || phase === "verifying" || phase === "verified" || phase === "installing") && (
        <div className="mt-5">
          <div className="h-2.5 overflow-hidden rounded-full bg-white/10">
            <div
              className={phase === "downloading" || phase === "installing" ? "bar-shimmer h-full rounded-full" : "h-full rounded-full bg-mint"}
              style={{ width: `${phase === "installing" ? 100 : progress}%`, transition: "width 0.15s linear" }}
            />
          </div>
          <div className="mono mt-2 flex items-center justify-between text-[12px] text-fg2">
            <span>
              {phase === "downloading" && t("updaterDownloading")}
              {phase === "verifying" && t("updaterVerifying")}
              {phase === "verified" && t("updaterVerified")}
              {phase === "installing" && t("updaterInstalling")}
            </span>
            <span>{progress}%</span>
          </div>
          {phase === "verified" && (
            <span className="mt-3 inline-flex items-center gap-2 rounded-full border border-mint/40 bg-mint/12 px-3 py-1 text-[12px] font-bold text-mint">
              <BadgeCheck size={14} />
              SHA-256 · Ed25519
            </span>
          )}
        </div>
      )}

      <div className="mt-6 flex justify-end gap-3">
        {(phase === "idle" || phase === "current") && (
          <Button variant="outline" onClick={() => void check()}>
            <RotateCw size={16} />
            {t("updaterCheck")}
          </Button>
        )}
        {phase === "checking" && (
          <Button variant="outline" disabled>
            <Loader2 size={16} className="spin-slow" />
            {t("updaterChecking")}
          </Button>
        )}
        {phase === "ready" && (
          <Button onClick={() => void download()}>
            <Download size={16} />
            {t("updaterDownload")}
          </Button>
        )}
        {(phase === "downloading" || phase === "verifying") && (
          <Button disabled>
            <Loader2 size={16} className="spin-slow" />
            {phase === "downloading" ? t("updaterDownloading") : t("updaterVerifying")}
          </Button>
        )}
        {phase === "verified" && <Button onClick={() => void install()}>{t("updaterInstall")}</Button>}
        {phase === "installing" && (
          <Button disabled>
            <Loader2 size={16} className="spin-slow" />
            {t("updaterInstall")}
          </Button>
        )}
      </div>
    </div>
  );
}

export function UpdaterModal() {
  const { t, modal, setModal } = useApp();
  return (
    <Modal open={modal === "updater"} title={t("updaterTitle")} onClose={() => setModal(null)} wide>
      <UpdaterBody />
    </Modal>
  );
}
