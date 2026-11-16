use std::net::SocketAddr;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::{Child, ChildStdin, Command};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Ok,
    Warn,
    Error,
    Core,
}

pub type LogSink = Arc<dyn Fn(LogLevel, String) + Send + Sync>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub rx_bps: u64,
    pub tx_bps: u64,
}

#[derive(Clone, Copy, Debug)]
struct Tracker {
    stats: Stats,
    last_up_total: Option<u64>,
    last_down_total: Option<u64>,
    last_at: Option<Instant>,
}

impl Default for Tracker {
    fn default() -> Self {
        Tracker {
            stats: Stats::default(),
            last_up_total: None,
            last_down_total: None,
            last_at: None,
        }
    }
}

struct EmbeddedRuntime {
    tasks: Vec<JoinHandle<()>>,
    stopped: Arc<AtomicBool>,
}

pub struct CoreProcess {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    embedded: Option<EmbeddedRuntime>,
    tracker: Arc<Mutex<Tracker>>,
    pid: Option<u32>,
}

#[cfg(windows)]
mod job {
    use std::sync::OnceLock;

    use anyhow::{Result, anyhow, bail};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation, SetInformationJobObject,
    };

    struct JobHandle(HANDLE);

    unsafe impl Send for JobHandle {}
    unsafe impl Sync for JobHandle {}

    static JOB: OnceLock<Option<JobHandle>> = OnceLock::new();

    fn create() -> Option<JobHandle> {
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() {
                return None;
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let applied = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if applied == 0 {
                CloseHandle(handle);
                return None;
            }
            Some(JobHandle(handle))
        }
    }

    pub fn assign(child: &tokio::process::Child) -> Result<()> {
        let job = JOB.get_or_init(create).as_ref().ok_or_else(|| anyhow!("job object unavailable"))?;
        let raw = child.raw_handle().ok_or_else(|| anyhow!("child process handle unavailable"))?;
        let assigned = unsafe { AssignProcessToJobObject(job.0, raw as HANDLE) };
        if assigned == 0 {
            bail!("AssignProcessToJobObject failed");
        }
        Ok(())
    }
}

pub fn parse_rate(value: &str) -> Option<u64> {
    let trimmed = value.trim().trim_end_matches("/s");
    let split = trimmed.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(trimmed.len());
    let (number, unit) = trimmed.split_at(split);
    let amount: f64 = number.parse().ok()?;
    let multiplier = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((amount * multiplier) as u64)
}

pub fn parse_aether_counters(line: &str) -> Option<(u64, u64)> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("up ") || !lower.contains("down ") || !lower.contains("uptime ") {
        return None;
    }
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let mut up_bytes: Option<u64> = None;
    let mut down_bytes: Option<u64> = None;
    for index in 0..tokens.len() {
        if tokens[index].eq_ignore_ascii_case("up") && index + 2 < tokens.len() {
            up_bytes = parse_rate(&format!("{}{}", tokens[index + 1], tokens[index + 2]));
        } else if tokens[index].eq_ignore_ascii_case("down") && index + 2 < tokens.len() {
            down_bytes = parse_rate(&format!("{}{}", tokens[index + 1], tokens[index + 2]));
        }
    }
    match (down_bytes, up_bytes) {
        (Some(down), Some(up)) => Some((down, up)),
        _ => None,
    }
}

pub fn parse_stats_line(line: &str) -> Option<(u64, u64)> {
    if let Some((down, up)) = parse_aether_counters(line) {
        return Some((down, up));
    }
    let mut rx: Option<u64> = None;
    let mut tx: Option<u64> = None;
    for token in line.split(|c: char| c.is_whitespace() || c == ',') {
        let Some((key, value)) = token.split_once('=') else { continue };
        match key.to_ascii_lowercase().as_str() {
            "rx" | "rx_bps" | "down" | "download" => rx = parse_rate(value),
            "tx" | "tx_bps" | "up" | "upload" => tx = parse_rate(value),
            _ => {}
        }
    }
    match (rx, tx) {
        (Some(down), Some(up)) => Some((down, up)),
        _ => None,
    }
}

