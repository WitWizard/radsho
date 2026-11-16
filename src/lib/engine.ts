import type { Egress, Engine, LogLevel, SessionSnapshot, Settings, TransportId } from "../types/vpn";
import { SANCTION_SITES, SANCTIONED_LOCS, TRACE_TARGETS, buildChain, buildCoreArgs, effectivePorts, NESTED_TRANSPORTS } from "./config";
import { call, isTauri, sleep } from "./bridge";
import { fakeIp, findServer, serverByRegion } from "./servers";

type SnapshotHandler = (snapshot: SessionSnapshot) => void;
type LogHandler = (level: LogLevel, text: string) => void;

interface Subscriber {
  onSnapshot: SnapshotHandler;
  onLog: LogHandler;
}

interface DrainedLog {
  level: LogLevel;
  text: string;
}

export function initialSnapshot(): SessionSnapshot {
  return {
    state: "OFF",
    gen: 0,
    serverId: "frankfurt",
    region: "DE",
    transport: null,
    egress: null,
    uptimeSecs: 0,
    rxBps: 0,
    txBps: 0,
    message: "",
  };
}

class SimulatedEngine implements Engine {
  private snap: SessionSnapshot = initialSnapshot();
  private settings: Settings | null = null;
  private subscribers = new Set<Subscriber>();
  private gen = 0;
  private ticker: number | null = null;
  private shuffleLeft = 0;

  subscribe(onSnapshot: SnapshotHandler, onLog: LogHandler): () => void {
    const subscriber: Subscriber = { onSnapshot, onLog };
    this.subscribers.add(subscriber);
    onSnapshot(this.snap);
    return () => {
      this.subscribers.delete(subscriber);
    };
  }

  snapshot(): SessionSnapshot {
    return this.snap;
  }

  configure(settings: Settings): void {
    this.settings = settings;
    if (!settings.shuffle.enabled) {
      this.shuffleLeft = settings.shuffle.intervalMin * 60;
    }
  }

  connect(settings: Settings, serverId: string, region: string): void {
    this.settings = settings;
    this.gen += 1;
    this.stopTicker();
    void this.run(this.gen, serverId, region);
  }

  disconnect(): void {
    this.gen += 1;
    this.stopTicker();
    const wasActive = this.snap.state !== "OFF";
    this.set({ state: "OFF", gen: this.gen, transport: null, egress: null, uptimeSecs: 0, rxBps: 0, txBps: 0, message: "" });
    if (wasActive) {
      this.log("info", "system proxy detached, aether core terminated (process group killed)");
    }
  }

  pause(): void {
    if (this.snap.state !== "RUNNING") {
      return;
    }
    this.set({ state: "PAUSED", rxBps: 0, txBps: 0 });
    this.log("warn", "paused: system proxy detached, core kept warm");
  }

  resume(): void {
    if (this.snap.state !== "PAUSED") {
      return;
    }
    this.set({ state: "RUNNING" });
    const ports = this.settings ? effectivePorts(this.settings) : { socks: 1821, http: 1822 };
    this.log("ok", `resumed: system proxy re-attached to 127.0.0.1:${ports.http}`);
  }

  private set(patch: Partial<SessionSnapshot>): void {
    this.snap = { ...this.snap, ...patch };
    this.subscribers.forEach((subscriber) => subscriber.onSnapshot(this.snap));
  }

  private log(level: LogLevel, text: string): void {
    this.subscribers.forEach((subscriber) => subscriber.onLog(level, text));
  }

  private alive(gen: number): boolean {
    return gen === this.gen;
  }

  private stopTicker(): void {
    if (this.ticker !== null) {
      window.clearInterval(this.ticker);
      this.ticker = null;
    }
  }

  private startTicker(): void {
    this.stopTicker();
    this.ticker = window.setInterval(() => this.tick(), 1000);
  }

  private trace(transport: TransportId, serverId: string, region: string): Egress {
    const server = findServer(serverId);
    const settings = this.settings;
    const carriesPsiphon = settings?.psiphon ?? true;
    const foreign = carriesPsiphon || NESTED_TRANSPORTS.includes(transport);
    if (foreign) {
      return { ip: fakeIp(`${serverId}-${region}`), loc: region, colo: server.colo, warp: carriesPsiphon ? "on" : "plus" };
    }
    return { ip: `104.28.${40 + (this.gen % 50)}.${12 + (this.gen % 200)}`, loc: "IR", colo: "IKA", warp: "on" };
  }

