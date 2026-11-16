use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::AsyncWriteExt;

use crate::config::APP_VERSION;
use crate::session::Session;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod app_update {
    use tauri::AppHandle;
    use tauri_plugin_updater::UpdaterExt;

    pub use tauri_plugin_updater::Update as AppUpdate;

    pub async fn check(app: &AppHandle) -> Option<AppUpdate> {
        app.updater().ok()?.check().await.ok().flatten()
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
mod app_update {
    use tauri::AppHandle;

    #[derive(Clone)]
    pub struct AppUpdate {
        pub version: String,
        pub body: Option<String>,
    }

    impl AppUpdate {
        pub async fn download<C, D>(&self, _on_chunk: C, _on_finish: D) -> Result<Vec<u8>, String>
        where
            C: FnMut(usize, Option<u64>),
            D: FnOnce(),
        {
            Err("mobile builds are updated through the app store".to_string())
        }

        pub fn install(&self, _bytes: Vec<u8>) -> Result<(), String> {
            Err("mobile builds are updated through the app store".to_string())
        }
    }

    pub async fn check(_app: &AppHandle) -> Option<AppUpdate> {
        None
    }
}

use app_update::AppUpdate;

pub const MIRROR_PREFIXES: [&str; 3] = ["", "https://gh-proxy.com/", "https://ghfast.top/"];
const CORE_RELEASE_API: &str = "https://api.github.com/repos/CluvexStudio/Aether/releases/latest";
const PROGRESS_EVENT: &str = "updater://progress";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub app_current: String,
    pub app_latest: String,
    pub core_current: String,
    pub core_latest: String,
    pub changelog: Vec<String>,
    pub mirror: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    body: Option<String>,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Clone, Debug)]
pub struct PendingCore {
    pub version: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Default)]