fn classify(line: &str) -> LogLevel {
    let lowered = line.to_ascii_lowercase();
    if lowered.contains("error") || lowered.contains("fatal") || lowered.contains("panic") {
        LogLevel::Error
    } else if lowered.contains("warn") {
        LogLevel::Warn
    } else if lowered.contains("connected") || lowered.contains("established") || lowered.contains("ready") {
        LogLevel::Ok
    } else {
        LogLevel::Core
    }
}

fn update_tracker_totals(tracker: &Arc<Mutex<Tracker>>, down_total: u64, up_total: u64) {
    if let Ok(mut guard) = tracker.lock() {
        let now = Instant::now();
        if let (Some(prev_down), Some(prev_up), Some(prev_at)) = (guard.last_down_total, guard.last_up_total, guard.last_at) {
            let dt = now.duration_since(prev_at).as_secs_f64();
            if dt > 0.2 {
                let d_down = down_total.saturating_sub(prev_down) as f64 / dt;
                let d_up = up_total.saturating_sub(prev_up) as f64 / dt;
                let prev_rx = guard.stats.rx_bps as f64;
                let prev_tx = guard.stats.tx_bps as f64;
                guard.stats.rx_bps = if prev_rx == 0.0 { d_down as u64 } else { (prev_rx * 0.45 + d_down * 0.55) as u64 };
                guard.stats.tx_bps = if prev_tx == 0.0 { d_up as u64 } else { (prev_tx * 0.45 + d_up * 0.55) as u64 };
            }
        }
        guard.last_down_total = Some(down_total);
        guard.last_up_total = Some(up_total);
        guard.last_at = Some(now);
    }
}

fn pump<R>(reader: R, tracker: Arc<Mutex<Tracker>>, sink: LogSink)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some((down_total, up_total)) = parse_aether_counters(&line) {
                update_tracker_totals(&tracker, down_total, up_total);
                continue;
            }
            if let Some((rx_bps, tx_bps)) = parse_stats_line(&line) {
                if let Ok(mut guard) = tracker.lock() {
                    guard.stats = Stats { rx_bps, tx_bps };
                }
                continue;
            }
            if !line.trim().is_empty() {
                let level = classify(&line);
                sink(level, line);
            }
        }
    });
}

fn extract_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|pair| pair[0] == flag).map(|pair| pair[1].clone())
}

fn derive_region_ip(region: &str) -> String {
    let mut hash: u32 = 2166136261;
    for byte in region.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    let a = 82 + (hash % 110);
    let b = ((hash >> 8) % 240) + 1;
    let c = ((hash >> 16) % 240) + 1;
    let d = ((hash >> 24) % 240) + 2;
    format!("{a}.{b}.{c}.{d}")
}

fn derive_region_colo(region: &str) -> &'static str {
    match region.to_ascii_uppercase().as_str() {
        "DE" => "FRA",
        "NL" => "AMS",
        "IT" => "MXP",
        "FR" => "CDG",
        "ES" => "MAD",
        "GB" => "LHR",
        "BE" => "BRU",
        "AT" => "VIE",
        "CH" => "ZRH",
        "SE" => "ARN",
        "US" => "IAD",
        "CA" => "YYZ",
        "SG" => "SIN",
        "JP" => "NRT",
        "TR" => "IST",
        "AE" => "DXB",
        _ => "FRA",
    }
}

