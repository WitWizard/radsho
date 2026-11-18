use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::Path;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
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

const CLEAN_CF_IPS: [Ipv4Addr; 6] = [
    Ipv4Addr::new(162, 159, 135, 42),
    Ipv4Addr::new(104, 18, 2, 161),
    Ipv4Addr::new(172, 67, 73, 163),
    Ipv4Addr::new(104, 16, 132, 229),
    Ipv4Addr::new(162, 159, 128, 7),
    Ipv4Addr::new(104, 17, 64, 4),
];

const UPSTREAM_SOURCES: [&str; 4] = [
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks5.txt",
    "https://raw.githubusercontent.com/monosans/proxy-list/main/proxies/socks5.txt",
    "https://raw.githubusercontent.com/hookzof/socks5_list/master/proxy.txt",
    "https://api.proxyscrape.com/v2/?request=getproxies&protocol=socks5&timeout=2500&country=all",
];

fn dns_cache() -> &'static Mutex<HashMap<String, IpAddr>> {
    static CACHE: OnceLock<Mutex<HashMap<String, IpAddr>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn upstream_pool() -> &'static Mutex<Vec<SocketAddr>> {
    static POOL: OnceLock<Mutex<Vec<SocketAddr>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(Vec::new()))
}

fn doh_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_default()
    })
}

fn is_poisoned_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 0
                || o[0] == 127
                || (o[0] == 10 && o[1] == 10)
                || (o[0] == 198 && (o[1] == 18 || o[1] == 19))
                || o[0] >= 240
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

fn is_cloudflare_ip(ip: IpAddr) -> bool {
    let IpAddr::V4(v4) = ip else { return false };
    let o = v4.octets();
    (o[0] == 104 && (16..=31).contains(&o[1]))
        || (o[0] == 172 && (64..=71).contains(&o[1]))
        || (o[0] == 162 && (158..=159).contains(&o[1]))
        || (o[0] == 188 && o[1] == 114 && (96..=99).contains(&o[2]))
        || (o[0] == 141 && o[1] == 101 && (64..=127).contains(&o[2]))
}

fn is_telegram_ip(ip: IpAddr) -> bool {
    let IpAddr::V4(v4) = ip else { return false };
    let o = v4.octets();
    (o[0] == 149 && o[1] == 154 && (160..=175).contains(&o[2]))
        || (o[0] == 91 && o[1] == 108 && ((4..=23).contains(&o[2]) || (56..=59).contains(&o[2])))
        || (o[0] == 95 && o[1] == 161 && (64..=79).contains(&o[2]))
        || (o[0] == 185 && o[1] == 76 && o[2] == 151)
        || (o[0] == 91 && o[1] == 105 && (192..=193).contains(&o[2]))
}

fn is_sanctioned_or_blocked_host(host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    h.ends_with("kimi.ai")
        || h.ends_with("claude.ai")
        || h.ends_with("anthropic.com")
        || h.ends_with("huggingface.co")
        || h.ends_with("openai.com")
        || h.ends_with("chatgpt.com")
        || h.ends_with("telegram.org")
        || h.ends_with("t.me")
        || h.ends_with("telegra.ph")
}

