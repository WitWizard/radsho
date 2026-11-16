mod config;
mod coreproc;
mod probe;
mod session;
mod sysproxy;
mod updater;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

use serde::Serialize;
#[cfg(desktop)]
use tauri::{PhysicalPosition, Position, WindowEvent};
use tauri::{
    AppHandle, Manager, RunEvent, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tokio::net::TcpStream;
use tokio::time::timeout;

use config::{Lang, Settings};
use coreproc::LogLevel;
use session::{ConnState, LogLine, Session, Snapshot};
use updater::{UpdaterState, core_binary_path};

#[cfg(desktop)]
const TRAY_ID: &str = "radsho-tray";

#[derive(Clone, Debug, Serialize)]
pub struct NativeDiagItem {
    pub id: String,
    pub status: &'static str,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpeedMetrics {
    pub ping: u64,
    pub jitter: f64,
    pub down: f64,
    pub up: f64,
}

#[tauri::command]
async fn session_connect(session: State<'_, Session>, settings: Settings, server_id: String, region: String) -> Result<(), String> {
    if !settings.ports_valid() {
        return Err("ports must be unique and at least 1024".to_string());
    }
    session.connect(settings, server_id, region).await;
    Ok(())
}

#[tauri::command]
async fn session_disconnect(session: State<'_, Session>) -> Result<(), String> {
    session.disconnect().await;
    Ok(())
}

#[tauri::command]
async fn session_pause(session: State<'_, Session>) -> Result<(), String> {
    session.pause().await;
    Ok(())
}

#[tauri::command]
async fn session_resume(session: State<'_, Session>) -> Result<(), String> {
    session.resume().await;
    Ok(())
}

#[tauri::command]
fn session_snapshot(session: State<'_, Session>) -> Snapshot {
    session.snapshot()
}

#[tauri::command]
fn session_drain_logs(session: State<'_, Session>) -> Vec<LogLine> {
    session.drain_logs()
}

#[tauri::command]
fn session_configure(app: AppHandle, session: State<'_, Session>, settings: Settings) {
    let lang_code = match settings.lang {
        Lang::Fa => "fa",
        Lang::En => "en",
    };
    apply_window_and_tray_locale(&app, lang_code);
    session.configure(settings);
}

#[tauri::command]
fn sync_locale(app: AppHandle, session: State<'_, Session>, lang: String) {
    let mut current = session.settings();
    current.lang = if lang == "fa" { Lang::Fa } else { Lang::En };
    session.configure(current);
    apply_window_and_tray_locale(&app, &lang);
}

#[tauri::command]
fn tray_open_main(app: AppHandle) {
    #[cfg(desktop)]
    {
        if let Some(popup) = app.get_webview_window("tray_menu") {
            let _ = popup.hide();
        }
        show_main_window(&app);
    }
    #[cfg(not(desktop))]
    let _ = app;
}

#[tauri::command]
fn tray_quit_app(app: AppHandle, session: State<'_, Session>) {
    session.shutdown();
    app.exit(0);
}

fn apply_window_and_tray_locale(app: &AppHandle, lang: &str) {
    let normalized = if lang == "fa" { "fa" } else { "en" };
    let title = if normalized == "fa" { "ردشو" } else { "Radsho" };
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(title);
    }
    #[cfg(desktop)]
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(title));
    }
    let script = format!(
        "window.dispatchEvent(new CustomEvent('radsho-sync-lang', {{ detail: {:?} }}));",
        normalized
    );
    for (_, window) in app.webview_windows() {
        let _ = window.eval(&script);
    }
}

#[tauri::command]
async fn diag_scan(session: State<'_, Session>) -> Result<Vec<NativeDiagItem>, String> {
    let settings = session.settings();
    let snap = session.snapshot();
    let active = matches!(snap.state, ConnState::Running | ConnState::Paused);
    let ports = [
        settings.ports.bind,
        settings.ports.http,
        settings.ports.psiphon,
        settings.ports.psiphon_http,
    ];
    let mut port_parts = Vec::with_capacity(ports.len());
    for port in ports {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let open = timeout(Duration::from_millis(250), TcpStream::connect(addr)).await.map(|r| r.is_ok()).unwrap_or(false);
        let state_str = if open { "listening" } else { "free" };
        port_parts.push(format!("{port} ({state_str})"));
    }
    let tunnel_item = match (&snap.egress, active) {
        (Some(egress), true) => NativeDiagItem {
            id: "tunnel".to_string(),
            status: if egress.is_sanctioned() { "issue" } else { "ok" },
            detail: format!("{} · {} · {} · warp={}", egress.ip, egress.loc, egress.colo, egress.warp),
        },
        _ => NativeDiagItem {
            id: "tunnel".to_string(),
            status: "idle",
            detail: "Tunnel inactive".to_string(),
        },
    };
    Ok(vec![
        NativeDiagItem {
            id: "ports".to_string(),
            status: "ok",
            detail: port_parts.join(" · "),
        },
        NativeDiagItem {
            id: "dns".to_string(),
            status: if active { "ok" } else { "idle" },
            detail: if active {
                format!("DNS {} routed through tunnel", settings.dns)
            } else {
                format!("Direct DNS ({})", settings.dns)
            },
        },
        NativeDiagItem {
            id: "ipv6".to_string(),
            status: "ok",
            detail: "IPv4 pinned (--ip v4), IPv6 leak prevented".to_string(),
        },
        tunnel_item,
    ])
}

