use std::path::{Path, PathBuf};
#[cfg(any(target_os = "android", target_os = "ios"))]
use std::sync::OnceLock;

use anyhow::Result;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use anyhow::Context;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use serde::{Deserialize, Serialize};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::fs::{self, File};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::io::Write;

use crate::config::{EffectivePorts, RoutingSettings};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyLock {
    pub previous_enable: Option<u32>,
    pub previous_server: Option<String>,
    pub previous_override: Option<String>,
    pub previous_mode: Option<String>,
    pub services: Vec<String>,
    pub applied_port: u16,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub trait TunBridge: Send + Sync {
    fn start(&self, ports: EffectivePorts, routing: &RoutingSettings, dns: &str) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

#[cfg(any(target_os = "android", target_os = "ios"))]
static BRIDGE: OnceLock<Box<dyn TunBridge>> = OnceLock::new();

#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn install_bridge(bridge: Box<dyn TunBridge>) {
    let _ = BRIDGE.set(bridge);
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn lock_path(dir: &Path) -> PathBuf {
    dir.join("proxy.lock")
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn write_lock_atomic(dir: &Path, lock: &ProxyLock) -> Result<()> {
    fs::create_dir_all(dir)?;
    let temporary = dir.join("proxy.lock.tmp");
    let mut file = File::create(&temporary)?;
    file.write_all(serde_json::to_string(lock)?.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, lock_path(dir)).context("atomic proxy.lock rename failed")
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_lock(dir: &Path) -> Option<ProxyLock> {
    fs::read_to_string(lock_path(dir)).ok().and_then(|raw| serde_json::from_str(&raw).ok())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn remove_lock(dir: &Path) {
    let _ = fs::remove_file(lock_path(dir));
}

#[cfg(windows)]
mod platform {
    use anyhow::Result;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use windows_sys::Win32::Networking::WinInet::{INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED, InternetSetOptionW};

    use super::ProxyLock;

    const KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    const BYPASS_LIST: &str = "localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.2*;172.30.*;172.31.*;192.168.*;<local>";

    fn refresh() {
        unsafe {
            InternetSetOptionW(std::ptr::null(), INTERNET_OPTION_SETTINGS_CHANGED, std::ptr::null(), 0);
            InternetSetOptionW(std::ptr::null(), INTERNET_OPTION_REFRESH, std::ptr::null(), 0);
        }
    }

    pub fn capture(lock: &mut ProxyLock) -> Result<()> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(KEY_PATH, KEY_READ)?;
        lock.previous_enable = key.get_value::<u32, _>("ProxyEnable").ok();
        lock.previous_server = key.get_value::<String, _>("ProxyServer").ok();
        lock.previous_override = key.get_value::<String, _>("ProxyOverride").ok();
        Ok(())
    }

    pub fn apply(port: u16, _lock: &ProxyLock) -> Result<()> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(KEY_PATH, KEY_SET_VALUE)?;
        key.set_value("ProxyServer", &format!("127.0.0.1:{port}"))?;
        key.set_value("ProxyOverride", &BYPASS_LIST)?;
        key.set_value("ProxyEnable", &1u32)?;
        refresh();
        Ok(())
    }

    pub fn restore(lock: &ProxyLock) -> Result<()> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(KEY_PATH, KEY_SET_VALUE)?;
        key.set_value("ProxyEnable", &lock.previous_enable.unwrap_or(0))?;
        match &lock.previous_server {
            Some(server) => key.set_value("ProxyServer", server)?,
            None => {
                let _ = key.delete_value("ProxyServer");
            }
        }
        match &lock.previous_override {
            Some(bypass) => key.set_value("ProxyOverride", bypass)?,
            None => {
                let _ = key.delete_value("ProxyOverride");
            }
        }
        refresh();
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::process::Command;

    use anyhow::{Result, bail};

    use super::ProxyLock;

    fn networksetup(args: &[&str]) -> Result<String> {
        let output = Command::new("networksetup").args(args).output()?;
        if !output.status.success() {
            bail!("networksetup {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn capture(lock: &mut ProxyLock) -> Result<()> {
        let listing = networksetup(&["-listallnetworkservices"])?;
        lock.services = listing
            .lines()
            .skip(1)
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('*'))
            .collect();
        Ok(())
    }

    pub fn apply(port: u16, lock: &ProxyLock) -> Result<()> {
        let port_text = port.to_string();
        for service in &lock.services {
            networksetup(&["-setwebproxy", service, "127.0.0.1", &port_text])?;
            networksetup(&["-setsecurewebproxy", service, "127.0.0.1", &port_text])?;
        }
        Ok(())
    }

    pub fn restore(lock: &ProxyLock) -> Result<()> {
        for service in &lock.services {
            let _ = networksetup(&["-setwebproxystate", service, "off"]);
            let _ = networksetup(&["-setsecurewebproxystate", service, "off"]);
        }
        Ok(())
    }
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "android"), not(target_os = "ios")))]
mod platform {
    use std::process::Command;

    use anyhow::Result;

    use super::ProxyLock;

    fn gsettings(args: &[&str]) -> Option<String> {
        let output = Command::new("gsettings").args(args).output().ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    pub fn capture(lock: &mut ProxyLock) -> Result<()> {
        lock.previous_mode = gsettings(&["get", "org.gnome.system.proxy", "mode"]).map(|mode| mode.trim_matches('\'').to_string());
        Ok(())
    }

    pub fn apply(port: u16, _lock: &ProxyLock) -> Result<()> {
        let port_text = port.to_string();
        for schema in ["org.gnome.system.proxy.http", "org.gnome.system.proxy.https"] {
            gsettings(&["set", schema, "host", "127.0.0.1"]);
            gsettings(&["set", schema, "port", &port_text]);
        }
        gsettings(&["set", "org.gnome.system.proxy", "mode", "manual"]);
        Ok(())
    }

    pub fn restore(lock: &ProxyLock) -> Result<()> {
        let mode = lock.previous_mode.clone().unwrap_or_else(|| "none".to_string());
        gsettings(&["set", "org.gnome.system.proxy", "mode", &mode]);
        Ok(())
    }
}

pub fn recover_if_needed(dir: &Path) -> bool {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = dir;
        return false;
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let Some(lock) = read_lock(dir) else { return false };
        let restored = platform::restore(&lock).is_ok();
        if restored {
            remove_lock(dir);
        }
        restored
    }
}

#[derive(Clone, Debug)]
pub struct SystemProxy {
    #[allow(dead_code)]
    dir: PathBuf,
}

impl SystemProxy {
    pub fn new(dir: PathBuf) -> SystemProxy {
        SystemProxy { dir }
    }

    pub fn enable(&self, ports: EffectivePorts, routing: &RoutingSettings, dns: &str) -> Result<()> {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            if let Some(bridge) = BRIDGE.get() {
                return bridge.start(ports, routing, dns);
            }
            return Ok(());
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let _ = (routing, dns);
            let mut lock = read_lock(&self.dir).unwrap_or_default();
            if lock.applied_port == 0 {
                platform::capture(&mut lock)?;
            }
            lock.applied_port = ports.http;
            write_lock_atomic(&self.dir, &lock)?;
            platform::apply(ports.http, &lock)
        }
    }

    pub fn disable(&self) -> Result<()> {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            if let Some(bridge) = BRIDGE.get() {
                return bridge.stop();
            }
            return Ok(());
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let Some(lock) = read_lock(&self.dir) else { return Ok(()) };
            platform::restore(&lock)?;
            remove_lock(&self.dir);
            Ok(())
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod mobile {
    use anyhow::{Result, anyhow};
    use serde::Serialize;
    use tauri::Runtime;
    use tauri::plugin::{Builder, PluginHandle, TauriPlugin};

    use super::{TunBridge, install_bridge};
    use crate::config::{EffectivePorts, RoutingSettings};

    #[cfg(target_os = "android")]
    const ANDROID_PACKAGE: &str = "dev.witwizard.radsho.vpn";

    #[cfg(target_os = "ios")]
    tauri::ios_plugin_binding!(init_plugin_radsho_vpn);

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct StartArgs {
        socks_port: u16,
        http_port: u16,
        dns: String,
        allowlist: bool,
        apps: Vec<String>,
        direct: Vec<String>,
    }

    struct PluginBridge<R: Runtime>(PluginHandle<R>);

    impl<R: Runtime> TunBridge for PluginBridge<R> {
        fn start(&self, ports: EffectivePorts, routing: &RoutingSettings, dns: &str) -> Result<()> {
            let args = StartArgs {
                socks_port: ports.socks,
                http_port: ports.http,
                dns: dns.to_string(),
                allowlist: routing.enabled && !routing.apps.is_empty(),
                apps: routing.apps.clone(),
                direct: routing.direct.clone(),
            };
            self.0
                .run_mobile_plugin::<serde_json::Value>("startVpn", args)
                .map(|_| ())
                .map_err(|error| anyhow!("startVpn failed: {error}"))
        }

        fn stop(&self) -> Result<()> {
            self.0
                .run_mobile_plugin::<serde_json::Value>("stopVpn", serde_json::json!({}))
                .map(|_| ())
                .map_err(|error| anyhow!("stopVpn failed: {error}"))
        }
    }

    pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
        Builder::new("radsho-vpn")
            .setup(|_app, api| {
                #[cfg(target_os = "android")]
                let handle = api.register_android_plugin(ANDROID_PACKAGE, "RadshoVpnPlugin")?;
                #[cfg(target_os = "ios")]
                let handle = api.register_ios_plugin(init_plugin_radsho_vpn)?;
                install_bridge(Box::new(PluginBridge(handle)));
                Ok(())
            })
            .build()
    }
}