async fn resolve_domain_doh(domain: &str) -> Option<IpAddr> {
    if let Ok(ip) = domain.parse::<IpAddr>() {
        return Some(ip);
    }
    if let Ok(guard) = dns_cache().lock() {
        if let Some(&cached) = guard.get(domain) {
            return Some(cached);
        }
    }
    let lower = domain.to_ascii_lowercase();
    if lower == "www.cloudflare.com" || lower == "cloudflare.com" || lower == "speed.cloudflare.com" {
        return Some(IpAddr::V4(CLEAN_CF_IPS[0]));
    }
    let endpoints = [
        format!("https://1.1.1.1/dns-query?name={domain}&type=A"),
        format!("https://1.0.0.1/dns-query?name={domain}&type=A"),
        format!("https://8.8.8.8/resolve?name={domain}&type=A"),
    ];
    for url in endpoints {
        let Ok(resp) = doh_client()
            .get(&url)
            .header("Accept", "application/dns-json")
            .send()
            .await
        else {
            continue;
        };
        let Ok(json) = resp.json::<serde_json::Value>().await else {
            continue;
        };
        if let Some(answers) = json.get("Answer").and_then(|a| a.as_array()) {
            for ans in answers {
                if ans.get("type").and_then(|t| t.as_u64()) == Some(1) {
                    if let Some(data) = ans.get("data").and_then(|d| d.as_str()) {
                        if let Ok(ip) = data.trim().parse::<IpAddr>() {
                            if !is_poisoned_ip(ip) {
                                let final_ip = if is_cloudflare_ip(ip) {
                                    IpAddr::V4(CLEAN_CF_IPS[0])
                                } else {
                                    ip
                                };
                                if let Ok(mut guard) = dns_cache().lock() {
                                    guard.insert(domain.to_string(), final_ip);
                                }
                                return Some(final_ip);
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(mut addrs) = tokio::net::lookup_host((domain, 443)).await {
        while let Some(sock) = addrs.next() {
            if !is_poisoned_ip(sock.ip()) {
                return Some(sock.ip());
            }
        }
    }
    None
}

async fn resolve_dns_wire_packet(packet: &[u8]) -> Option<Vec<u8>> {
    for doh_url in ["https://1.1.1.1/dns-query", "https://1.0.0.1/dns-query"] {
        if let Ok(resp) = doh_client()
            .post(doh_url)
            .header("Content-Type", "application/dns-message")
            .header("Accept", "application/dns-message")
            .body(packet.to_vec())
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(bytes) = resp.bytes().await {
                    if bytes.len() >= 12 {
                        return Some(bytes.to_vec());
                    }
                }
            }
        }
    }
    for tcp_dns in ["1.1.1.1:53", "8.8.8.8:53"] {
        if let Ok(Ok(mut s)) = timeout(Duration::from_secs(3), TcpStream::connect(tcp_dns)).await {
            let len_bytes = (packet.len() as u16).to_be_bytes();
            if s.write_all(&len_bytes).await.is_ok() && s.write_all(packet).await.is_ok() {
                let mut rlen = [0u8; 2];
                if timeout(Duration::from_secs(3), s.read_exact(&mut rlen)).await.is_ok() {
                    let size = u16::from_be_bytes(rlen) as usize;
                    let mut out = vec![0u8; size];
                    if timeout(Duration::from_secs(3), s.read_exact(&mut out)).await.is_ok() {
                        return Some(out);
                    }
                }
            }
        }
    }
    None
}

async fn connect_via_upstream_socks5(proxy: SocketAddr, host: &str, port: u16) -> Result<TcpStream> {
    let mut s = timeout(Duration::from_millis(2400), TcpStream::connect(proxy)).await??;
    s.set_nodelay(true).ok();
    s.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut greeting = [0u8; 2];
    timeout(Duration::from_millis(2000), s.read_exact(&mut greeting)).await??;
    if greeting[0] != 0x05 || greeting[1] != 0x00 {
        bail!("upstream socks5 auth rejected");
    }
    let mut req = Vec::with_capacity(7 + host.len());
    req.extend_from_slice(&[0x05, 0x01, 0x00]);
    if let Ok(ipv4) = host.parse::<Ipv4Addr>() {
        req.push(0x01);
        req.extend_from_slice(&ipv4.octets());
    } else {
        req.push(0x03);
        req.push(host.len() as u8);
        req.extend_from_slice(host.as_bytes());
    }
    req.extend_from_slice(&port.to_be_bytes());
    s.write_all(&req).await?;
    let mut head = [0u8; 4];
    timeout(Duration::from_millis(3000), s.read_exact(&mut head)).await??;
    if head[0] != 0x05 || head[1] != 0x00 {
        bail!("upstream socks5 connect failed");
    }
    let skip = match head[3] {
        0x01 => 6,
        0x04 => 18,
        0x03 => {
            let mut l = [0u8; 1];
            s.read_exact(&mut l).await?;
            l[0] as usize + 2
        }
        _ => bail!("invalid atyp"),
    };
    let mut discard = vec![0u8; skip];
    s.read_exact(&mut discard).await?;
    Ok(s)
}

async fn verify_candidate_proxy(addr: SocketAddr) -> bool {
    if let Ok(mut stream) = connect_via_upstream_socks5(addr, "149.154.167.50", 443).await {
        let _ = stream.shutdown().await;
        return true;
    }
    if let Ok(mut stream) = connect_via_upstream_socks5(addr, "www.cloudflare.com", 443).await {
        let _ = stream.shutdown().await;
        return true;
    }
    false
}

async fn refresh_upstream_pool() {
    let mut candidates: Vec<SocketAddr> = Vec::new();
    for url in UPSTREAM_SOURCES {
        if let Ok(resp) = doh_client().get(url).send().await {
            if let Ok(text) = resp.text().await {
                for line in text.lines().take(160) {
                    if let Ok(addr) = line.trim().parse::<SocketAddr>() {
                        if !candidates.contains(&addr) {
                            candidates.push(addr);
                        }
                    }
                }
            }
        }
        if candidates.len() >= 120 {
            break;
        }
    }
    let mut handles = Vec::new();
    for addr in candidates.into_iter().take(80) {
        handles.push(tokio::spawn(async move {
            if verify_candidate_proxy(addr).await {
                Some(addr)
            } else {
                None
            }
        }));
    }
    let mut verified = Vec::new();
    for h in handles {
        if let Ok(Some(addr)) = h.await {
            verified.push(addr);
            if verified.len() >= 16 {
                break;
            }
        }
    }
    if !verified.is_empty() {
        if let Ok(mut pool) = upstream_pool().lock() {
            *pool = verified;
        }
    }
}

async fn connect_smart(host: &str, port: u16) -> Result<TcpStream> {
    let parsed_ip = host.parse::<IpAddr>().ok();
    let needs_upstream = parsed_ip.map(is_telegram_ip).unwrap_or(false) || is_sanctioned_or_blocked_host(host);

    let pool_snapshot: Vec<SocketAddr> = upstream_pool()
        .lock()
        .map(|g| g.iter().copied().take(5).collect())
        .unwrap_or_default();

    if needs_upstream && !pool_snapshot.is_empty() {
        for proxy in &pool_snapshot {
            if let Ok(s) = connect_via_upstream_socks5(*proxy, host, port).await {
                return Ok(s);
            }
        }
    }

    if let Some(ip) = resolve_domain_doh(host).await {
        if is_telegram_ip(ip) && !pool_snapshot.is_empty() {
            for proxy in &pool_snapshot {
                if let Ok(s) = connect_via_upstream_socks5(*proxy, host, port).await {
                    return Ok(s);
                }
            }
        }
        let target = SocketAddr::new(ip, port);
        if let Ok(Ok(s)) = timeout(Duration::from_millis(3200), TcpStream::connect(target)).await {
            s.set_nodelay(true).ok();
            return Ok(s);
        }
        if is_cloudflare_ip(ip) {
            for cf_ip in CLEAN_CF_IPS {
                let alt = SocketAddr::new(IpAddr::V4(cf_ip), port);
                if let Ok(Ok(s)) = timeout(Duration::from_millis(2200), TcpStream::connect(alt)).await {
                    s.set_nodelay(true).ok();
                    return Ok(s);
                }
            }
        }
    }

    for proxy in &pool_snapshot {
        if let Ok(s) = connect_via_upstream_socks5(*proxy, host, port).await {
            return Ok(s);
        }
    }

    let fallback_target = format!("{host}:{port}");
    let s = timeout(Duration::from_secs(4), TcpStream::connect(&fallback_target)).await??;
    s.set_nodelay(true).ok();
    Ok(s)
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

async fn write_tls_fragmented(remote: &mut TcpStream, data: &[u8]) -> Result<()> {
    if data.len() > 5 && data[0] == 0x16 && data[1] == 0x03 {
        let rec_len = u16::from_be_bytes([data[3], data[4]]) as usize;
        if 5 + rec_len == data.len() && rec_len > 8 {
            let payload = &data[5..];
            let sizes = [2usize, 14, 21, 29, 43];
            let mut offset = 0usize;
            let mut idx = 0usize;
            while offset < payload.len() {
                let step = if idx < sizes.len() {
                    sizes[idx].min(payload.len() - offset)
                } else {
                    payload.len() - offset
                };
                let chunk = &payload[offset..offset + step];
                let len_bytes = (chunk.len() as u16).to_be_bytes();
                let mut frame = Vec::with_capacity(5 + chunk.len());
                frame.extend_from_slice(&[0x16, data[1], data[2], len_bytes[0], len_bytes[1]]);
                frame.extend_from_slice(chunk);
                remote.write_all(&frame).await?;
                remote.flush().await?;
                offset += step;
                idx += 1;
                if offset < payload.len() {
                    sleep(Duration::from_millis(6)).await;
                }
            }
            return Ok(());
        }
    }
    if data.len() > 24 {
        let split = 5.min(data.len() / 2);
        remote.write_all(&data[..split]).await?;
        remote.flush().await?;
        sleep(Duration::from_millis(6)).await;
        remote.write_all(&data[split..]).await?;
        remote.flush().await?;
    } else {
        remote.write_all(data).await?;
        remote.flush().await?;
    }
    Ok(())
}

async fn relay_streams(mut client: TcpStream, mut remote: TcpStream, tracker: Arc<Mutex<Tracker>>) -> Result<()> {
    let mut first = [0u8; 4096];
    if let Ok(Ok(n)) = timeout(Duration::from_millis(250), client.read(&mut first)).await {
        if n > 0 {
            write_tls_fragmented(&mut remote, &first[..n]).await?;
            add_traffic(&tracker, 0, n as u64);
        }
    }
    if let Ok((up, down)) = tokio::io::copy_bidirectional(&mut client, &mut remote).await {
        add_traffic(&tracker, down, up);
    }
    Ok(())
}

async fn handle_hev_udp_in_tcp(mut stream: TcpStream, tracker: Arc<Mutex<Tracker>>) -> Result<()> {
    stream.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 0]).await?;
    loop {
        let mut hdr = [0u8; 3];
        if timeout(Duration::from_secs(30), stream.read_exact(&mut hdr)).await.is_err() {
            break;
        }
        let datalen = u16::from_be_bytes([hdr[0], hdr[1]]) as usize;
        let atyp = hdr[2];
        let mut addr_bytes = Vec::new();
        let port: u16;
        match atyp {
            0x01 => {
                let mut b = [0u8; 6];
                stream.read_exact(&mut b).await?;
                addr_bytes.extend_from_slice(&b);
                port = u16::from_be_bytes([b[4], b[5]]);
            }
            0x03 => {
                let mut l = [0u8; 1];
                stream.read_exact(&mut l).await?;
                let mut b = vec![0u8; l[0] as usize + 2];
                stream.read_exact(&mut b).await?;
                addr_bytes.push(l[0]);
                addr_bytes.extend_from_slice(&b);
                port = u16::from_be_bytes([b[b.len() - 2], b[b.len() - 1]]);
            }
            0x04 => {
                let mut b = [0u8; 18];
                stream.read_exact(&mut b).await?;
                addr_bytes.extend_from_slice(&b);
                port = u16::from_be_bytes([b[16], b[17]]);
            }
            _ => break,
        }
        let mut payload = vec![0u8; datalen];
        stream.read_exact(&mut payload).await?;
        add_traffic(&tracker, 0, datalen as u64);
        if port == 53 {
            if let Some(ans) = resolve_dns_wire_packet(&payload).await {
                let rlen = (ans.len() as u16).to_be_bytes();
                let mut reply = Vec::with_capacity(3 + addr_bytes.len() + ans.len());
                reply.extend_from_slice(&[rlen[0], rlen[1], atyp]);
                reply.extend_from_slice(&addr_bytes);
                reply.extend_from_slice(&ans);
                stream.write_all(&reply).await?;
                add_traffic(&tracker, ans.len() as u64, 0);
            }
        }
    }
    Ok(())
}

async fn handle_socks5_udp_associate(mut stream: TcpStream, tracker: Arc<Mutex<Tracker>>) -> Result<()> {
    let udp = UdpSocket::bind("127.0.0.1:0").await?;
    let local_port = udp.local_addr()?.port().to_be_bytes();
    stream
        .write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, local_port[0], local_port[1]])
        .await?;
    let mut buf = [0u8; 4096];
    loop {
        tokio::select! {
            res = udp.recv_from(&mut buf) => {
                let Ok((n, client_addr)) = res else { break };
                if n < 10 || buf[2] != 0x00 { continue; }
                let atyp = buf[3];
                let (hdr_len, port) = match atyp {
                    0x01 if n >= 10 => (10usize, u16::from_be_bytes([buf[8], buf[9]])),
                    0x03 if n >= 7 => {
                        let dlen = buf[4] as usize;
                        if n < 7 + dlen { continue; }
                        (7 + dlen, u16::from_be_bytes([buf[5 + dlen], buf[6 + dlen]]))
                    }
                    0x04 if n >= 22 => (22usize, u16::from_be_bytes([buf[20], buf[21]])),
                    _ => continue,
                };
                let payload = &buf[hdr_len..n];
                add_traffic(&tracker, 0, payload.len() as u64);
                if port == 53 {
                    if let Some(ans) = resolve_dns_wire_packet(payload).await {
                        let mut reply = Vec::with_capacity(hdr_len + ans.len());
                        reply.extend_from_slice(&buf[..hdr_len]);
                        reply.extend_from_slice(&ans);
                        let _ = udp.send_to(&reply, client_addr).await;
                        add_traffic(&tracker, ans.len() as u64, 0);
                    }
                }
            }
            r = stream.read(&mut [0u8; 1]) => {
                if r.unwrap_or(0) == 0 { break; }
            }
        }
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
    if req[0] != 0x05 {
        return Ok(());
    }
    let cmd = req[1];
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
            Ipv6Addr::from(ip).to_string()
        }
        _ => return Ok(()),
    };
    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf).await?;
    let port = u16::from_be_bytes(port_buf);

    if cmd == 0x04 {
        return handle_hev_udp_in_tcp(stream, tracker).await;
    }
    if cmd == 0x03 {
        return handle_socks5_udp_associate(stream, tracker).await;
    }
    if cmd != 0x01 {
        let _ = stream.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
        return Ok(());
    }

    match connect_smart(&host, port).await {
        Ok(remote) => {
            stream.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 0]).await?;
            relay_streams(stream, remote, tracker).await?;
        }
        Err(_) => {
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
        let (host, port) = match parts[1].rsplit_once(':') {
            Some((h, p)) => (h, p.parse::<u16>().unwrap_or(443)),
            None => (parts[1], 443),
        };
        if let Ok(remote) = connect_smart(host, port).await {
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

    tasks.push(tokio::spawn(async move {
        loop {
            refresh_upstream_pool().await;
            sleep(Duration::from_secs(180)).await;
        }
    }));

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