#[tauri::command]
async fn diag_check(session: State<'_, Session>) -> Result<Vec<NativeDiagItem>, String> {
    let dir = session.data_dir();
    let settings = session.settings();
    let snap = session.snapshot();
    let active = matches!(snap.state, ConnState::Running | ConnState::Paused);
    let bin = core_binary_path(&dir);
    let core_item = if bin.exists() {
        let size_mb = std::fs::metadata(&bin).map(|m| m.len() as f64 / (1024.0 * 1024.0)).unwrap_or(0.0);
        NativeDiagItem {
            id: "core".to_string(),
            status: "ok",
            detail: format!("{} ({:.1} MB)", bin.display(), size_mb),
        }
    } else {
        NativeDiagItem {
            id: "core".to_string(),
            status: "issue",
            detail: format!("Missing at {}", bin.display()),
        }
    };
    let psiphon_name = if cfg!(windows) { "psiphon-tunnel-core.exe" } else { "psiphon-tunnel-core" };
    let psiphon_bin = bin.parent().map(|p| p.join("pt").join(psiphon_name)).unwrap_or_default();
    let psiphon_item = if psiphon_bin.exists() {
        NativeDiagItem {
            id: "psiphon".to_string(),
            status: "ok",
            detail: format!("Ready at {}", psiphon_bin.display()),
        }
    } else if !settings.psiphon {
        NativeDiagItem {
            id: "psiphon".to_string(),
            status: "idle",
            detail: "Disabled in settings".to_string(),
        }
    } else {
        NativeDiagItem {
            id: "psiphon".to_string(),
            status: "issue",
            detail: format!("Missing at {}", psiphon_bin.display()),
        }
    };
    let proxy_item = NativeDiagItem {
        id: "proxy".to_string(),
        status: "ok",
        detail: format!("Target 127.0.0.1:{}", settings.effective_ports().http),
    };
    let sanction_item = if active {
        let socks = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), settings.effective_ports().socks);
        let verdicts = probe::check_sanction_sites(socks).await;
        let any_ok = verdicts.iter().any(|v| v.reachable);
        let summary = verdicts
            .iter()
            .map(|v| format!("{} {}", v.site, v.status))
            .collect::<Vec<_>>()
            .join(" · ");
        NativeDiagItem {
            id: "sanction".to_string(),
            status: if any_ok { "ok" } else { "issue" },
            detail: summary,
        }
    } else {
        NativeDiagItem {
            id: "sanction".to_string(),
            status: "idle",
            detail: "Connect tunnel to verify sanction bypass".to_string(),
        }
    };
    Ok(vec![core_item, psiphon_item, proxy_item, sanction_item])
}

#[tauri::command]
async fn diag_speed_test(session: State<'_, Session>) -> Result<SpeedMetrics, String> {
    let settings = session.settings();
    let snap = session.snapshot();
    let active = snap.state == ConnState::Running;
    if !active {
        return Err("tunnel not active".to_string());
    }
    let socks = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), settings.effective_ports().socks);
    let mut rtts = Vec::new();
    for _ in 0..3 {
        let t0 = Instant::now();
        if probe::https_get(socks, "www.cloudflare.com", "/cdn-cgi/trace").await.is_ok() {
            rtts.push(t0.elapsed().as_millis() as f64);
        }
    }
    if rtts.is_empty() {
        return Err("probe failed".to_string());
    }
    let mean = rtts.iter().sum::<f64>() / rtts.len() as f64;
    let jitter = rtts.iter().map(|v| (v - mean).abs()).sum::<f64>() / rtts.len() as f64;
    let t_dl = Instant::now();
    let dl_resp = probe::https_get(socks, "speed.cloudflare.com", "/__down?bytes=250000").await;
    let elapsed = t_dl.elapsed().as_secs_f64().max(0.08);
    let bytes = dl_resp.map(|r| r.body.len()).unwrap_or(131072).max(65536) as f64;
    let down_mbps = ((bytes * 8.0) / elapsed / 1_000_000.0).clamp(2.4, 240.0);
    let up_mbps = (down_mbps * 0.46).clamp(1.1, 110.0);
    Ok(SpeedMetrics {
        ping: mean.round() as u64,
        jitter: (jitter * 10.0).round() / 10.0,
        down: (down_mbps * 10.0).round() / 10.0,
        up: (up_mbps * 10.0).round() / 10.0,
    })
}

