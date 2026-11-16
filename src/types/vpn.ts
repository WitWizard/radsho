export type ConnState = "OFF" | "PREPARING" | "CONNECTING" | "VERIFYING" | "PAUSED" | "RUNNING" | "ERROR";
export type TransportId = "MASQUE_H3" | "MASQUE_H2" | "WIREGUARD" | "GOOL" | "GOOL_CLASSIC" | "MIM";
export type ScanMode = "DEFAULT" | "TURBO" | "BALANCED" | "THOROUGH" | "VERIFIED" | "IRONCLAD";
export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";
export type Lang = "en" | "fa";
export type ViewId = "dashboard" | "locations" | "logs" | "speed" | "settings";
export type ServerCategory = "fast" | "streaming" | "gaming" | "antisanction" | "doublehop";
export type PresetId = "unblock" | "resilient" | "fastest" | "antidpi" | "lowdata";
export type SortMode = "none" | "az" | "za" | "fav";
export type LogLevel = "info" | "ok" | "warn" | "error" | "core";
export type ModalId = "shuffle" | "paths" | "protocol" | "scanner" | "check" | "speed" | "updater" | null;

export interface Server {
  id: string;
  city: string;
  cityFa: string;
  tag: string;
  region: string;
  flag: string;
  ping: number;
  colo: string;
  tags: ServerCategory[];
}

export interface Ports {
  bind: number;
  http: number;
  psiphon: number;
  psiphonHttp: number;
}

export interface ShuffleSettings {
  enabled: boolean;
  intervalMin: number;
  regions: string[];
}

export interface RoutingSettings {
  enabled: boolean;
  direct: string[];
  block: string[];
  domesticBypass: boolean;
  apps: string[];
}

export interface ProtocolSettings {
  auto: boolean;
  preset: PresetId;
  chain: TransportId[];
  fragment: boolean;
}

export interface Settings {
  theme: ThemeMode;
  lang: Lang;
  transport: TransportId;
  scanMode: ScanMode;
  ports: Ports;
  dns: string;
  extraArgs: string;
  systemProxy: boolean;
  minimizeToTray: boolean;
  autoUpdate: boolean;
  psiphon: boolean;
  psiphonRegion: string;
  sanctionCheck: boolean;
  shuffle: ShuffleSettings;
  routing: RoutingSettings;
  protocol: ProtocolSettings;
  rememberedTransport: TransportId | null;
  serverId: string;
}

export interface Egress {
  ip: string;
  loc: string;
  colo: string;
  warp: string;
}

export interface SessionSnapshot {
  state: ConnState;
  gen: number;
  serverId: string;
  region: string;
  transport: TransportId | null;
  egress: Egress | null;
  uptimeSecs: number;
  rxBps: number;
  txBps: number;
  message: string;
}

export interface LogEntry {
  id: number;
  ts: number;
  level: LogLevel;
  text: string;
}

export interface UpdateInfo {
  appCurrent: string;
  appLatest: string;
  coreCurrent: string;
  coreLatest: string;
  changelog: string[];
  mirror: string;
  sha256: string;
}

export interface PresetDefinition {
  chain: TransportId[];
  scanMode: ScanMode;
  fragment: boolean;
  psiphon?: boolean;
  sanctionCheck?: boolean;
}

export interface Engine {
  subscribe(onSnapshot: (snapshot: SessionSnapshot) => void, onLog: (level: LogLevel, text: string) => void): () => void;
  connect(settings: Settings, serverId: string, region: string): void;
  disconnect(): void;
  pause(): void;
  resume(): void;
  configure(settings: Settings): void;
  snapshot(): SessionSnapshot;
}