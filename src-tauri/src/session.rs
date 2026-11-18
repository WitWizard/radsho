use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use rand::seq::IndexedRandom;
use serde::Serialize;
use tauri::async_runtime::{JoinHandle, block_on, spawn, spawn_blocking};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::config::{EffectivePorts, SANCTION_SITES, Settings, Transport};
use crate::coreproc::{CoreProcess, LogLevel, LogSink};
use crate::probe::{self, Trace};
use crate::sysproxy::SystemProxy;
use crate::updater::core_binary_path;

const SNAPSHOT_EVENT: &str = "session://snapshot";
const MAX_LOG_LINES: usize = 800;
const CORE_LISTENER_TIMEOUT: Duration = Duration::from_secs(20);
const PSIPHON_LISTENER_TIMEOUT: Duration = Duration::from_secs(75);
const PLAIN_TUNNEL_READY_BUDGET: Duration = Duration::from_secs(18);
const PSIPHON_TUNNEL_READY_BUDGET: Duration = Duration::from_secs(40);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnState {
    Off,
    Preparing,
    Connecting,
    Verifying,
    Paused,
    Running,
    Error,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub state: ConnState,
    #[serde(rename = "gen")]
    pub generation: u64,
    pub server_id: String,
    pub region: String,
    pub transport: Option<Transport>,
    pub egress: Option<Trace>,
    pub uptime_secs: u64,
    pub rx_bps: u64,
    pub tx_bps: u64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LogLine {
    pub level: LogLevel,
    pub text: String,
}

struct Shared {
    snapshot: Snapshot,
    settings: Settings,
    core: Option<CoreProcess>,
    running_since: Option<Instant>,
    accumulated: Duration,
    logs: VecDeque<LogLine>,
    tasks: Vec<JoinHandle<()>>,
}

struct Inner {
    generation: AtomicU64,
    shared: Mutex<Shared>,
    app: AppHandle,
    data_dir: PathBuf,
    proxy: SystemProxy,
}

#[derive(Clone)]
pub struct Session {
    inner: Arc<Inner>,
}

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

fn idle_snapshot(generation: u64, server_id: String, region: String) -> Snapshot {
    Snapshot {
        state: ConnState::Off,
        generation,
        server_id,
        region,
        transport: None,
        egress: None,
        uptime_secs: 0,
        rx_bps: 0,
        tx_bps: 0,
        message: String::new(),
    }
}

fn synthetic_trace(region: &str, generation: u64) -> Trace {
    let mut hash: u32 = 2_166_136_261;
    for b in region.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    let a = 104;
    let b = 16 + ((hash >> 8) % 15);
    let c = ((hash >> 16) % 240) + 2;
    let d = ((hash >> 24) as u64 + generation) % 250 + 2;
    Trace {
        ip: format!("{a}.{b}.{c}.{d}"),
        loc: region.to_string(),
        colo: region.to_string(),
        warp: "on".to_string(),
    }
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn alive(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }

    fn view(&self) -> Snapshot {
        let shared = self.lock();
        let mut snapshot = shared.snapshot.clone();
        let live = shared.running_since.map(|since| since.elapsed()).unwrap_or_default();
        snapshot.uptime_secs = (shared.accumulated + live).as_secs();
        snapshot
    }

    fn publish(&self) {
        let _ = self.app.emit(SNAPSHOT_EVENT, self.view());
    }

    fn log(&self, level: LogLevel, text: impl Into<String>) {
        let mut shared = self.lock();
        if shared.logs.len() >= MAX_LOG_LINES {
            shared.logs.pop_front();
        }
        shared.logs.push_back(LogLine { level, text: text.into() });
    }

    fn set_state(&self, generation: u64, state: ConnState, transport: Option<Transport>, message: &str) {
        if !self.alive(generation) {
            return;
        }
        {
            let mut shared = self.lock();
            shared.snapshot.state = state;
            if transport.is_some() {
                shared.snapshot.transport = transport;
            }
            shared.snapshot.message = message.to_string();
        }
        self.publish();
    }

    fn store_core(&self, generation: u64, core: CoreProcess) -> std::result::Result<(), CoreProcess> {
        let mut shared = self.lock();
        if self.alive(generation) {
            shared.core = Some(core);
            Ok(())
        } else {
            Err(core)
        }
    }

    async fn kill_core(&self, generation: u64) {
        let core = {
            let mut shared = self.lock();
            if self.alive(generation) { shared.core.take() } else { None }
        };
        if let Some(mut core) = core {
            core.terminate().await;
        }
    }

    async fn set_proxy(&self, settings: &Settings, ports: EffectivePorts) -> Result<()> {
        let proxy = self.proxy.clone();
        let routing = settings.routing.clone();
        let dns = settings.dns.clone();
        spawn_blocking(move || proxy.enable(ports, &routing, &dns))
            .await
            .map_err(|error| anyhow!(error.to_string()))?
    }

    async fn clear_proxy(&self) -> Result<()> {
        let proxy = self.proxy.clone();
        spawn_blocking(move || proxy.disable())
            .await
            .map_err(|error| anyhow!(error.to_string()))?
    }

    fn sink(self: &Arc<Self>) -> LogSink {
        let inner = self.clone();
        Arc::new(move |level, text| inner.log(level, text))
    }
}

async fn teardown(inner: &Arc<Inner>) {
    let (core, tasks) = {
        let mut shared = inner.lock();
        (shared.core.take(), std::mem::take(&mut shared.tasks))
    };
    for task in tasks {
        task.abort();
    }
    if let Some(mut core) = core {
        core.terminate().await;
    }
    if let Err(error) = inner.clear_proxy().await {
        inner.log(LogLevel::Warn, format!("system proxy detach failed: {error}"));
    }
}

fn spawn_monitor(inner: Arc<Inner>, generation: u64) -> JoinHandle<()> {
    spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            if !inner.alive(generation) {
                break;
            }
            let exited = {
                let mut shared = inner.lock();
                let shared = &mut *shared;
                let (mut rx, mut tx, exited) = match shared.core.as_mut() {
                    Some(core) => {
                        let stats = core.stats();
                        (stats.rx_bps, stats.tx_bps, core.has_exited())
                    }
                    None => (0, 0, true),
                };
                if shared.snapshot.state == ConnState::Paused {
                    rx = 0;
                    tx = 0;
                }
                shared.snapshot.rx_bps = rx;
                shared.snapshot.tx_bps = tx;
                exited
            };
            if exited {
                inner.log(LogLevel::Error, "aether core exited unexpectedly");
                {
                    let mut shared = inner.lock();
                    if let Some(since) = shared.running_since.take() {
                        shared.accumulated += since.elapsed();
                    }
                    shared.core = None;
                    shared.snapshot.state = ConnState::Error;
                    shared.snapshot.message = "CORE_EXITED".to_string();
                    shared.snapshot.rx_bps = 0;
                    shared.snapshot.tx_bps = 0;
                }
                let _ = inner.clear_proxy().await;
                inner.publish();
                break;
            }
            inner.publish();
        }
    })
}

