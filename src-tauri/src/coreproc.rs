use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
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

#[derive(Clone, Copy, Debug, Default)]
struct Tracker {
    stats: Stats,
    last_up_total: Option<u64>,
    last_down_total: Option<u64>,
    last_at: Option<Instant>,
}

enum RuntimeBackend {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    External {
        child: Child,
        stdin: Option<ChildStdin>,
        pid: Option<u32>,
    },
    Embedded {
        exited: Arc<AtomicBool>,
        tasks: Vec<JoinHandle<()>>,
    },
}

pub struct CoreProcess {
    backend: RuntimeBackend,
    tracker: Arc<Mutex<Tracker>>,
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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn pump<R>(reader: R, tracker: Arc<Mutex<Tracker>>, sink: LogSink)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some((down_total, up_total)) = parse_aether_counters(&line) {
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

#[derive(Clone, Debug)]
struct EmbeddedConfig {
    bind: SocketAddr,
    http: SocketAddr,
    psiphon_bind: Option<SocketAddr>,
    psiphon_http: Option<SocketAddr>,
}

fn parse_addr_arg(args: &[String], flag: &str, default_port: u16) -> SocketAddr {
    for pair in args.windows(2) {
        if pair[0] == flag {
            if let Ok(addr) = pair[1].parse::<SocketAddr>() {
                return addr;
            }
        }
    }
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), default_port)
}

fn parse_embedded_args(args: &[String]) -> EmbeddedConfig {
    let bind = parse_addr_arg(args, "--bind", 1819);
    let http = parse_addr_arg(args, "--http-proxy", 1820);
    let has_psiphon = args.iter().any(|arg| arg == "--psiphon");
    let psiphon_bind = if has_psiphon {
        Some(parse_addr_arg(args, "--psiphon-bind", 1821))
    } else {
        None
    };
    let psiphon_http = if has_psiphon {
        Some(parse_addr_arg(args, "--psiphon-http", 1822))
    } else {
        None
    };
    EmbeddedConfig {
        bind,
        http,
        psiphon_bind,
        psiphon_http,
    }
}

fn add_traffic(tracker: &Arc<Mutex<Tracker>>, rx_bytes: u64, tx_bytes: u64) {
    if let Ok(mut guard) = tracker.lock() {
        let prev_rx = guard.stats.rx_bps;
        let prev_tx = guard.stats.tx_bps;
        guard.stats.rx_bps = ((prev_rx as f64 * 0.55) + (rx_bytes as f64 * 2.2)) as u64;
        guard.stats.tx_bps = ((prev_tx as f64 * 0.55) + (tx_bytes as f64 * 2.2)) as u64;
        guard.last_down_total = Some(guard.last_down_total.unwrap_or(0).saturating_add(rx_bytes));
        guard.last_up_total = Some(guard.last_up_total.unwrap_or(0).saturating_add(tx_bytes));
        guard.last_at = Some(Instant::now());
    }
}

async fn relay_streams(mut client: TcpStream, mut remote: TcpStream, tracker: Arc<Mutex<Tracker>>) -> Result<()> {
    let mut first = [0u8; 2048];
    if let Ok(Ok(n)) = timeout(Duration::from_millis(800), client.read(&mut first)).await {
        if n > 0 {
            if first[0] == 0x16 && n > 24 {
                let split = 11.min(n / 2);
                remote.write_all(&first[..split]).await?;
                remote.flush().await?;
                sleep(Duration::from_millis(12)).await;
                remote.write_all(&first[split..n]).await?;
                remote.flush().await?;
            } else {
                remote.write_all(&first[..n]).await?;
                remote.flush().await?;
            }
            add_traffic(&tracker, 0, n as u64);
        }
    }
    if let Ok((up, down)) = tokio::io::copy_bidirectional(&mut client, &mut remote).await {
        add_traffic(&tracker, down, up);
    }
    Ok(())
}

async fn handle_socks5_client(mut stream: TcpStream, tracker: Arc<Mutex<Tracker>>) -> Result<()> {
    let mut header = [0u8; 2];
    timeout(Duration::from_secs(6), stream.read_exact(&mut header)).await??;
    if header[0] != 0x05 {
        return Ok(());
    }
    let nmethods = header[1] as usize;
    let mut methods = vec![0u8; nmethods];
    timeout(Duration::from_secs(6), stream.read_exact(&mut methods)).await??;
    stream.write_all(&[0x05, 0x00]).await?;

    let mut req = [0u8; 4];
    timeout(Duration::from_secs(6), stream.read_exact(&mut req)).await??;
    if req[0] != 0x05 || req[1] != 0x01 {
        let _ = stream.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
        return Ok(());
    }

    let host = match req[3] {
        0x01 => {
            let mut ip = [0u8; 4];
            stream.read_exact(&mut ip).await?;
            Ipv4Addr::from(ip).to_string()
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut domain = vec![0u8; len[0] as usize];
            stream.read_exact(&mut domain).await?;
            String::from_utf8_lossy(&domain).to_string()
        }
        0x04 => {
            let mut ip = [0u8; 16];
            stream.read_exact(&mut ip).await?;
            std::net::Ipv6Addr::from(ip).to_string()
        }
        _ => return Ok(()),
    };
    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf).await?;
    let port = u16::from_be_bytes(port_buf);

    let target = format!("{host}:{port}");
    match timeout(Duration::from_secs(8), TcpStream::connect(&target)).await {
        Ok(Ok(remote)) => {
            remote.set_nodelay(true).ok();
            stream.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 0]).await?;
            relay_streams(stream, remote, tracker).await?;
        }
        _ => {
            let _ = stream.write_all(&[0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
        }
    }
    Ok(())
}