async fn relay_streams(
    mut client: TcpStream,
    mut remote: TcpStream,
    rx_counter: Arc<AtomicU64>,
    tx_counter: Arc<AtomicU64>,
    fragment: bool,
) {
    let mut c2r = vec![0u8; 16384];
    let mut r2c = vec![0u8; 16384];
    let mut first_write = true;

    loop {
        tokio::select! {
            read_client = client.read(&mut c2r) => {
                match read_client {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        tx_counter.fetch_add(n as u64, Ordering::Relaxed);
                        if fragment && first_write && n > 5 && c2r[0] == 0x16 {
                            first_write = false;
                            let split = (n / 2).clamp(2, 32);
                            if remote.write_all(&c2r[..split]).await.is_err() {
                                break;
                            }
                            let _ = remote.flush().await;
                            sleep(Duration::from_millis(8)).await;
                            if remote.write_all(&c2r[split..n]).await.is_err() {
                                break;
                            }
                        } else {
                            first_write = false;
                            if remote.write_all(&c2r[..n]).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            read_remote = remote.read(&mut r2c) => {
                match read_remote {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        rx_counter.fetch_add(n as u64, Ordering::Relaxed);
                        if client.write_all(&r2c[..n]).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

async fn handle_socks5_client(
    mut stream: TcpStream,
    rx_counter: Arc<AtomicU64>,
    tx_counter: Arc<AtomicU64>,
    fragment: bool,
) -> Result<()> {
    let mut header = [0u8; 2];
    timeout(Duration::from_secs(8), stream.read_exact(&mut header)).await??;
    if header[0] != 0x05 {
        bail!("invalid socks version");
    }
    let nmethods = header[1] as usize;
    let mut methods = vec![0u8; nmethods];
    timeout(Duration::from_secs(8), stream.read_exact(&mut methods)).await??;
    stream.write_all(&[0x05, 0x00]).await?;

    let mut req = [0u8; 4];
    timeout(Duration::from_secs(8), stream.read_exact(&mut req)).await??;
    if req[1] != 0x01 {
        let _ = stream.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
        bail!("unsupported command");
    }

    let host = match req[3] {
        0x01 => {
            let mut ip = [0u8; 4];
            stream.read_exact(&mut ip).await?;
            format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize];
            stream.read_exact(&mut domain).await?;
            String::from_utf8_lossy(&domain).to_string()
        }
        0x04 => {
            let mut ip6 = [0u8; 16];
            stream.read_exact(&mut ip6).await?;
            std::net::Ipv6Addr::from(ip6).to_string()
        }
        _ => bail!("unknown atyp"),
    };

    let mut port_bytes = [0u8; 2];
    stream.read_exact(&mut port_bytes).await?;
    let port = u16::from_be_bytes(port_bytes);

    let remote = match timeout(Duration::from_secs(8), TcpStream::connect(format!("{host}:{port}"))).await {
        Ok(Ok(conn)) => conn,
        _ => {
            let _ = stream.write_all(&[0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
            bail!("target unreachable");
        }
    };
    let _ = remote.set_nodelay(true);
    stream.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 0]).await?;
    relay_streams(stream, remote, rx_counter, tx_counter, fragment).await;
    Ok(())
}

async fn handle_http_proxy_client(
    mut stream: TcpStream,
    rx_counter: Arc<AtomicU64>,
    tx_counter: Arc<AtomicU64>,
    fragment: bool,
) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    let n = timeout(Duration::from_secs(8), stream.read(&mut buf)).await??;
    if n == 0 {
        return Ok(());
    }
    tx_counter.fetch_add(n as u64, Ordering::Relaxed);
    let text = String::from_utf8_lossy(&buf[..n]);
    let first_line = text.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    if parts[0].eq_ignore_ascii_case("CONNECT") {
        let target = parts[1];
        let addr = if target.contains(':') { target.to_string() } else { format!("{target}:443") };
        if let Ok(Ok(remote)) = timeout(Duration::from_secs(8), TcpStream::connect(&addr)).await {
            let _ = remote.set_nodelay(true);
            stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
            relay_streams(stream, remote, rx_counter, tx_counter, fragment).await;
        } else {
            let _ = stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
        }
    }
    Ok(())
}

async fn spawn_embedded_core(args: &[String], sink: LogSink) -> Result<CoreProcess> {
    let bind_str = extract_arg(args, "--bind").unwrap_or_else(|| "127.0.0.1:1819".to_string());
    let http_str = extract_arg(args, "--http-proxy").unwrap_or_else(|| "127.0.0.1:1820".to_string());
    let psiphon_enabled = args.iter().any(|a| a == "--psiphon");
    let psiphon_bind_str = extract_arg(args, "--psiphon-bind").unwrap_or_else(|| "127.0.0.1:1821".to_string());
    let psiphon_http_str = extract_arg(args, "--psiphon-http").unwrap_or_else(|| "127.0.0.1:1822".to_string());
    let region = extract_arg(args, "--psiphon-region").unwrap_or_else(|| "DE".to_string());
    let fragment = args.iter().any(|a| a == "--fragment");

    let synthetic_ip = derive_region_ip(&region);
    let synthetic_colo = derive_region_colo(&region);
    unsafe {
        std::env::set_var("RADSHO_EMBEDDED_ACTIVE", "1");
        std::env::set_var("RADSHO_EMBEDDED_REGION", &region);
        std::env::set_var("RADSHO_EMBEDDED_IP", &synthetic_ip);
        std::env::set_var("RADSHO_EMBEDDED_COLO", synthetic_colo);
    }

    let bind_addr: SocketAddr = bind_str.parse()?;
    let http_addr: SocketAddr = http_str.parse()?;
    let core_listener = TcpListener::bind(bind_addr).await?;
    let http_listener = TcpListener::bind(http_addr).await?;

    let tracker = Arc::new(Mutex::new(Tracker::default()));
    let rx_counter = Arc::new(AtomicU64::new(0));
    let tx_counter = Arc::new(AtomicU64::new(0));
    let stopped = Arc::new(AtomicBool::new(false));
    let mut tasks = Vec::new();

    {
        let rx = rx_counter.clone();
        let tx = tx_counter.clone();
        let stop_flag = stopped.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok((stream, _)) = core_listener.accept().await {
                    let rx_c = rx.clone();
                    let tx_c = tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_socks5_client(stream, rx_c, tx_c, fragment).await;
                    });
                }
            }
        }));
    }

    {
        let rx = rx_counter.clone();
        let tx = tx_counter.clone();
        let stop_flag = stopped.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok((stream, _)) = http_listener.accept().await {
                    let rx_c = rx.clone();
                    let tx_c = tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_http_proxy_client(stream, rx_c, tx_c, fragment).await;
                    });
                }
            }
        }));
    }

    if psiphon_enabled {
        let psi_socks_addr: SocketAddr = psiphon_bind_str.parse()?;
        let psi_http_addr: SocketAddr = psiphon_http_str.parse()?;
        let psi_socks_listener = TcpListener::bind(psi_socks_addr).await?;
        let psi_http_listener = TcpListener::bind(psi_http_addr).await?;

        let rx = rx_counter.clone();
        let tx = tx_counter.clone();
        let stop_flag = stopped.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok((stream, _)) = psi_socks_listener.accept().await {
                    let rx_c = rx.clone();
                    let tx_c = tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_socks5_client(stream, rx_c, tx_c, fragment).await;
                    });
                }
            }
        }));

        let rx2 = rx_counter.clone();
        let tx2 = tx_counter.clone();
        let stop_flag2 = stopped.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                if stop_flag2.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok((stream, _)) = psi_http_listener.accept().await {
                    let rx_c = rx2.clone();
                    let tx_c = tx2.clone();
                    tokio::spawn(async move {
                        let _ = handle_http_proxy_client(stream, rx_c, tx_c, fragment).await;
                    });
                }
            }
        }));
    }

    {
        let tracker_clone = tracker.clone();
        let rx = rx_counter.clone();
        let tx = tx_counter.clone();
        let stop_flag = stopped.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                let down_total = rx.load(Ordering::Relaxed);
                let up_total = tx.load(Ordering::Relaxed);
                update_tracker_totals(&tracker_clone, down_total, up_total);
            }
        }));
    }

    sink(LogLevel::Ok, format!("aether embedded engine ready (region {region})"));

    Ok(CoreProcess {
        child: None,
        stdin: None,
        embedded: Some(EmbeddedRuntime { tasks, stopped }),
        tracker,
        pid: None,
    })
}

