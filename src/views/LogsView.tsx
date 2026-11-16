import { useEffect, useMemo, useRef, useState } from "react";
import { Copy, Trash2 } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { LogLevel } from "../types/vpn";
import { Page } from "../components/Page";
import { Button, Segmented } from "../components/ui";

type Filter = "all" | LogLevel;

const LEVEL_STYLE: Record<LogLevel, string> = {
  info: "text-sky-300",
  ok: "text-mint",
  warn: "text-amber-fast",
  error: "text-red-400",
  core: "text-fuchsia-300",
};

function stamp(ts: number): string {
  const date = new Date(ts);
  return [date.getHours(), date.getMinutes(), date.getSeconds()].map((part) => String(part).padStart(2, "0")).join(":");
}

export function LogsView() {
  const { t, logs, clearLogs, lastArgv, notify } = useApp();
  const [filter, setFilter] = useState<Filter>("all");
  const endRef = useRef<HTMLDivElement | null>(null);
  const visible = useMemo(() => (filter === "all" ? logs : logs.filter((entry) => entry.level === filter)), [logs, filter]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ block: "end" });
  }, [visible.length]);

  const copy = async () => {
    const text = visible.map((entry) => `${stamp(entry.ts)} [${entry.level}] ${entry.text}`).join("\n");
    try {
      await navigator.clipboard.writeText(text);
      notify(t("copied"));
    } catch {
      notify(t("copied"));
    }
  };

  const options: Array<{ value: Filter; label: string }> = [
    { value: "all", label: t("logsAll") },
    { value: "info", label: "info" },
    { value: "ok", label: "ok" },
    { value: "warn", label: "warn" },
    { value: "error", label: "error" },
    { value: "core", label: "core" },
  ];

  return (
    <Page
      title={t("logsTitle")}
      fill
      actions={
        <>
          <Button variant="ghost" onClick={() => void copy()}>
            <Copy size={15} />
            {t("logsCopy")}
          </Button>
          <Button variant="ghost" onClick={clearLogs}>
            <Trash2 size={15} />
            {t("logsClear")}
          </Button>
        </>
      }
    >
      <div className="scroll-x mb-3 shrink-0">
        <Segmented value={filter} options={options} onChange={setFilter} small />
      </div>
      <div className="surface-dark scroll-y mono min-h-0 flex-1 rounded-[24px] border border-white/8 bg-[#0a0f0e] p-4 text-[12.5px] leading-[1.7] text-fg">
        {visible.length === 0 && <div className="py-8 text-center text-fg2">{t("logsEmpty")}</div>}
        {visible.map((entry) => (
          <div key={entry.id} className="flex gap-3 break-all">
            <span className="shrink-0 text-fg2">{stamp(entry.ts)}</span>
            <span className={cn("w-12 shrink-0 font-bold uppercase", LEVEL_STYLE[entry.level])}>{entry.level}</span>
            <span className={cn(entry.level === "core" && "text-fuchsia-200/90")}>{entry.text}</span>
          </div>
        ))}
        <div ref={endRef} />
      </div>
      {lastArgv && (
        <div className="mono mt-3 shrink-0 break-all rounded-2xl border border-line bg-card px-4 py-3 text-[12px] text-fg2">
          <span className="font-bold text-fg">{t("logsArgv")}: </span>
          {lastArgv}
        </div>
      )}
    </Page>
  );
}