async fn handle_http_proxy_client(mut stream: TcpStream, tracker: Arc<Mutex<Tracker>>) -> Result<()> {
    let mut buf = [0u8; 4096];
    let n = timeout(Duration::from_secs(6), stream.read(&mut buf)).await??;
    if n == 0 {
        return Ok(());
    }
    let text = String::from_utf8_lossy(&buf[..n]);
    let first_line = text.lines().next().unwrap_or_default();
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    if parts[0].eq_ignore_ascii_case("CONNECT") {
        let target = parts[1];
        if let Ok(Ok(remote)) = timeout(Duration::from_secs(8), TcpStream::connect(target)).await {
            remote.set_nodelay(true).ok();
            stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
            relay_streams(stream, remote, tracker).await?;
        } else {
            let _ = stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
        }
    }
    Ok(())
}

async fn spawn_embedded(args: &[String], tracker: Arc<Mutex<Tracker>>, sink: LogSink) -> Result<CoreProcess> {
    let cfg = parse_embedded_args(args);
    let exited = Arc::new(AtomicBool::new(false));
    let mut tasks = Vec::new();

    let socks_listener = TcpListener::bind(cfg.bind).await?;
    let t_socks = tracker.clone();
    tasks.push(tokio::spawn(async move {
        while let Ok((stream, _)) = socks_listener.accept().await {
            stream.set_nodelay(true).ok();
            let tr = t_socks.clone();
            tokio::spawn(async move {
                let _ = handle_socks5_client(stream, tr).await;
            });
        }
    }));

    let http_listener = TcpListener::bind(cfg.http).await?;
    let t_http = tracker.clone();
    tasks.push(tokio::spawn(async move {
        while let Ok((stream, _)) = http_listener.accept().await {
            stream.set_nodelay(true).ok();
            let tr = t_http.clone();
            tokio::spawn(async move {
                let _ = handle_http_proxy_client(stream, tr).await;
            });
        }
    }));

    if let Some(psiphon_bind) = cfg.psiphon_bind {
        let p_listener = TcpListener::bind(psiphon_bind).await?;
        let t_psiphon = tracker.clone();
        tasks.push(tokio::spawn(async move {
            while let Ok((stream, _)) = p_listener.accept().await {
                stream.set_nodelay(true).ok();
                let tr = t_psiphon.clone();
                tokio::spawn(async move {
                    let _ = handle_socks5_client(stream, tr).await;
                });
            }
        }));
    }

    if let Some(psiphon_http) = cfg.psiphon_http {
        let ph_listener = TcpListener::bind(psiphon_http).await?;
        let t_phttp = tracker.clone();
        tasks.push(tokio::spawn(async move {
            while let Ok((stream, _)) = ph_listener.accept().await {
                stream.set_nodelay(true).ok();
                let tr = t_phttp.clone();
                tokio::spawn(async move {
                    let _ = handle_http_proxy_client(stream, tr).await;
                });
            }
        }));
    }

    sink(LogLevel::Ok, format!("aether engine initialized on {}", cfg.bind));
    Ok(CoreProcess {
        backend: RuntimeBackend::Embedded { exited, tasks },
        tracker,
    })
}

impl CoreProcess {
    pub async fn spawn(binary: &Path, args: &[String], sink: LogSink) -> Result<CoreProcess> {
        let tracker = Arc::new(Mutex::new(Tracker::default()));

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        if binary.exists() {
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

            if let Ok(mut child) = command.spawn() {
                #[cfg(windows)]
                let _ = job::assign(&child);
                let pid = child.id();
                let stdin = child.stdin.take();
                if let Some(stdout) = child.stdout.take() {
                    pump(stdout, tracker.clone(), sink.clone());
                }
                if let Some(stderr) = child.stderr.take() {
                    pump(stderr, tracker.clone(), sink);
                }
                return Ok(CoreProcess {
                    backend: RuntimeBackend::External { child, stdin, pid },
                    tracker,
                });
            }
        }

        let _ = binary;
        spawn_embedded(args, tracker, sink).await
    }

    pub fn is_embedded(&self) -> bool {
        matches!(self.backend, RuntimeBackend::Embedded { .. })
    }

    pub fn stats(&self) -> Stats {
        self.tracker.lock().map(|guard| guard.stats).unwrap_or_default()
    }

    pub fn has_exited(&mut self) -> bool {
        match &mut self.backend {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            RuntimeBackend::External { child, .. } => matches!(child.try_wait(), Ok(Some(_))),
            RuntimeBackend::Embedded { exited, .. } => exited.load(Ordering::SeqCst),
        }
    }

    pub async fn terminate(&mut self) {
        match &mut self.backend {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            RuntimeBackend::External { child, stdin, pid } => {
                if let Some(mut input) = stdin.take() {
                    let _ = input.write_all(b"quit\n").await;
                    let _ = input.flush().await;
                    drop(input);
                    if timeout(Duration::from_millis(1500), child.wait()).await.is_ok() {
                        return;
                    }
                }
                #[cfg(unix)]
                if let Some(pid_val) = *pid {
                    unsafe {
                        libc::killpg(pid_val as libc::pid_t, libc::SIGTERM);
                    }
                    if timeout(Duration::from_secs(2), child.wait()).await.is_ok() {
                        return;
                    }
                    unsafe {
                        libc::killpg(pid_val as libc::pid_t, libc::SIGKILL);
                    }
                }
                let _ = pid;
                let _ = child.kill().await;
                let _ = timeout(Duration::from_secs(2), child.wait()).await;
            }
            RuntimeBackend::Embedded { exited, tasks } => {
                exited.store(true, Ordering::SeqCst);
                for task in tasks.drain(..) {
                    task.abort();
                }
            }
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