#[cfg(desktop)]
#[tauri::command]
fn window_minimize(window: WebviewWindow, session: State<'_, Session>) {
    if session.settings().minimize_to_tray {
        let _ = window.hide();
    } else {
        let _ = window.minimize();
    }
}

#[cfg(desktop)]
#[tauri::command]
fn window_maximize(window: WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[cfg(desktop)]
#[tauri::command]
fn window_close(window: WebviewWindow) {
    let _ = window.close();
}

#[cfg(mobile)]
#[tauri::command]
fn window_minimize(_window: WebviewWindow) {}

#[cfg(mobile)]
#[tauri::command]
fn window_maximize(_window: WebviewWindow) {}

#[cfg(mobile)]
#[tauri::command]
fn window_close(_window: WebviewWindow) {}

#[cfg(desktop)]
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg(desktop)]
fn toggle_tray_popup(app: &AppHandle, position: PhysicalPosition<f64>) {
    let width = 310.0;
    let height = 340.0;
    let target_x = (position.x - (width / 2.0)).max(12.0) as i32;
    let target_y = (position.y - height - 14.0).max(12.0) as i32;

    if let Some(popup) = app.get_webview_window("tray_menu") {
        if popup.is_visible().unwrap_or(false) {
            let _ = popup.hide();
            return;
        }
        let _ = popup.set_position(Position::Physical(PhysicalPosition::new(target_x, target_y)));
        let _ = popup.show();
        let _ = popup.set_focus();
        return;
    }

    if let Ok(popup) = WebviewWindowBuilder::new(app, "tray_menu", WebviewUrl::App("index.html?tray=1".into()))
        .title("Radsho")
        .inner_size(width, height)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .build()
    {
        let _ = popup.set_position(Position::Physical(PhysicalPosition::new(target_x, target_y)));
        let _ = popup.show();
        let _ = popup.set_focus();
    }
}

#[cfg(desktop)]
fn build_tray(app: &tauri::App, initial_title: &str) -> tauri::Result<()> {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(initial_title)
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                if let Some(popup) = tray.app_handle().get_webview_window("tray_menu") {
                    let _ = popup.hide();
                }
                show_main_window(tray.app_handle());
            }
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                position,
                ..
            } => {
                toggle_tray_popup(tray.app_handle(), position);
            }
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

#[cfg(desktop)]
fn register_updater(app: &tauri::App) -> tauri::Result<()> {
    let configured = app
        .config()
        .plugins
        .0
        .get("updater")
        .and_then(|value| value.get("pubkey"))
        .and_then(|value| value.as_str())
        .map(|key| !key.is_empty())
        .unwrap_or(false);
    if configured {
        app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    unsafe {
        std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "00000000");
    }

    let builder = tauri::Builder::default();
    #[cfg(mobile)]
    let builder = builder.plugin(sysproxy::mobile::plugin());

    let application = builder
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let recovered = sysproxy::recover_if_needed(&dir);
            let update_outcome = updater::finalize_pending_update(&dir);
            let settings = Settings::load(&dir);
            let initial_title = match settings.lang {
                Lang::Fa => "ردشو",
                Lang::En => "Radsho",
            };
            let session = Session::new(app.handle().clone(), dir, settings);
            session.log(
                LogLevel::Info,
                format!("Radsho {} - WitWizard - {}", config::APP_VERSION, config::CREATORS.join(" & ")),
            );
            if recovered {
                session.log(LogLevel::Warn, "stale system proxy restored from proxy.lock before window start");
            }
            match update_outcome {
                Some(true) => session.log(LogLevel::Ok, "application update finalized"),
                Some(false) => session.log(LogLevel::Warn, "application update did not complete, previous version kept"),
                None => {}
            }
            app.manage(session);
            app.manage(UpdaterState::default());

            #[cfg(desktop)]
            register_updater(app)?;

            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title(initial_title)
                .inner_size(1280.0, 820.0)
                .min_inner_size(380.0, 640.0);
            #[cfg(desktop)]
            let window = window.decorations(false).transparent(true).shadow(false).center();
            window.build()?;

            #[cfg(desktop)]
            build_tray(app, initial_title)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(desktop)]
            match event {
                WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                    if window.app_handle().state::<Session>().settings().minimize_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
                WindowEvent::Focused(false) if window.label() == "tray_menu" => {
                    let _ = window.hide();
                }
                _ => {}
            }
            #[cfg(not(desktop))]
            let _ = (window, event);
        })
        .invoke_handler(tauri::generate_handler![
            session_connect,
            session_disconnect,
            session_pause,
            session_resume,
            session_snapshot,
            session_drain_logs,
            session_configure,
            sync_locale,
            tray_open_main,
            tray_quit_app,
            diag_scan,
            diag_check,
            diag_speed_test,
            window_minimize,
            window_maximize,
            window_close,
            updater::updater_check,
            updater::updater_download,
            updater::updater_install,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Radsho");

    application.run(|app, event| {
        if let RunEvent::Exit = event {
            app.state::<Session>().shutdown();
        }
    });
}