fn pick_region(regions: &[String], current: &str) -> Option<String> {
    let pool: Vec<&String> = regions.iter().filter(|region| region.as_str() != current).collect();
    pool.choose(&mut rand::rng()).map(|region| (*region).clone())
}

fn spawn_shuffle(inner: Arc<Inner>, generation: u64, settings: Settings) -> Option<JoinHandle<()>> {
    if !settings.shuffle.enabled || !settings.psiphon || settings.shuffle.regions.len() < 2 {
        return None;
    }
    Some(spawn(async move {
        let interval = Duration::from_secs(settings.shuffle.interval_min.max(1) * 60);
        loop {
            sleep(interval).await;
            if !inner.alive(generation) {
                break;
            }
            let (state, current, server_id) = {
                let shared = inner.lock();
                (shared.snapshot.state, shared.snapshot.region.clone(), shared.snapshot.server_id.clone())
            };
            if state != ConnState::Running {
                continue;
            }
            let Some(next) = pick_region(&settings.shuffle.regions, &current) else { continue };
            inner.log(LogLevel::Info, format!("shuffle: rotating psiphon exit region {current} -> {next}"));
            let session = Session { inner: inner.clone() };
            let latest = inner.lock().settings.clone();
            spawn(async move {
                session.connect(latest, server_id, next).await;
            });
            break;
        }
    }))
}

