import type { PresetDefinition, PresetId, ScanMode, Settings, TransportId } from "../types/vpn";

export const APP_VERSION = "1.0.0";
export const CORE_VERSION = "2.4.0";
export const SANCTIONED_LOCS: readonly string[] = ["IR", "KP", "SY", "CU"];
export const SANCTION_SITES: readonly string[] = ["www.kimi.ai", "claude.ai", "huggingface.co"];
export const TRACE_TARGETS: readonly string[] = ["www.cloudflare.com:443", "cloudflare.com:443", "1.1.1.1:443"];

export const SHUFFLE_REGIONS: readonly string[] = [
  "DE",
  "NL",
  "IT",
  "FR",
  "ES",
  "GB",
  "AT",
  "CH",
  "SE",
  "NO",
  "FI",
  "DK",
  "IS",
  "IE",
  "BE",
  "PT",
  "PL",
  "CZ",
  "SK",
  "HU",
  "RO",
  "BG",
  "GR",
  "HR",
  "SI",
  "RS",
  "EE",
  "LV",
  "LT",
  "UA",
  "MD",
  "TR",
  "AE",
  "US",
  "CA",
  "BR",
  "JM",
  "SG",
  "JP",
  "KR",
  "HK",
  "TW",
  "IN",
  "AU",
  "NZ",
  "ZA",
];

export const SCAN_MODES: readonly ScanMode[] = ["DEFAULT", "TURBO", "BALANCED", "THOROUGH", "VERIFIED", "IRONCLAD"];
export const ALL_TRANSPORTS: readonly TransportId[] = ["MASQUE_H3", "MASQUE_H2", "WIREGUARD", "GOOL", "GOOL_CLASSIC", "MIM"];
export const NESTED_TRANSPORTS: readonly TransportId[] = ["GOOL", "GOOL_CLASSIC", "MIM"];
export const SHUFFLE_INTERVALS: readonly number[] = [1, 5, 10, 15, 30, 60];
export const SPLIT_APPS: readonly string[] = ["Telegram", "Chrome", "Firefox", "Spotify", "Steam", "WhatsApp", "Discord", "VS Code"];

export const TRANSPORT_ARGS: Record<TransportId, readonly string[]> = {
  MASQUE_H3: ["--masque"],
  MASQUE_H2: ["--masque", "--h2"],
  WIREGUARD: ["--wg"],
  GOOL: ["--gool"],
  GOOL_CLASSIC: ["--gool-classic"],
  MIM: ["--mim"],
};

export const PRESETS: Record<PresetId, PresetDefinition> = {
  unblock: {
    chain: ["MASQUE_H3", "MASQUE_H2", "WIREGUARD", "GOOL", "GOOL_CLASSIC", "MIM"],
    scanMode: "VERIFIED",
    fragment: false,
    psiphon: true,
    sanctionCheck: true,
  },
  resilient: {
    chain: ["MASQUE_H2", "MASQUE_H3", "WIREGUARD", "GOOL", "MIM", "GOOL_CLASSIC"],
    scanMode: "IRONCLAD",
    fragment: false,
  },
  fastest: {
    chain: ["MASQUE_H3", "WIREGUARD", "MASQUE_H2", "GOOL", "MIM", "GOOL_CLASSIC"],
    scanMode: "TURBO",
    fragment: false,
  },
  antidpi: {
    chain: ["MASQUE_H2", "GOOL", "MIM", "GOOL_CLASSIC", "MASQUE_H3", "WIREGUARD"],
    scanMode: "BALANCED",
    fragment: true,
  },
  lowdata: {
    chain: ["WIREGUARD", "MASQUE_H3", "MASQUE_H2", "GOOL", "GOOL_CLASSIC", "MIM"],
    scanMode: "DEFAULT",
    fragment: false,
    sanctionCheck: false,
  },
};

