import { useCallback, useEffect, useRef, useState } from "react";
import { ArrowDown, ArrowUp, Check, Loader2, Minus, Play, RotateCw, TriangleAlert } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import {
  checkSeed,
  runCheck,
  runScanner,
  runSpeedTest,
  scannerSeed,
  type DiagRow,
  type RowStatus,
  type SpeedPhase,
  type SpeedResult,
} from "../lib/diagnostics";
import { Button, Modal } from "./ui";

function StatusIcon({ status }: { status: RowStatus }) {
  if (status === "running") {
    return <Loader2 size={20} className="spin-slow text-amber-fast" />;
  }
  if (status === "ok") {
    return (
      <span className="grid h-6 w-6 place-items-center rounded-full bg-mint text-[#04231a]">
        <Check size={14} strokeWidth={3} />
      </span>
    );
  }
  if (status === "issue") {
    return <TriangleAlert size={20} className="text-red-400" />;
  }
  if (status === "idle") {
    return (
      <span className="grid h-6 w-6 place-items-center rounded-full bg-white/10 text-fg2">
        <Minus size={14} />
      </span>
    );
  }
  return <span className="h-6 w-6 rounded-full border-2 border-white/15" />;
}

type DiagnosticKind = "scanner" | "check";

function DiagnosticPanel({ kind }: { kind: DiagnosticKind }) {
  const { t, settings, snapshot } = useApp();
  const [rows, setRows] = useState<DiagRow[]>(kind === "scanner" ? scannerSeed() : checkSeed());
  const [running, setRunning] = useState(false);
  const mounted = useRef(true);
  const latest = useRef({ settings, snapshot });
  latest.current = { settings, snapshot };

  const run = useCallback(async () => {
    setRunning(true);
    const sink = (next: DiagRow[]) => {
      if (mounted.current) {
        setRows(next);
      }
    };
    const seed = kind === "scanner" ? scannerSeed() : checkSeed();
    const { settings: currentSettings, snapshot: currentSnapshot } = latest.current;
    if (kind === "scanner") {
      await runScanner(seed, currentSettings, currentSnapshot, t, sink);
    } else {
      await runCheck(seed, currentSettings, currentSnapshot, t, sink);
    }
    if (mounted.current) {
      setRunning(false);
    }
  }, [kind, t]);

  useEffect(() => {
    mounted.current = true;
    void run();
    return () => {
      mounted.current = false;
    };
  }, [run]);

  return (
    <div>
      <ul className="m-0 flex list-none flex-col gap-2 p-0">
        {rows.map((row) => (
          <li key={row.id} className="flex items-start gap-3 rounded-2xl border border-white/8 bg-white/4 px-4 py-3">
            <span className="mt-0.5">
              <StatusIcon status={row.status} />
            </span>
            <div className="min-w-0 flex-1">
              <div className="text-[15px] font-semibold">{t(row.label)}</div>
              <div className="mono mt-0.5 break-words text-[12px] leading-relaxed text-fg2">{row.detail || "…"}</div>
            </div>
          </li>
        ))}
      </ul>
      <div className="mt-4 flex justify-end">
        <Button variant="outline" disabled={running} onClick={() => void run()}>
          {running ? <Loader2 size={16} className="spin-slow" /> : <RotateCw size={16} />}
          {running ? t("scanning") : kind === "scanner" ? t("scanNow") : t("checkRun")}
        </Button>
      </div>
    </div>
  );
}

const SPEED_HISTORY_KEY = "radsho.speed.v1";

function loadHistory(): SpeedResult[] {
  try {
    const raw = window.localStorage.getItem(SPEED_HISTORY_KEY);
    return raw ? (JSON.parse(raw) as SpeedResult[]) : [];
  } catch {
    return [];
  }
}