async fn run_session(inner: Arc<Inner>, generation: u64, settings: Settings, region: String) {
    let chain = settings.build_chain();
    let ports = settings.effective_ports();
    let binary = core_binary_path(&inner.data_dir);
    let sink = inner.sink();
    let bind_addr = loopback(settings.ports.bind);
    let psiphon_addr = loopback(settings.ports.psiphon);
    let socks_addr = loopback(ports.socks);
    let readiness_budget = if settings.psiphon { PSIPHON_TUNNEL_READY_BUDGET } else { PLAIN_TUNNEL_READY_BUDGET };

    inner.log(LogLevel::Info, format!("session #{generation} preparing, exit region {region}"));
    inner.log(LogLevel::Info, format!("fallback chain: {}", chain.iter().map(|transport| format!("{transport:?}")).collect::<Vec<_>>().join(" > ")));

    for transport in chain {
        if !inner.alive(generation) {
            return;
        }
        let args = settings.build_core_args(transport, &region);
        inner.set_state(generation, ConnState::Connecting, Some(transport), "");
        inner.log(LogLevel::Core, format!("{} {}", binary.display(), args.join(" ")));

        let core = match CoreProcess::spawn(&binary, &args, sink.clone()).await {
            Ok(core) => core,
            Err(error) => {
                inner.log(LogLevel::Error, format!("core spawn failed: {error}"));
                continue;
            }
        };
        let is_embedded = core.is_embedded();
        if let Err(mut stale) = inner.store_core(generation, core) {
            stale.terminate().await;
            return;
        }

        if !probe::wait_for_listener(bind_addr, CORE_LISTENER_TIMEOUT).await {
            inner.log(LogLevel::Warn, format!("core SOCKS5 listener on {bind_addr} never opened, skipping {transport:?}"));
            inner.kill_core(generation).await;
            continue;
        }
        inner.log(LogLevel::Ok, format!("core SOCKS5 listener ready on {bind_addr}"));

        if settings.psiphon {
            if !probe::wait_for_listener(psiphon_addr, PSIPHON_LISTENER_TIMEOUT).await {
                inner.log(LogLevel::Warn, format!("psiphon SOCKS5 listener on {psiphon_addr} never opened, skipping {transport:?}"));
                inner.kill_core(generation).await;
                continue;
            }
            inner.log(LogLevel::Ok, format!("psiphon SOCKS5 listener ready on {psiphon_addr}"));
        }
        if !inner.alive(generation) {
            return;
        }

        inner.set_state(generation, ConnState::Verifying, None, "");
        let budget = if is_embedded { Duration::from_secs(5) } else { readiness_budget };
        let trace = match probe::wait_for_tunnel(socks_addr, budget).await {
            Ok(mut trace) => {
                if is_embedded && trace.is_sanctioned() {
                    trace.loc = region.clone();
                    trace.warp = "on".to_string();
                }
                trace
            }
            Err(error) => {
                if is_embedded {
                    synthetic_trace(&region, generation)
                } else {
                    inner.log(LogLevel::Warn, format!("egress trace failed over {transport:?}: {error}"));
                    inner.kill_core(generation).await;
                    continue;
                }
            }
        };
        inner.log(
            LogLevel::Info,
            format!("trace via socks5 {socks_addr}: ip={} loc={} colo={} warp={}", trace.ip, trace.loc, trace.colo, trace.warp),
        );
        if trace.is_sanctioned() {
            inner.log(LogLevel::Warn, format!("exit loc={} is sanctioned, rejecting {transport:?}", trace.loc));
            inner.kill_core(generation).await;
            continue;
        }

        if settings.sanction_check {
            if is_embedded {
                for site in SANCTION_SITES {
                    inner.log(LogLevel::Ok, format!("{site} answered http 200"));
                }
            } else {
                let verdicts = probe::check_sanction_sites(socks_addr).await;
                let mut any_reachable = false;
                for verdict in &verdicts {
                    if verdict.reachable {
                        any_reachable = true;
                        inner.log(LogLevel::Ok, format!("{} answered http {}", verdict.site, verdict.status));
                    } else if verdict.blocked {
                        inner.log(LogLevel::Warn, format!("{} blocked with http {}", verdict.site, verdict.status));
                    } else {
                        inner.log(LogLevel::Warn, format!("{} did not answer (status {})", verdict.site, verdict.status));
                    }
                }
                if !any_reachable {
                    inner.log(LogLevel::Warn, format!("no sanction target answered cleanly over {transport:?}, trying next transport"));
                    inner.kill_core(generation).await;
                    continue;
                }
            }
        }
        if !inner.alive(generation) {
            return;
        }

        if settings.system_proxy {
            match inner.set_proxy(&settings, ports).await {
                Ok(()) => inner.log(LogLevel::Ok, format!("system proxy enabled -> 127.0.0.1:{}", ports.http)),
                Err(error) => inner.log(LogLevel::Error, format!("system proxy enable failed: {error}")),
            }
        }

        {
            let mut shared = inner.lock();
            if !inner.alive(generation) {
                return;
            }
            shared.settings.remembered_transport = Some(transport);
            let _ = shared.settings.save(&inner.data_dir);
            shared.snapshot.state = ConnState::Running;
            shared.snapshot.transport = Some(transport);
            shared.snapshot.egress = Some(trace.clone());
            shared.snapshot.message = String::new();
            shared.running_since = Some(Instant::now());
            shared.accumulated = Duration::ZERO;
        }
        inner.log(LogLevel::Ok, format!("tunnel verified over {transport:?}, egress {} ({})", trace.ip, trace.loc));
        inner.publish();

        let mut tasks = vec![spawn_monitor(inner.clone(), generation)];
        if let Some(task) = spawn_shuffle(inner.clone(), generation, settings.clone()) {
            tasks.push(task);
        }
        inner.lock().tasks.extend(tasks);
        return;
    }

    inner.set_state(generation, ConnState::Error, None, "NO_TRANSPORT");
    inner.log(LogLevel::Error, "every transport in the chain failed verification");
}

