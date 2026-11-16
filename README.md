# Radsho (ردشو)

Censorship-resistant VPN and smart sanction-bypass client by **WitWizard** — created by **Lilmmd** and **LFnighTT**.

Single codebase: Tauri v2 (Rust) + React 19 + TypeScript + Tailwind CSS 4 + Framer Motion.

## Layout

```
Cargo.toml                    workspace
src-tauri/
  tauri.conf.json
  capabilities/default.json
  src/
    main.rs  lib.rs           entry, commands, tray, window, crash recovery before window start
    config.rs                 settings, presets, transports, aether argv builder, port helpers
    coreproc.rs               async process supervisor, Windows Job Object, Unix process groups, stats parser
    probe.rs                  RFC 1928 SOCKS5 client, rustls probe, /cdn-cgi/trace parser, sanction verifier
    sysproxy.rs               WinINET + proxy.lock, networksetup, gsettings, mobile TUN bridge
    session.rs                state machine, generation counter, fallback chain, pause/resume, shuffle
    updater.rs                signed app updater + core downloader with SHA-256 and mirror fallback
  mobile/android              VpnService + hev-socks5-tunnel plugin
  mobile/ios                  NEPacketTunnelProvider + plugin
src/
  App.tsx  styles/globals.css  i18n/translations.ts  types/vpn.ts
  lib/        config, engine (native + simulated), diagnostics, servers, bridge
  state/      AppContext
  components/ Titlebar, Navigation, ConnectionHub, OptionCards, OptionsDrawer, OptionModals,
              AdvancedCards, AdvancedModals, UpdaterModal, AboutCard, Flag, Logo, ui
  views/      DashboardView, LocationsView, LogsView, SettingsView, SimpleViews
scripts/build-windows.ps1  scripts/build-all.sh
```

## Run

```
npm install
npm run dev          browser preview with the simulated engine
npm run tauri:dev    native app, requires the aether core at <app data>/bin/aether
```

Inside Tauri the UI talks to the Rust session through `session_*` commands. In a plain browser it falls back to a simulated engine that runs the same state machine and argv rules.

## Release

```
pwsh scripts/build-windows.ps1
./scripts/build-all.sh desktop | android | ios | all
```

The scripts generate an Ed25519 updater key in `.keys/` (or use `RADSHO_UPDATER_PUBKEY` and `TAURI_SIGNING_PRIVATE_KEY`) and pass the public key through a merged config file.