pub struct UpdaterState {
    app_update: Mutex<Option<AppUpdate>>,
    app_bytes: Mutex<Option<Vec<u8>>>,
    core_pending: Mutex<Option<PendingCore>>,
    core_staged: Mutex<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateJournal {
    from: String,
    to: String,
    started_at: u64,
    backup: Option<PathBuf>,
}

#[derive(Clone, Serialize)]
struct Progress {
    stage: &'static str,
    percent: u8,
}

pub fn core_dir(dir: &Path) -> PathBuf {
    dir.join("bin")
}

pub fn core_binary_path(dir: &Path) -> PathBuf {
    #[cfg(target_os = "android")]
    {
        let _ = dir;
        let native = std::env::var("RADSHO_NATIVE_LIB_DIR").unwrap_or_default();
        return PathBuf::from(native).join("libaether.so");
    }
    #[cfg(not(target_os = "android"))]
    {
        for env_key in ["RADSHO_CORE", "AETHERGATE_CORE"] {
            if let Ok(custom) = std::env::var(env_key) {
                let path = PathBuf::from(custom.trim());
                if !path.as_os_str().is_empty() {
                    return path;
                }
            }
        }
        let name = if cfg!(windows) { "aether.exe" } else { "aether" };
        let appdata_bin = core_dir(dir).join(name);
        if appdata_bin.exists() {
            return appdata_bin;
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                for candidate in [
                    exe_dir.join("core").join(name),
                    exe_dir.join("resources").join("core").join(name),
                    exe_dir.join(name),
                ] {
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            for candidate in [
                cwd.join("core").join(name),
                cwd.join("src-tauri").join("core").join(name),
                cwd.join("..").join("core").join(name),
            ] {
                if candidate.exists() {
                    return candidate;
                }
            }
        }
        appdata_bin
    }
}

pub fn installed_core_version(dir: &Path) -> String {
    if let Ok(raw) = fs::read_to_string(core_dir(dir).join("aether.version")) {
        return raw.trim().to_string();
    }
    if core_binary_path(dir).exists() {
        return "2.4.0".to_string();
    }
    "0.0.0".to_string()
}

pub fn core_asset_name() -> String {
    let extension = if cfg!(windows) { ".exe" } else { "" };
    format!("aether-{}-{}{}", std::env::consts::OS, std::env::consts::ARCH, extension)
}

pub fn version_greater(candidate: &str, current: &str) -> bool {
    fn parts(value: &str) -> Vec<u64> {
        value
            .trim_start_matches('v')
            .split(['.', '-', '+'])
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect()
    }
    let left = parts(candidate);
    let right = parts(current);
    for index in 0..left.len().max(right.len()) {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    false
}

fn mirror_label(prefix: &str) -> String {
    if prefix.is_empty() {
        "github.com".to_string()
    } else {
        prefix.trim_start_matches("https://").trim_end_matches('/').to_string()
    }
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("Radsho-Updater/1.0 (WitWizard)")
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(600))
        .build()
        .context("http client")
}

async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<(String, String)> {
    let mut last_error = anyhow!("no mirror attempted");
    for prefix in MIRROR_PREFIXES {
        let target = format!("{prefix}{url}");
        match client.get(&target).send().await.and_then(|response| response.error_for_status()) {
            Ok(response) => match response.text().await {
                Ok(text) => return Ok((text, mirror_label(prefix))),
                Err(error) => last_error = error.into(),
            },
            Err(error) => last_error = error.into(),
        }
    }
    Err(last_error)
}

async fn check_core(dir: &Path) -> Result<Option<(PendingCore, Vec<String>, String)>> {
    let client = http_client()?;
    let (raw, mirror) = fetch_text(&client, CORE_RELEASE_API).await?;
    let release: Release = serde_json::from_str(&raw).context("core release manifest")?;
    let latest = release.tag_name.trim_start_matches('v').to_string();
    if !version_greater(&latest, &installed_core_version(dir)) {
        return Ok(None);
    }
    let wanted = core_asset_name();
    let binary = release.assets.iter().find(|asset| asset.name == wanted).ok_or_else(|| anyhow!("no core asset for this platform"))?;
    let checksum_name = format!("{wanted}.sha256");
    let checksum = release.assets.iter().find(|asset| asset.name == checksum_name).ok_or_else(|| anyhow!("core asset has no sha256 sidecar"))?;
    let (checksum_text, _) = fetch_text(&client, &checksum.browser_download_url).await?;
    let sha256 = checksum_text.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
    if sha256.len() != 64 {
        bail!("malformed core sha256 sidecar");
    }
    let notes = release
        .body
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim().trim_start_matches(['-', '*', ' ']).to_string())
        .filter(|line| !line.is_empty())
        .take(6)
        .collect();
    Ok(Some((
        PendingCore {
            version: latest,
            url: binary.browser_download_url.clone(),
            sha256,
        },
        notes,
        mirror,
    )))
}

pub async fn download_core(dir: &Path, pending: &PendingCore, on_progress: impl Fn(u8)) -> Result<PathBuf> {
    let target_dir = core_dir(dir);
    tokio::fs::create_dir_all(&target_dir).await?;
    let staged = target_dir.join("aether.new");
    let client = http_client()?;
    let mut last_error = anyhow!("no mirror attempted");
    for prefix in MIRROR_PREFIXES {
        let url = format!("{prefix}{}", pending.url);
        let attempt = async {
            let response = client.get(&url).send().await?.error_for_status()?;
            let total = response.content_length().unwrap_or(0);
            let mut file = tokio::fs::File::create(&staged).await?;
            let mut hasher = Sha256::new();
            let mut received: u64 = 0;
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                hasher.update(&chunk);
                file.write_all(&chunk).await?;
                received += chunk.len() as u64;
                if total > 0 {
                    on_progress(((received * 100) / total).min(100) as u8);
                }
            }
            file.flush().await?;
            file.sync_all().await?;
            let digest = hex::encode(hasher.finalize());
            if digest != pending.sha256 {
                bail!("sha256 mismatch from {}: expected {}, got {}", mirror_label(prefix), pending.sha256, digest);
            }
            Ok::<(), anyhow::Error>(())
        };
        match attempt.await {
            Ok(()) => return Ok(staged),
            Err(error) => {
                last_error = error;
                let _ = tokio::fs::remove_file(&staged).await;
            }
        }
    }
    Err(last_error)
}

pub fn install_staged_core(dir: &Path, version: &str) -> Result<()> {
    let target_dir = core_dir(dir);
    let staged = target_dir.join("aether.new");
    if !staged.exists() {
        bail!("no staged core binary");
    }
    let final_path = core_dir(dir).join(if cfg!(windows) { "aether.exe" } else { "aether" });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))?;
    }
    if final_path.exists() {
        let backup = target_dir.join("aether.previous");
        let _ = fs::remove_file(&backup);
        fs::rename(&final_path, &backup)?;
    }
    if let Err(error) = fs::rename(&staged, &final_path) {
        let backup = target_dir.join("aether.previous");
        if backup.exists() {
            let _ = fs::rename(&backup, &final_path);
        }
        return Err(error.into());
    }
    fs::write(target_dir.join("aether.version"), version)?;
    Ok(())
}

fn journal_path(dir: &Path) -> PathBuf {
    dir.join("update.journal.json")
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_secs()).unwrap_or(0)
}

pub fn finalize_pending_update(dir: &Path) -> Option<bool> {
    let raw = fs::read_to_string(journal_path(dir)).ok()?;
    let journal: UpdateJournal = serde_json::from_str(&raw).ok()?;
    let succeeded = journal.to == APP_VERSION;
    if let Some(backup) = &journal.backup {
        let _ = fs::remove_file(backup);
    }
    let _ = fs::remove_file(journal_path(dir));
    Some(succeeded)
}