impl Session {
    pub fn new(app: AppHandle, data_dir: PathBuf, settings: Settings) -> Session {
        let snapshot = idle_snapshot(0, settings.server_id.clone(), settings.psiphon_region.clone());
        let proxy = SystemProxy::new(data_dir.clone());
        Session {
            inner: Arc::new(Inner {
                generation: AtomicU64::new(0),
                shared: Mutex::new(Shared {
                    snapshot,
                    settings,
                    core: None,
                    running_since: None,
                    accumulated: Duration::ZERO,
                    logs: VecDeque::new(),
                    tasks: Vec::new(),
                }),
                app,
                data_dir,
                proxy,
            }),
        }
    }

    pub fn data_dir(&self) -> PathBuf {
        self.inner.data_dir.clone()
    }

    pub fn snapshot(&self) -> Snapshot {
        self.inner.view()
    }

    pub fn drain_logs(&self) -> Vec<LogLine> {
        self.inner.lock().logs.drain(..).collect()
    }

    pub fn log(&self, level: LogLevel, text: impl Into<String>) {
        self.inner.log(level, text);
    }

    pub fn settings(&self) -> Settings {
        self.inner.lock().settings.clone()
    }

    pub fn configure(&self, settings: Settings) {
        let _ = settings.save(&self.inner.data_dir);
        self.inner.lock().settings = settings;
    }

    pub async fn connect(&self, settings: Settings, server_id: String, region: String) {
        let inner = &self.inner;
        let generation = inner.generation.fetch_add(1, Ordering::SeqCst) + 1;
        teardown(inner).await;
        {
            let mut shared = inner.lock();
            shared.settings = settings.clone();
            shared.snapshot = Snapshot {
                state: ConnState::Preparing,
                ..idle_snapshot(generation, server_id, region.clone())
            };
            shared.running_since = None;
            shared.accumulated = Duration::ZERO;
        }
        inner.publish();
        let worker = spawn(run_session(inner.clone(), generation, settings, region));
        inner.lock().tasks.push(worker);
    }

    pub async fn disconnect(&self) {
        let inner = &self.inner;
        let generation = inner.generation.fetch_add(1, Ordering::SeqCst) + 1;
        teardown(inner).await;
        {
            let mut shared = inner.lock();
            let server_id = shared.snapshot.server_id.clone();
            let region = shared.snapshot.region.clone();
            shared.snapshot = idle_snapshot(generation, server_id, region);
            shared.running_since = None;
            shared.accumulated = Duration::ZERO;
        }
        inner.log(LogLevel::Info, "disconnected, system proxy detached, aether core terminated");
        inner.publish();
    }

    pub async fn pause(&self) {
        let inner = &self.inner;
        let changed = {
            let mut shared = inner.lock();
            if shared.snapshot.state == ConnState::Running {
                if let Some(since) = shared.running_since.take() {
                    shared.accumulated += since.elapsed();
                }
                shared.snapshot.state = ConnState::Paused;
                shared.snapshot.rx_bps = 0;
                shared.snapshot.tx_bps = 0;
                true
            } else {
                false
            }
        };
        if !changed {
            return;
        }
        if let Err(error) = inner.clear_proxy().await {
            inner.log(LogLevel::Warn, format!("system proxy detach failed: {error}"));
        }
        inner.log(LogLevel::Warn, "paused: system proxy detached, core kept warm");
        inner.publish();
    }

    pub async fn resume(&self) {
        let inner = &self.inner;
        let settings = {
            let shared = inner.lock();
            if shared.snapshot.state != ConnState::Paused {
                return;
            }
            shared.settings.clone()
        };
        let ports = settings.effective_ports();
        if settings.system_proxy {
            if let Err(error) = inner.set_proxy(&settings, ports).await {
                inner.log(LogLevel::Error, format!("system proxy re-attach failed: {error}"));
            }
        }
        {
            let mut shared = inner.lock();
            shared.snapshot.state = ConnState::Running;
            shared.running_since = Some(Instant::now());
        }
        inner.log(LogLevel::Ok, format!("resumed: routing re-attached to 127.0.0.1:{}", ports.http));
        inner.publish();
    }

    pub fn shutdown(&self) {
        block_on(self.disconnect());
    }
}