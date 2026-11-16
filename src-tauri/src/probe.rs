use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use rustls::ClientConfig;
use rustls::RootCertStore;
use rustls::pki_types::ServerName;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tokio_rustls::TlsConnector;

use crate::config::{BLOCKED_STATUSES, SANCTION_SITES, SANCTIONED_LOCS, TRACE_HOSTS, TRACE_PATH};

const IO_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RESPONSE_BYTES: usize = 256 * 1024;
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Trace {
    pub ip: String,
    pub loc: String,
    pub colo: String,
    pub warp: String,
}

impl Trace {
    pub fn is_sanctioned(&self) -> bool {
        SANCTIONED_LOCS.iter().any(|loc| loc.eq_ignore_ascii_case(&self.loc))
    }
}

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteVerdict {
    pub site: String,
    pub status: u16,
    pub blocked: bool,
    pub reachable: bool,
}

fn tls_connector() -> &'static TlsConnector {
    static CONNECTOR: OnceLock<TlsConnector> = OnceLock::new();
    CONNECTOR.get_or_init(|| {
        let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("tls protocol versions")
            .with_root_certificates(roots)
            .with_no_client_auth();
        TlsConnector::from(Arc::new(config))
    })
}

pub async fn wait_for_listener(addr: SocketAddr, limit: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < limit {
        if let Ok(Ok(_)) = timeout(Duration::from_millis(400), TcpStream::connect(addr)).await {
            return true;
        }
        sleep(Duration::from_millis(180)).await;
    }
    false
}

pub async fn socks5_connect(proxy: SocketAddr, host: &str, port: u16) -> Result<TcpStream> {
    if host.is_empty() || host.len() > 255 {
        bail!("invalid socks5 target host length");
    }
    let mut stream = timeout(IO_TIMEOUT, TcpStream::connect(proxy))
        .await
        .context("socks5 proxy connect timed out")?
        .context("socks5 proxy unreachable")?;
    stream.set_nodelay(true).ok();

    stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut method = [0u8; 2];
    timeout(IO_TIMEOUT, stream.read_exact(&mut method)).await.context("socks5 greeting timed out")??;
    if method[0] != 0x05 || method[1] != 0x00 {
        bail!("socks5 server refused no-auth method");
    }

    let mut request: Vec<u8> = vec![0x05, 0x01, 0x00, 0x03, host.len() as u8];
    request.extend_from_slice(host.as_bytes());
    request.extend_from_slice(&port.to_be_bytes());
    stream.write_all(&request).await?;

    let mut head = [0u8; 4];
    timeout(IO_TIMEOUT, stream.read_exact(&mut head)).await.context("socks5 reply timed out")??;
    if head[0] != 0x05 {
        bail!("socks5 reply has invalid version");
    }
    if head[1] != 0x00 {
        bail!("socks5 connect failed: {}", socks5_reply_text(head[1]));
    }
    let remaining = match head[3] {
        0x01 => 4 + 2,
        0x04 => 16 + 2,
        0x03 => {
            let mut length = [0u8; 1];
            stream.read_exact(&mut length).await?;
            length[0] as usize + 2
        }
        other => bail!("socks5 reply has unknown address type {other}"),
    };
    let mut discard = vec![0u8; remaining];
    stream.read_exact(&mut discard).await?;
    Ok(stream)
}

fn socks5_reply_text(code: u8) -> &'static str {
    match code {
        0x01 => "general failure",
        0x02 => "connection not allowed",
        0x03 => "network unreachable",
        0x04 => "host unreachable",
        0x05 => "connection refused",
        0x06 => "ttl expired",
        0x07 => "command not supported",
        0x08 => "address type not supported",
        _ => "unknown error",
    }
}