export function SpeedPanel() {
  const { t, snapshot } = useApp();
  const viaTunnel = snapshot.state === "RUNNING";
  const [phase, setPhase] = useState<SpeedPhase>("idle");
  const [result, setResult] = useState<SpeedResult>({ ping: 0, jitter: 0, down: 0, up: 0 });
  const [live, setLive] = useState(0);
  const [history, setHistory] = useState<SpeedResult[]>(loadHistory);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const start = async () => {
    setPhase("ping");
    setLive(0);
    setResult({ ping: 0, jitter: 0, down: 0, up: 0 });
    const final = await runSpeedTest(viaTunnel, (progress) => {
      if (!mounted.current) {
        return;
      }
      setPhase(progress.phase);
      setLive(progress.live);
      setResult({ ping: progress.ping, jitter: progress.jitter, down: progress.down, up: progress.up });
    });
    const next = [final, ...history].slice(0, 5);
    window.localStorage.setItem(SPEED_HISTORY_KEY, JSON.stringify(next));
    if (mounted.current) {
      setHistory(next);
    }
  };

  const testing = phase === "ping" || phase === "download" || phase === "upload";
  const gaugeMax = 120;
  const gaugeValue = phase === "ping" ? Math.min(gaugeMax, live / 3) : phase === "done" ? result.down : live;
  const display = phase === "ping" ? result.ping : phase === "done" ? result.down : live;
  const circumference = Math.PI * 80;
  const progress = Math.min(1, Math.max(0, gaugeValue / gaugeMax));
  const phaseColor = phase === "upload" ? "#f59e0b" : phase === "ping" ? "#38bdf8" : "#00e599";

  const stageText =
    phase === "ping"
      ? t("speedStagePing")
      : phase === "download"
        ? t("speedStageDown")
        : phase === "upload"
          ? t("speedStageUp")
          : viaTunnel
            ? t("speedViaTunnel")
            : t("speedDirect");

  const tiles = [
    {
      label: t("speedDownload"),
      value: result.down.toFixed(1),
      unit: t("speedUnit"),
      icon: ArrowDown,
      color: "text-mint",
      active: phase === "download",
    },
    {
      label: t("speedUpload"),
      value: result.up.toFixed(1),
      unit: t("speedUnit"),
      icon: ArrowUp,
      color: "text-amber-fast",
      active: phase === "upload",
    },
    {
      label: t("speedPing"),
      value: String(result.ping),
      unit: t("msUnit"),
      icon: null,
      color: "text-fg",
      active: phase === "ping",
    },
    {
      label: t("speedJitter"),
      value: result.jitter.toFixed(1),
      unit: t("msUnit"),
      icon: null,
      color: "text-fg",
      active: phase === "ping",
    },
  ];

  return (
    <div>
      <div className="relative mx-auto h-[145px] w-[240px]">
        <svg viewBox="0 0 200 115" className="h-full w-full">
          <path
            d="M20 100 A80 80 0 0 1 180 100"
            fill="none"
            stroke="rgba(148,163,184,0.18)"
            strokeWidth="13"
            strokeLinecap="round"
          />
          <path
            d="M20 100 A80 80 0 0 1 180 100"
            fill="none"
            stroke={phaseColor}
            strokeWidth="13"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={circumference * (1 - progress)}
            style={{ transition: "stroke-dashoffset 0.14s linear, stroke 0.25s ease" }}
          />
        </svg>
        <div className="absolute inset-x-0 bottom-1 text-center">
          <div className="mono text-[38px] font-extrabold leading-none tracking-tight">
            {display.toFixed(phase === "ping" ? 0 : 1)}
          </div>
          <div className="mt-1.5 text-[12px] font-semibold uppercase tracking-wider text-fg2">
            {phase === "ping" ? t("msUnit") : t("speedUnit")}
          </div>
        </div>
      </div>

      <p className="m-0 mt-2 text-center text-[13px] font-medium text-fg2">{stageText}</p>

      <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
        {tiles.map((tile) => (
          <div
            key={tile.label}
            className={cn(
              "rounded-2xl border bg-chip px-4 py-3 transition-all",
              tile.active ? "border-mint shadow-[0_0_0_3px_rgba(0,229,153,0.12)]" : "border-line",
            )}
          >
            <div className="flex items-center gap-1 text-[12px] font-semibold text-fg2">
              {tile.icon && <tile.icon size={12} className={tile.color} />}
              {tile.label}
            </div>
            <div className={cn("mono mt-1 text-[22px] font-bold", tile.color)}>{tile.value}</div>
            <div className="text-[11px] text-fg2">{tile.unit}</div>
          </div>
        ))}
      </div>

      <div className="mt-5 flex justify-center">
        <Button disabled={testing} onClick={() => void start()} className="min-w-[190px]">
          {testing ? <Loader2 size={16} className="spin-slow" /> : <Play size={16} className="fill-current" />}
          {testing ? t("speedTesting") : phase === "done" ? t("runAgain") : t("speedStart")}
        </Button>
      </div>

      {history.length > 0 && (
        <div className="mt-6">
          <div className="mb-2 text-[13px] font-semibold text-fg2">{t("speedHistory")}</div>
          <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
            {history.map((entry, index) => (
              <li key={index} className="mono flex items-center justify-between rounded-xl bg-chip px-3.5 py-2 text-[12px] text-fg2">
                <span className="font-semibold text-mint">↓ {entry.down.toFixed(1)} Mbps</span>
                <span className="font-semibold text-amber-fast">↑ {entry.up.toFixed(1)} Mbps</span>
                <span>{entry.ping} ms</span>
                <span>± {entry.jitter.toFixed(1)} ms</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

export function AdvancedModals() {
  const { t, modal, setModal } = useApp();
  const close = () => setModal(null);
  return (
    <>
      <Modal open={modal === "scanner"} title={t("scannerTitle")} onClose={close} wide>
        <DiagnosticPanel kind="scanner" />
      </Modal>
      <Modal open={modal === "check"} title={t("checkTitle")} onClose={close} wide>
        <DiagnosticPanel kind="check" />
      </Modal>
      <Modal open={modal === "speed"} title={t("speedTitle")} onClose={close} wide>
        <SpeedPanel />
      </Modal>
    </>
  );
}