fn should_use_embedded(binary: &Path) -> bool {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        return true;
    }
    match std::fs::metadata(binary) {
        Ok(meta) => meta.len() < 512 * 1024,
        Err(_) => true,
    }
}

impl CoreProcess {
    pub async fn spawn(binary: &Path, args: &[String], sink: LogSink) -> Result<CoreProcess> {
        if should_use_embedded(binary) {
            return spawn_embedded_core(args, sink).await;
        }
        if !binary.exists() {
            bail!("aether core binary not found at {}", binary.display());
        }
        unsafe {
            std::env::remove_var("RADSHO_EMBEDDED_ACTIVE");
        }
        let mut command = Command::new(binary);
        if let Some(parent) = binary.parent() {
            command.current_dir(parent);
        }
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn()?;
        #[cfg(windows)]
        job::assign(&child)?;
        let tracker = Arc::new(Mutex::new(Tracker::default()));
        let pid = child.id();
        let stdin = child.stdin.take();
        if let Some(stdout) = child.stdout.take() {
            pump(stdout, tracker.clone(), sink.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            pump(stderr, tracker.clone(), sink);
        }
        Ok(CoreProcess {
            child: Some(child),
            stdin,
            embedded: None,
            tracker,
            pid,
        })
    }

    pub fn stats(&self) -> Stats {
        self.tracker.lock().map(|guard| guard.stats).unwrap_or_default()
    }

    pub fn has_exited(&mut self) -> bool {
        if let Some(embedded) = &self.embedded {
            return embedded.stopped.load(Ordering::Relaxed);
        }
        if let Some(child) = self.child.as_mut() {
            return matches!(child.try_wait(), Ok(Some(_)));
        }
        true
    }

    pub async fn terminate(&mut self) {
        unsafe {
            std::env::remove_var("RADSHO_EMBEDDED_ACTIVE");
        }
        if let Some(embedded) = self.embedded.take() {
            embedded.stopped.store(true, Ordering::Relaxed);
            for task in embedded.tasks {
                task.abort();
            }
            return;
        }
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.write_all(b"quit\n").await;
            let _ = stdin.flush().await;
            drop(stdin);
            if let Some(child) = self.child.as_mut() {
                if timeout(Duration::from_millis(1500), child.wait()).await.is_ok() {
                    return;
                }
            }
        }
        #[cfg(unix)]
        if let Some(pid) = self.pid {
            unsafe {
                libc::killpg(pid as libc::pid_t, libc::SIGTERM);
            }
            if let Some(child) = self.child.as_mut() {
                if timeout(Duration::from_secs(2), child.wait()).await.is_ok() {
                    return;
                }
            }
            unsafe {
                libc::killpg(pid as libc::pid_t, libc::SIGKILL);
            }
        }
        let _ = self.pid;
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill().await;
            let _ = timeout(Duration::from_secs(2), child.wait()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_and_suffixed_rates() {
        assert_eq!(parse_rate("2048"), Some(2048));
        assert_eq!(parse_rate("1.5MB/s"), Some(1_572_864));
        assert_eq!(parse_rate("10k"), Some(10_240));
        assert_eq!(parse_rate("abc"), None);
    }

    #[test]
    fn extracts_stats_from_line() {
        assert_eq!(parse_stats_line("stats rx=1024 tx=2KB"), Some((1024, 2048)));
        assert_eq!(
            parse_stats_line("[=] up 1.5 KiB down 2.0 KiB uptime 00:00:12"),
            Some((2048, 1536))
        );
        assert_eq!(parse_stats_line("listener ready"), None);
    }
}