export const DEFAULT_SETTINGS: Settings = {
  theme: "system",
  lang: "en",
  transport: "MASQUE_H3",
  scanMode: "VERIFIED",
  ports: { bind: 1819, http: 1820, psiphon: 1821, psiphonHttp: 1822 },
  dns: "1.1.1.1",
  extraArgs: "",
  systemProxy: true,
  minimizeToTray: true,
  autoUpdate: true,
  psiphon: true,
  psiphonRegion: "DE",
  sanctionCheck: true,
  shuffle: { enabled: false, intervalMin: 10, regions: ["DE", "NL", "FR", "AT", "IT", "ES", "CH", "SE", "GB", "US"] },
  routing: { enabled: true, direct: [], block: [], domesticBypass: true, apps: [] },
  protocol: { auto: true, preset: "unblock", chain: [...PRESETS.unblock.chain], fragment: false },
  rememberedTransport: null,
  serverId: "frankfurt",
};

export function buildChain(settings: Settings): TransportId[] {
  const base = settings.protocol.auto ? settings.protocol.chain : [settings.transport];
  const remembered = settings.protocol.auto && settings.rememberedTransport ? [settings.rememberedTransport] : [];
  const merged = Array.from(new Set<TransportId>([...remembered, ...base]));
  if (settings.psiphon) {
    return merged;
  }
  const nested = merged.filter((id) => NESTED_TRANSPORTS.includes(id));
  const plain = merged.filter((id) => !NESTED_TRANSPORTS.includes(id));
  return [...nested, ...plain];
}

export function effectivePorts(settings: Settings): { socks: number; http: number } {
  return settings.psiphon
    ? { socks: settings.ports.psiphon, http: settings.ports.psiphonHttp }
    : { socks: settings.ports.bind, http: settings.ports.http };
}

export function buildCoreArgs(settings: Settings, transport: TransportId, region: string): string[] {
  const args: string[] = [
    "--bind",
    `127.0.0.1:${settings.ports.bind}`,
    "--config",
    "aether.toml",
    "--log-level",
    "info",
    "--stats",
    "--stats-secs",
    "1",
    ...TRANSPORT_ARGS[transport],
  ];
  if (settings.dns.trim()) {
    args.push("--dns", settings.dns.trim());
  }
  args.push("--ip", "v4");
  if (!settings.psiphon) {
    args.push("--exit-loc", "!IR");
  }
  if (settings.routing.enabled) {
    if (settings.routing.domesticBypass) {
      args.push("--route-direct", ".ir");
    }
    settings.routing.direct.forEach((domain) => args.push("--route-direct", domain));
    settings.routing.block.forEach((domain) => args.push("--route-block", domain));
  }
  args.push("--reconnect-secs", "2", "--quick-reconnect", "--keepalive", "5");
  if (settings.protocol.fragment) {
    args.push("--fragment");
  }
  if (settings.scanMode !== "DEFAULT") {
    args.push("--scan", settings.scanMode.toLowerCase());
  }
  if (settings.psiphon) {
    args.push(
      "--psiphon",
      "--psiphon-bind",
      `127.0.0.1:${settings.ports.psiphon}`,
      "--psiphon-http",
      `127.0.0.1:${settings.ports.psiphonHttp}`,
      "--psiphon-region",
      region,
    );
  }
  args.push("--http-proxy", `127.0.0.1:${settings.ports.http}`);
  const extra = settings.extraArgs.trim();
  if (extra) {
    args.push(...extra.split(/\s+/));
  }
  return args;
}

export function portsValid(settings: Settings): boolean {
  const values = [settings.ports.bind, settings.ports.http, settings.ports.psiphon, settings.ports.psiphonHttp];
  const inRange = values.every((value) => Number.isInteger(value) && value >= 1024 && value <= 65535);
  return inRange && new Set(values).size === values.length;
}

export function formatDuration(totalSecs: number): string {
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs / 3600) % 60);
  const s = totalSecs % 60;
  return [h, m, s].map((part) => String(part).padStart(2, "0")).join(":");
}

export function formatRate(bytesPerSec: number): string {
  if (bytesPerSec >= 1024 * 1024) {
    return `${(bytesPerSec / 1024 / 1024).toFixed(1)} MB/s`;
  }
  return `${Math.max(0, Math.round(bytesPerSec / 1024))} KB/s`;
}