  private async run(gen: number, serverId: string, region: string): Promise<void> {
    const settings = this.settings;
    if (!settings) {
      return;
    }
    this.set({
      state: "PREPARING",
      gen,
      serverId,
      region,
      transport: null,
      egress: null,
      uptimeSecs: 0,
      rxBps: 0,
      txBps: 0,
      message: "",
    });
    const chain = buildChain(settings);
    const ports = effectivePorts(settings);
    this.log("info", `session #${gen} preparing, exit region ${region}`);
    this.log("info", `fallback chain: ${chain.join(" > ")}`);
    await sleep(450);
    if (!this.alive(gen)) {
      return;
    }
    for (const transport of chain) {
      const args = buildCoreArgs(settings, transport, region);
      this.set({ state: "CONNECTING", transport, message: transport });
      this.log("core", `aether ${args.join(" ")}`);
      await sleep(650);
      if (!this.alive(gen)) {
        return;
      }
      this.log("ok", `core SOCKS5 listener ready on 127.0.0.1:${settings.ports.bind}`);
      if (settings.psiphon) {
        await sleep(900);
        if (!this.alive(gen)) {
          return;
        }
        this.log("ok", `psiphon SOCKS5 listener ready on 127.0.0.1:${settings.ports.psiphon}`);
      }
      this.set({ state: "VERIFYING" });
      await sleep(700);
      if (!this.alive(gen)) {
        return;
      }
      const target = TRACE_TARGETS[0];
      const egress = this.trace(transport, serverId, region);
      this.log("info", `GET https://${target}/cdn-cgi/trace via socks5 127.0.0.1:${ports.socks}: ip=${egress.ip} loc=${egress.loc} colo=${egress.colo} warp=${egress.warp}`);
      if (SANCTIONED_LOCS.includes(egress.loc)) {
        this.log("warn", `exit loc=${egress.loc} is sanctioned, rejecting ${transport}`);
        continue;
      }
      if (settings.sanctionCheck) {
        for (const site of SANCTION_SITES) {
          await sleep(350);
          if (!this.alive(gen)) {
            return;
          }
          this.log("ok", `GET https://${site}/ via socks5 127.0.0.1:${ports.socks} -> 200`);
        }
      }
      this.shuffleLeft = settings.shuffle.intervalMin * 60;
      this.set({ state: "RUNNING", transport, egress, message: "" });
      if (settings.systemProxy) {
        this.log("ok", `system proxy enabled -> 127.0.0.1:${ports.http}`);
      }
      this.log("ok", `tunnel verified over ${transport}, egress ${egress.ip} (${egress.loc})`);
      this.startTicker();
      return;
    }
    this.set({ state: "ERROR", message: "NO_TRANSPORT" });
    this.log("error", "every transport in the chain failed verification");
  }

  private tick(): void {
    const settings = this.settings;
    if (this.snap.state === "RUNNING") {
      const rx = Math.max(40_000, this.snap.rxBps * 0.6 + (180_000 + Math.random() * 2_400_000) * 0.4);
      const tx = Math.max(12_000, this.snap.txBps * 0.6 + (60_000 + Math.random() * 520_000) * 0.4);
      this.set({ uptimeSecs: this.snap.uptimeSecs + 1, rxBps: rx, txBps: tx });
      if (settings && settings.shuffle.enabled && settings.psiphon) {
        this.shuffleLeft -= 1;
        if (this.shuffleLeft <= 0) {
          this.shuffleLeft = settings.shuffle.intervalMin * 60;
          this.rotateRegion(settings);
        }
      }
    }
  }

  private rotateRegion(settings: Settings): void {
    const pool = settings.shuffle.regions.filter((region) => region !== this.snap.region);
    if (pool.length === 0) {
      return;
    }
    const region = pool[Math.floor(Math.random() * pool.length)];
    const server = serverByRegion(region, "");
    const egress: Egress = { ip: fakeIp(`${server.id}-${Date.now()}`), loc: region, colo: server.colo, warp: "on" };
    this.set({ region, serverId: server.id, egress });
    this.log("info", `shuffle: psiphon exit region ${region} (WARP carrier kept)`);
  }
}

class NativeEngine implements Engine {
  private snap: SessionSnapshot = initialSnapshot();
  private subscribers = new Set<Subscriber>();
  private timer: number | null = null;

  subscribe(onSnapshot: SnapshotHandler, onLog: LogHandler): () => void {
    const subscriber: Subscriber = { onSnapshot, onLog };
    this.subscribers.add(subscriber);
    onSnapshot(this.snap);
    if (this.timer === null) {
      void this.poll();
      this.timer = window.setInterval(() => void this.poll(), 1000);
    }
    return () => {
      this.subscribers.delete(subscriber);
      if (this.subscribers.size === 0 && this.timer !== null) {
        window.clearInterval(this.timer);
        this.timer = null;
      }
    };
  }

  snapshot(): SessionSnapshot {
    return this.snap;
  }

  configure(settings: Settings): void {
    void call<void>("session_configure", { settings }, () => undefined);
  }

  connect(settings: Settings, serverId: string, region: string): void {
    void call<void>("session_connect", { settings, serverId, region }, () => undefined).then(() => this.poll());
  }

  disconnect(): void {
    void call<void>("session_disconnect", {}, () => undefined).then(() => this.poll());
  }

  pause(): void {
    void call<void>("session_pause", {}, () => undefined).then(() => this.poll());
  }

  resume(): void {
    void call<void>("session_resume", {}, () => undefined).then(() => this.poll());
  }

  private async poll(): Promise<void> {
    const snapshot = await call<SessionSnapshot>("session_snapshot", {}, () => this.snap);
    this.snap = snapshot;
    const logs = await call<DrainedLog[]>("session_drain_logs", {}, () => []);
    this.subscribers.forEach((subscriber) => {
      subscriber.onSnapshot(snapshot);
      logs.forEach((entry) => subscriber.onLog(entry.level, entry.text));
    });
  }
}

export function createEngine(): Engine {
  return isTauri ? new NativeEngine() : new SimulatedEngine();
}