pub async fn https_get(proxy: SocketAddr, host: &str, path: &str) -> Result<HttpResponse> {
    let tcp = socks5_connect(proxy, host, 443).await?;
    let sni_host = if host == "1.1.1.1" { "cloudflare-dns.com" } else { host };
    let server_name = ServerName::try_from(sni_host.to_string()).map_err(|_| anyhow!("invalid tls server name {host}"))?;
    let mut tls = timeout(IO_TIMEOUT, tls_connector().connect(server_name, tcp))
        .await
        .context("tls handshake timed out")?
        .context("tls handshake failed")?;
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {BROWSER_UA}\r\nAccept: text/html,application/xhtml+xml,*/*\r\nAccept-Language: en-US,en;q=0.9\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n"
    );
    tls.write_all(request.as_bytes()).await?;
    let mut raw: Vec<u8> = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    loop {
        let read = match timeout(IO_TIMEOUT, tls.read(&mut chunk)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(count)) => count,
            Ok(Err(_)) => break,
            Err(_) => {
                if !raw.is_empty() {
                    break;
                }
                bail!("response read timed out");
            }
        };
        raw.extend_from_slice(&chunk[..read]);
        if raw.len() >= MAX_RESPONSE_BYTES {
            break;
        }
        if has_complete_head_only(&raw) {
            break;
        }
    }
    parse_http_response(&raw)
}

fn has_complete_head_only(raw: &[u8]) -> bool {
    let text = String::from_utf8_lossy(raw);
    match text.find("\r\n\r\n") {
        Some(index) => {
            let head = text[..index].to_ascii_lowercase();
            head.contains("content-length: 0") && text.len() >= index + 4
        }
        None => false,
    }
}

pub fn parse_http_response(raw: &[u8]) -> Result<HttpResponse> {
    let text = String::from_utf8_lossy(raw).to_string();
    let split = text.find("\r\n\r\n").ok_or_else(|| anyhow!("malformed http response"))?;
    let head = &text[..split];
    let body = &text[split + 4..];
    let status_line = head.lines().next().ok_or_else(|| anyhow!("missing http status line"))?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| anyhow!("invalid http status line"))?;
    let chunked = head.to_ascii_lowercase().contains("transfer-encoding: chunked");
    let body = if chunked { decode_chunked(body) } else { body.to_string() };
    Ok(HttpResponse { status, body })
}

fn decode_chunked(body: &str) -> String {
    let mut output = String::new();
    let mut rest = body;
    loop {
        let Some(line_end) = rest.find("\r\n") else { break };
        let size_text = rest[..line_end].split(';').next().unwrap_or("0").trim();
        let Ok(size) = usize::from_str_radix(size_text, 16) else { break };
        if size == 0 {
            break;
        }
        let start = line_end + 2;
        let end = start + size;
        if end > rest.len() {
            output.push_str(&rest[start..]);
            break;
        }
        output.push_str(&rest[start..end]);
        rest = rest.get(end + 2..).unwrap_or("");
    }
    output
}

pub fn parse_trace(body: &str) -> Option<Trace> {
    let mut trace = Trace::default();
    for line in body.lines() {
        let Some((key, value)) = line.split_once('=') else { continue };
        match key.trim() {
            "ip" => trace.ip = value.trim().to_string(),
            "loc" => trace.loc = value.trim().to_string(),
            "colo" => trace.colo = value.trim().to_string(),
            "warp" => trace.warp = value.trim().to_string(),
            _ => {}
        }
    }
    if trace.ip.is_empty() || trace.loc.is_empty() {
        None
    } else {
        Some(trace)
    }
}

pub async fn trace_via(proxy: SocketAddr) -> Result<Trace> {
    let mut last_error = anyhow!("no trace endpoint attempted");
    for host in TRACE_HOSTS {
        match https_get(proxy, host, TRACE_PATH).await {
            Ok(response) if response.status == 200 => match parse_trace(&response.body) {
                Some(trace) => return Ok(trace),
                None => last_error = anyhow!("{host} returned an unparseable trace"),
            },
            Ok(response) => last_error = anyhow!("{host} returned http {}", response.status),
            Err(error) => last_error = error.context(format!("trace via {host} failed")),
        }
    }
    Err(last_error)
}

pub async fn wait_for_tunnel(proxy: SocketAddr, budget: Duration) -> Result<Trace> {
    let deadline = Instant::now() + budget;
    let mut last_error = anyhow!("tunnel readiness budget expired");
    while Instant::now() < deadline {
        match trace_via(proxy).await {
            Ok(trace) => return Ok(trace),
            Err(error) => {
                last_error = error;
                sleep(Duration::from_millis(900)).await;
            }
        }
    }
    Err(last_error)
}

pub async fn check_site(proxy: SocketAddr, site: &str) -> SiteVerdict {
    match https_get(proxy, site, "/").await {
        Ok(response) => {
            let blocked = BLOCKED_STATUSES.contains(&response.status);
            let reachable = response.status >= 200 && response.status < 500 && !blocked;
            SiteVerdict {
                site: site.to_string(),
                status: response.status,
                blocked,
                reachable,
            }
        }
        Err(_) => SiteVerdict {
            site: site.to_string(),
            status: 0,
            blocked: false,
            reachable: false,
        },
    }
}

pub async fn check_sanction_sites(proxy: SocketAddr) -> Vec<SiteVerdict> {
    let mut verdicts = Vec::with_capacity(SANCTION_SITES.len());
    for site in SANCTION_SITES {
        verdicts.push(check_site(proxy, site).await);
    }
    verdicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cloudflare_trace() {
        let body = "fl=1\nip=82.165.110.196\nloc=DE\ncolo=FRA\nwarp=on\n";
        let trace = parse_trace(body).unwrap();
        assert_eq!(trace.ip, "82.165.110.196");
        assert_eq!(trace.loc, "DE");
        assert_eq!(trace.colo, "FRA");
        assert_eq!(trace.warp, "on");
        assert!(!trace.is_sanctioned());
    }

    #[test]
    fn flags_sanctioned_exit() {
        let body = "ip=104.28.1.2\nloc=IR\ncolo=IKA\nwarp=on\n";
        assert!(parse_trace(body).unwrap().is_sanctioned());
    }

    #[test]
    fn decodes_chunked_body() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nip=1.\r\n4\r\n2.3.\r\n0\r\n\r\n";
        let response = parse_http_response(raw).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "ip=1.2.3.");
    }
}