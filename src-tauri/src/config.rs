use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SANCTIONED_LOCS: [&str; 4] = ["IR", "KP", "SY", "CU"];
pub const SANCTION_SITES: [&str; 3] = ["www.kimi.ai", "claude.ai", "huggingface.co"];
pub const TRACE_HOSTS: [&str; 3] = ["www.cloudflare.com", "cloudflare.com", "1.1.1.1"];
pub const TRACE_PATH: &str = "/cdn-cgi/trace";
pub const BLOCKED_STATUSES: [u16; 2] = [403, 451];
pub const CREATORS: [&str; 2] = ["Lilmmd", "LFnighTT"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Transport {
    MasqueH3,
    MasqueH2,
    Wireguard,
    Gool,
    GoolClassic,
    Mim,
}

impl Transport {
    pub fn is_nested(self) -> bool {
        matches!(self, Transport::Gool | Transport::GoolClassic | Transport::Mim)
    }

    pub fn argv(self) -> &'static [&'static str] {
        match self {
            Transport::MasqueH3 => &["--masque"],
            Transport::MasqueH2 => &["--masque", "--h2"],
            Transport::Wireguard => &["--wg"],
            Transport::Gool => &["--gool"],
            Transport::GoolClassic => &["--gool-classic"],
            Transport::Mim => &["--mim"],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScanMode {
    Default,
    Turbo,
    Balanced,
    Thorough,
    Verified,
    Ironclad,
}

impl ScanMode {
    pub fn flag_value(self) -> &'static str {
        match self {
            ScanMode::Default => "default",
            ScanMode::Turbo => "turbo",
            ScanMode::Balanced => "balanced",
            ScanMode::Thorough => "thorough",
            ScanMode::Verified => "verified",
            ScanMode::Ironclad => "ironclad",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Fa,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Unblock,
    Resilient,
    Fastest,
    Antidpi,
    Lowdata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ports {
    pub bind: u16,
    pub http: u16,
    pub psiphon: u16,
    pub psiphon_http: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShuffleSettings {
    pub enabled: bool,
    pub interval_min: u64,
    pub regions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingSettings {
    pub enabled: bool,
    pub direct: Vec<String>,
    pub block: Vec<String>,
    pub domestic_bypass: bool,
    pub apps: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolSettings {
    pub auto: bool,
    pub preset: Preset,
    pub chain: Vec<Transport>,
    pub fragment: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: ThemeMode,
    pub lang: Lang,
    pub transport: Transport,
    pub scan_mode: ScanMode,
    pub ports: Ports,
    pub dns: String,
    pub extra_args: String,
    pub system_proxy: bool,
    pub minimize_to_tray: bool,
    pub auto_update: bool,
    pub psiphon: bool,
    pub psiphon_region: String,
    pub sanction_check: bool,
    pub shuffle: ShuffleSettings,
    pub routing: RoutingSettings,
    pub protocol: ProtocolSettings,
    pub remembered_transport: Option<Transport>,
    pub server_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectivePorts {
    pub socks: u16,
    pub http: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: ThemeMode::System,
            lang: Lang::En,
            transport: Transport::MasqueH3,
            scan_mode: ScanMode::Verified,
            ports: Ports {
                bind: 1819,
                http: 1820,
                psiphon: 1821,
                psiphon_http: 1822,
            },
            dns: "1.1.1.1".to_string(),
            extra_args: String::new(),
            system_proxy: true,
            minimize_to_tray: true,
            auto_update: true,
            psiphon: true,
            psiphon_region: "DE".to_string(),
            sanction_check: true,
            shuffle: ShuffleSettings {
                enabled: false,
                interval_min: 10,
                regions: ["DE", "NL", "FR", "AT", "IT", "ES", "CH", "SE", "GB", "US"]
                    .iter()
                    .map(|region| region.to_string())
                    .collect(),
            },
            routing: RoutingSettings {
                enabled: true,
                direct: Vec::new(),
                block: Vec::new(),
                domestic_bypass: true,
                apps: Vec::new(),
            },
            protocol: ProtocolSettings {
                auto: true,
                preset: Preset::Unblock,
                chain: preset_chain(Preset::Unblock),
                fragment: false,
            },
            remembered_transport: None,
            server_id: "frankfurt".to_string(),
        }
    }
}

pub fn preset_chain(preset: Preset) -> Vec<Transport> {
    use Transport::*;
    match preset {
        Preset::Unblock => vec![MasqueH3, MasqueH2, Wireguard, Gool, GoolClassic, Mim],
        Preset::Resilient => vec![MasqueH2, MasqueH3, Wireguard, Gool, Mim, GoolClassic],
        Preset::Fastest => vec![MasqueH3, Wireguard, MasqueH2, Gool, Mim, GoolClassic],
        Preset::Antidpi => vec![MasqueH2, Gool, Mim, GoolClassic, MasqueH3, Wireguard],
        Preset::Lowdata => vec![Wireguard, MasqueH3, MasqueH2, Gool, GoolClassic, Mim],
    }
}

impl Settings {
    pub fn effective_ports(&self) -> EffectivePorts {
        if self.psiphon {
            EffectivePorts {
                socks: self.ports.psiphon,
                http: self.ports.psiphon_http,
            }
        } else {
            EffectivePorts {
                socks: self.ports.bind,
                http: self.ports.http,
            }
        }
    }

    pub fn ports_valid(&self) -> bool {
        let values = [self.ports.bind, self.ports.http, self.ports.psiphon, self.ports.psiphon_http];
        let unique: HashSet<u16> = values.iter().copied().collect();
        values.iter().all(|port| *port >= 1024) && unique.len() == values.len()
    }

    pub fn build_chain(&self) -> Vec<Transport> {
        let base: Vec<Transport> = if self.protocol.auto {
            self.protocol.chain.clone()
        } else {
            vec![self.transport]
        };
        let mut merged: Vec<Transport> = Vec::new();
        if self.protocol.auto {
            if let Some(remembered) = self.remembered_transport {
                merged.push(remembered);
            }
        }
        for transport in base {
            if !merged.contains(&transport) {
                merged.push(transport);
            }
        }
        if self.psiphon {
            return merged;
        }
        let (nested, plain): (Vec<Transport>, Vec<Transport>) = merged.into_iter().partition(|transport| transport.is_nested());
        nested.into_iter().chain(plain).collect()
    }

    pub fn build_core_args(&self, transport: Transport, region: &str) -> Vec<String> {
        let config_file = std::env::temp_dir().join("radsho-aether.toml");
        if !config_file.exists() {
            let _ = fs::File::create(&config_file);
        }
        let mut args: Vec<String> = vec![
            "--bind".into(),
            format!("127.0.0.1:{}", self.ports.bind),
            "--config".into(),
            config_file.to_string_lossy().to_string(),
            "--log-level".into(),
            "info".into(),
            "--stats".into(),
            "--stats-secs".into(),
            "1".into(),
        ];
        args.extend(transport.argv().iter().map(|flag| flag.to_string()));
        let dns = self.dns.trim();
        if !dns.is_empty() {
            args.extend(["--dns".to_string(), dns.to_string()]);
        }
        args.extend(["--ip".to_string(), "v4".to_string()]);
        if !self.psiphon {
            args.extend(["--exit-loc".to_string(), "!IR".to_string()]);
        }
        if self.routing.enabled {
            if self.routing.domestic_bypass {
                args.extend(["--route-direct".to_string(), ".ir".to_string()]);
            }
            for domain in &self.routing.direct {
                args.extend(["--route-direct".to_string(), domain.clone()]);
            }
            for domain in &self.routing.block {
                args.extend(["--route-block".to_string(), domain.clone()]);
            }
        }
        args.extend([
            "--reconnect-secs".to_string(),
            "2".to_string(),
            "--quick-reconnect".to_string(),
            "--keepalive".to_string(),
            "5".to_string(),
        ]);
        if self.protocol.fragment {
            args.push("--fragment".into());
        }
        if self.scan_mode != ScanMode::Default {
            args.extend(["--scan".to_string(), self.scan_mode.flag_value().to_string()]);
        }
        if self.psiphon {
            args.extend([
                "--psiphon".to_string(),
                "--psiphon-bind".to_string(),
                format!("127.0.0.1:{}", self.ports.psiphon),
                "--psiphon-http".to_string(),
                format!("127.0.0.1:{}", self.ports.psiphon_http),
                "--psiphon-region".to_string(),
                region.to_string(),
            ]);
        }
        args.extend([
            "--http-proxy".to_string(),
            format!("127.0.0.1:{}", self.ports.http),
        ]);
        args.extend(self.extra_args.split_whitespace().map(|part| part.to_string()));
        args
    }

    pub fn load(dir: &Path) -> Settings {
        fs::read_to_string(settings_path(dir))
            .ok()
            .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        let raw = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(settings_path(dir), raw)
    }
}

pub fn settings_path(dir: &Path) -> PathBuf {
    dir.join("settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psiphon_mode_never_passes_exit_loc() {
        let settings = Settings::default();
        for transport in settings.build_chain() {
            let args = settings.build_core_args(transport, "DE");
            assert!(!args.iter().any(|arg| arg == "--exit-loc"));
            assert!(args.iter().any(|arg| arg == "--psiphon"));
            assert!(args.iter().any(|arg| arg == "--config"));
            assert!(args.windows(2).any(|pair| pair[0] == "--psiphon-region" && pair[1] == "DE"));
            assert!(args.windows(2).any(|pair| pair[0] == "--scan" && pair[1] == "verified"));
        }
    }

    #[test]
    fn direct_mode_passes_exit_loc_and_promotes_nested() {
        let mut settings = Settings::default();
        settings.psiphon = false;
        let chain = settings.build_chain();
        assert!(chain[0].is_nested());
        let args = settings.build_core_args(chain[0], "DE");
        assert!(args.windows(2).any(|pair| pair[0] == "--exit-loc" && pair[1] == "!IR"));
        assert!(!args.iter().any(|arg| arg == "--psiphon"));
    }

    #[test]
    fn effective_ports_follow_mode() {
        let mut settings = Settings::default();
        assert_eq!(settings.effective_ports(), EffectivePorts { socks: 1821, http: 1822 });
        settings.psiphon = false;
        assert_eq!(settings.effective_ports(), EffectivePorts { socks: 1819, http: 1820 });
    }

    #[test]
    fn transport_serialises_in_screaming_case() {
        let raw = serde_json::to_string(&Transport::MasqueH3).unwrap();
        assert_eq!(raw, "\"MASQUE_H3\"");
        let raw = serde_json::to_string(&Transport::GoolClassic).unwrap();
        assert_eq!(raw, "\"GOOL_CLASSIC\"");
    }
}