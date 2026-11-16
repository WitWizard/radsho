use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::time::timeout;

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

pub struct CoreProcess {
    child: Child,
    stdin: Option<ChildStdin>,
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

#[cfg(not(target_os = "ios"))]
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

#[cfg(not(target_os = "ios"))]
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

impl CoreProcess {
    #[cfg(not(target_os = "ios"))]
    pub async fn spawn(binary: &Path, args: &[String], sink: LogSink) -> Result<CoreProcess> {
        if !binary.exists() {
            bail!("aether core binary not found at {}", binary.display());
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
        Ok(CoreProcess { child, stdin, tracker, pid })
    }

    #[cfg(target_os = "ios")]
    pub async fn spawn(_binary: &Path, _args: &[String], _sink: LogSink) -> Result<CoreProcess> {
        bail!("spawning child processes is not permitted on iOS; the core runs inside the packet tunnel extension")
    }

    pub fn stats(&self) -> Stats {
        self.tracker.lock().map(|guard| guard.stats).unwrap_or_default()
    }

    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    pub async fn terminate(&mut self) {
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.write_all(b"quit\n").await;
            let _ = stdin.flush().await;
            drop(stdin);
            if timeout(Duration::from_millis(1500), self.child.wait()).await.is_ok() {
                return;
            }
        }
        #[cfg(unix)]
        if let Some(pid) = self.pid {
            unsafe {
                libc::killpg(pid as libc::pid_t, libc::SIGTERM);
            }
            if timeout(Duration::from_secs(2), self.child.wait()).await.is_ok() {
                return;
            }
            unsafe {
                libc::killpg(pid as libc::pid_t, libc::SIGKILL);
            }
        }
        let _ = self.pid;
        let _ = self.child.kill().await;
        let _ = timeout(Duration::from_secs(2), self.child.wait()).await;
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