fn begin_update_journal(dir: &Path, target: &str) -> Result<()> {
    let backup_dir = dir.join("backup");
    fs::create_dir_all(&backup_dir)?;
    let backup = match std::env::current_exe() {
        Ok(current) => {
            let name = current.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| "radsho".to_string());
            let target_path = backup_dir.join(format!("{APP_VERSION}-{name}"));
            fs::copy(&current, &target_path)?;
            Some(target_path)
        }
        Err(_) => None,
    };
    let journal = UpdateJournal {
        from: APP_VERSION.to_string(),
        to: target.to_string(),
        started_at: now_secs(),
        backup,
    };
    fs::write(journal_path(dir), serde_json::to_string(&journal)?)?;
    Ok(())
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

fn emit_progress(app: &AppHandle, stage: &'static str, percent: u8) {
    let _ = app.emit(PROGRESS_EVENT, Progress { stage, percent });
}

#[tauri::command]
pub async fn updater_check(app: AppHandle, state: State<'_, UpdaterState>) -> Result<Option<UpdateInfo>, String> {
    let dir = data_dir(&app)?;
    let app_update = app_update::check(&app).await;
    let core = check_core(&dir).await.ok().flatten();
    if app_update.is_none() && core.is_none() {
        return Ok(None);
    }
    let mut changelog: Vec<String> = Vec::new();
    let mut mirror = "github.com".to_string();
    let mut sha256 = String::new();
    let app_latest = app_update.as_ref().map(|update| update.version.clone()).unwrap_or_else(|| APP_VERSION.to_string());
    if let Some(update) = &app_update {
        if let Some(body) = &update.body {
            changelog.extend(body.lines().map(|line| line.trim().trim_start_matches(['-', '*', ' ']).to_string()).filter(|line| !line.is_empty()).take(6));
        }
    }
    let installed = installed_core_version(&dir);
    let mut core_latest = installed.clone();
    if let Some((pending, notes, used_mirror)) = &core {
        core_latest = pending.version.clone();
        changelog.extend(notes.clone());
        mirror = used_mirror.clone();
        sha256 = pending.sha256.clone();
    }
    *state.app_update.lock().map_err(|error| error.to_string())? = app_update;
    *state.core_pending.lock().map_err(|error| error.to_string())? = core.map(|(pending, _, _)| pending);
    Ok(Some(UpdateInfo {
        app_current: APP_VERSION.to_string(),
        app_latest,
        core_current: installed,
        core_latest,
        changelog,
        mirror,
        sha256,
    }))
}

#[tauri::command]
pub async fn updater_download(app: AppHandle, state: State<'_, UpdaterState>) -> Result<(), String> {
    let dir = data_dir(&app)?;
    let update = state.app_update.lock().map_err(|error| error.to_string())?.clone();
    let pending = state.core_pending.lock().map_err(|error| error.to_string())?.clone();
    emit_progress(&app, "download", 0);
    if let Some(pending) = pending {
        let handle = app.clone();
        download_core(&dir, &pending, move |percent| emit_progress(&handle, "core", percent))
            .await
            .map_err(|error| error.to_string())?;
        *state.core_staged.lock().map_err(|error| error.to_string())? = true;
    }
    if let Some(update) = update {
        let handle = app.clone();
        let mut received: usize = 0;
        let bytes = update
            .download(
                move |chunk, total| {
                    received += chunk;
                    if let Some(total) = total {
                        emit_progress(&handle, "app", ((received as u64 * 100) / total.max(1)).min(100) as u8);
                    }
                },
                || {},
            )
            .await
            .map_err(|error| error.to_string())?;
        *state.app_bytes.lock().map_err(|error| error.to_string())? = Some(bytes);
    }
    emit_progress(&app, "download", 100);
    Ok(())
}

#[tauri::command]
pub async fn updater_install(app: AppHandle, state: State<'_, UpdaterState>, session: State<'_, Session>) -> Result<(), String> {
    let dir = data_dir(&app)?;
    session.disconnect().await;
    let staged = *state.core_staged.lock().map_err(|error| error.to_string())?;
    let pending = state.core_pending.lock().map_err(|error| error.to_string())?.clone();
    if staged {
        if let Some(pending) = pending {
            install_staged_core(&dir, &pending.version).map_err(|error| error.to_string())?;
        }
        *state.core_staged.lock().map_err(|error| error.to_string())? = false;
    }
    let update = state.app_update.lock().map_err(|error| error.to_string())?.take();
    let bytes = state.app_bytes.lock().map_err(|error| error.to_string())?.take();
    if let (Some(update), Some(bytes)) = (update, bytes) {
        begin_update_journal(&dir, &update.version).map_err(|error| error.to_string())?;
        update.install(bytes).map_err(|error| error.to_string())?;
        app.restart();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!(version_greater("1.10.0", "1.9.9"));
        assert!(version_greater("v0.9.6", "0.9.4"));
        assert!(!version_greater("1.0.0", "1.0.0"));
        assert!(!version_greater("0.9.4", "0.9.6"));
    }
}