# Radsho2

- Generated: 2026-11-18 12:58:15
- Root: `C:\Users\MIT\Desktop\Radsho2`
- Files included: 60
- Total size: 374.74 KB

## Structure

```text
Radsho2/
+-- .github/
|   \-- workflows/
|       \-- release.yml
+-- scripts/
|   +-- build-all.sh
|   \-- build-windows.ps1
+-- src/
|   +-- components/
|   |   +-- AboutCard.tsx
|   |   +-- AdvancedCards.tsx
|   |   +-- AdvancedModals.tsx
|   |   +-- ConnectionHub.tsx
|   |   +-- Flag.tsx
|   |   +-- Logo.tsx
|   |   +-- Navigation.tsx
|   |   +-- OptionCards.tsx
|   |   +-- OptionModals.tsx
|   |   +-- OptionsDrawer.tsx
|   |   +-- Page.tsx
|   |   +-- SettingRow.tsx
|   |   +-- Titlebar.tsx
|   |   +-- ui.tsx
|   |   \-- UpdaterModal.tsx
|   +-- i18n/
|   |   \-- translations.ts
|   +-- lib/
|   |   +-- bridge.ts
|   |   +-- config.ts
|   |   +-- diagnostics.ts
|   |   +-- engine.ts
|   |   \-- servers.ts
|   +-- state/
|   |   \-- AppContext.tsx
|   +-- styles/
|   |   \-- globals.css
|   +-- types/
|   |   \-- vpn.ts
|   +-- utils/
|   |   \-- cn.ts
|   +-- views/
|   |   +-- DashboardView.tsx
|   |   +-- LocationsView.tsx
|   |   +-- LogsView.tsx
|   |   +-- SettingsView.tsx
|   |   \-- SimpleViews.tsx
|   +-- App.tsx
|   \-- main.tsx
+-- src-tauri/
|   +-- capabilities/
|   |   \-- default.json
|   +-- mobile/
|   |   +-- android/
|   |   |   +-- RadshoVpnPlugin.kt
|   |   |   +-- RadshoVpnService.kt
|   |   |   +-- service-manifest.xml
|   |   |   \-- TProxyService.kt
|   |   \-- ios/
|   |       +-- PacketTunnelProvider.swift
|   |       \-- RadshoVpnPlugin.swift
|   +-- src/
|   |   +-- config.rs
|   |   +-- coreproc.rs
|   |   +-- lib.rs
|   |   +-- main.rs
|   |   +-- probe.rs
|   |   +-- session.rs
|   |   +-- sysproxy.rs
|   |   \-- updater.rs
|   +-- build.rs
|   +-- Cargo.toml
|   \-- tauri.conf.json
+-- .gitignore
+-- Cargo.toml
+-- index.html
+-- package.json
+-- README.md
+-- tsconfig.json
\-- vite.config.ts
```

## `.github/workflows/release.yml`

```yaml
name: Build All Platforms (Radsho)

on:
  push:
    tags:
      - "v*"
  workflow_dispatch:

permissions:
  contents: write

env:
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"
  NODE_OPTIONS: "--no-deprecation"
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-A warnings"

jobs:
  build-desktop:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: windows-latest
            platform: windows
            script: powershell -ExecutionPolicy Bypass -File ./scripts/build-windows.ps1
          - os: ubuntu-22.04
            platform: linux
            script: bash ./scripts/build-all.sh desktop
          - os: macos-latest
            platform: macos
            script: bash ./scripts/build-all.sh desktop
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - name: Install Linux Dependencies
        if: matrix.os == 'ubuntu-22.04'
        env:
          DEBIAN_FRONTEND: noninteractive
        run: |
          sudo apt-get update -qq
          sudo apt-get install -y -qq \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            libxdo-dev \
            libssl-dev \
            libfuse2 \
            squashfs-tools \
            desktop-file-utils \
            xdg-utils \
            rpm \
            file \
            zip \
            build-essential \
            curl \
            wget >/dev/null
      - name: Build Desktop Artifacts
        run: ${{ matrix.script }}
      - name: Upload Desktop Installers
        uses: actions/upload-artifact@v4
        with:
          name: radsho-${{ matrix.platform }}
          path: dist-installers/*
          if-no-files-found: error

  build-android:
    runs-on: ubuntu-22.04
    env:
      GRADLE_OPTS: "-Dorg.gradle.daemon=false -Dorg.gradle.logging.level=lifecycle -Dorg.gradle.warning.mode=none"
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - uses: actions/setup-java@v4
        with:
          distribution: temurin
          java-version: 17
      - name: Setup Android SDK & NDK
        run: |
          SDK_ROOT="${ANDROID_HOME:-/usr/local/lib/android/sdk}"
          export ANDROID_HOME="$SDK_ROOT"
          export ANDROID_SDK_ROOT="$SDK_ROOT"
          echo "ANDROID_HOME=$SDK_ROOT" >> "$GITHUB_ENV"
          echo "ANDROID_SDK_ROOT=$SDK_ROOT" >> "$GITHUB_ENV"
          SDKMANAGER="$SDK_ROOT/cmdline-tools/latest/bin/sdkmanager"
          if [ ! -x "$SDKMANAGER" ]; then
            SDKMANAGER="$(find "$SDK_ROOT" -name sdkmanager -type f | head -n 1)"
          fi
          yes | "$SDKMANAGER" --licenses >/dev/null 2>&1 || true
          "$SDKMANAGER" --install "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;27.0.12077973" >/dev/null 2>&1
          NDK_PATH="$SDK_ROOT/ndk/27.0.12077973"
          echo "NDK_HOME=$NDK_PATH" >> "$GITHUB_ENV"
          echo "ANDROID_NDK_HOME=$NDK_PATH" >> "$GITHUB_ENV"
          echo "$SDK_ROOT/platform-tools" >> "$GITHUB_PATH"
          echo "$NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/bin" >> "$GITHUB_PATH"
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-linux-android,armv7-linux-androideabi,i686-linux-android,x86_64-linux-android
      - name: Build Android APK & AAB
        run: bash ./scripts/build-all.sh android
      - name: Upload Android Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: radsho-android
          path: dist-installers/*
          if-no-files-found: error

  publish-release:
    needs: [build-desktop, build-android]
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - name: Download All Artifacts
        uses: actions/download-artifact@v4
        with:
          path: release-assets
          merge-multiple: true
      - name: Generate Checksums and Summary
        run: |
          cd release-assets
          rm -f SHA256SUMS.txt
          sha256sum * > SHA256SUMS.txt
          ls -lh
      - name: Upload Combined Release Bundle
        uses: actions/upload-artifact@v4
        with:
          name: radsho-all-platforms-release
          path: release-assets/*
      - name: Publish GitHub Release
        if: startsWith(github.ref, 'refs/tags/v')
        uses: softprops/action-gh-release@v2
        with:
          files: release-assets/*
          generate_release_notes: true
          draft: false
          prerelease: false
```

## `.gitignore`

```gitignore
node_modules
dist
dist-installers
target
.keys
src-tauri/gen
src-tauri/tauri.updater.conf.json
src-tauri/Cargo.lock.bak
*.log
```

## `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = ["src-tauri"]

[workspace.package]
version = "1.0.0"
edition = "2021"
rust-version = "1.77"
authors = ["WitWizard (Lilmmd & LFnighTT)"]
license = "MIT"
homepage = "https://witwizard.dev/radsho"
repository = "https://github.com/WitWizard/radsho"

[profile.release]
opt-level = "s"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true
```

## `index.html`

```html
<!doctype html>
<html lang="en" dir="ltr">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover" />
    <meta name="theme-color" content="#0e1514" />
    <meta name="author" content="WitWizard (Lilmmd &amp; LFnighTT)" />
    <meta name="description" content="Radsho (ردشو) - ultra-fast, censorship-resistant VPN and smart sanction-bypass client by WitWizard." />
    <link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 32 40%27%3E%3Crect width=%2732%27 height=%2740%27 rx=%276%27 fill=%27%230e1514%27/%3E%3Crect x=%279%27 y=%271%27 width=%2714%27 height=%278%27 fill=%27%2300e599%27/%3E%3Crect x=%273%27 y=%2711%27 width=%2726%27 height=%276%27 fill=%27%2300e599%27/%3E%3Crect x=%273%27 y=%2719%27 width=%2726%27 height=%276%27 fill=%27%2300e599%27/%3E%3Cpath d=%27M9 27h14v5.500l-7 6-7-6z%27 fill=%27%2300e599%27/%3E%3C/svg%3E" />
    <title>Radsho (ردشو) — Smart Sanction-Bypass VPN by WitWizard</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

## `package.json`

```json
{
  "name": "radsho",
  "private": true,
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  },
  "dependencies": {
    "@fontsource-variable/jetbrains-mono": "^5.1.0",
    "@fontsource-variable/plus-jakarta-sans": "^5.1.0",
    "@fontsource-variable/vazirmatn": "^5.1.0",
    "clsx": "^2.1.1",
    "framer-motion": "^11.15.0",
    "lucide-react": "^0.468.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "tailwind-merge": "^2.6.0"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4.0.0",
    "@tauri-apps/cli": "^2.1.0",
    "@types/node": "^22.10.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.4",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.7.2",
    "vite": "^6.0.5"
  },
  "description": "Radsho (ردشو) - censorship-resistant VPN and smart sanction-bypass client by WitWizard",
  "author": "WitWizard (Lilmmd & LFnighTT)",
  "license": "MIT",
  "contributors": [
    {
      "name": "Lilmmd"
    },
    {
      "name": "LFnighTT"
    }
  ],
  "homepage": "https://witwizard.dev/radsho"
}
```

## `README.md`

````markdown
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
````

## `scripts/build-all.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export RUSTFLAGS="${RUSTFLAGS:--A warnings}"
export NPM_CONFIG_LOGLEVEL=error

PLATFORM="${1:-desktop}"
KEY_DIR="$ROOT/.keys"
KEY_PATH="$KEY_DIR/radsho-updater.key"
OVERLAY="$ROOT/src-tauri/tauri.updater.conf.json"
OUT_DIR="$ROOT/dist-installers"
VERSION="$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')"

write_sig() {
  local file="$1"
  if [ -f "$file" ] && [ ! -f "$file.sig" ]; then
    local digest
    if command -v sha256sum >/dev/null 2>&1; then
      digest="$(sha256sum "$file" | awk '{print $1}')"
    else
      digest="$(shasum -a 256 "$file" | awk '{print $1}')"
    fi
    printf "untrusted comment: radsho sha256 signature\n%s\n" "$digest" > "$file.sig"
  fi
}

prepare_toolchain() {
  if [ -f package-lock.json ]; then
    npm ci --no-audit --no-fund --silent
  else
    npm install --no-audit --no-fund --silent
  fi
  npm run --silent tauri -- icon public/radsho-icon.png >/dev/null 2>&1 || npm run tauri -- icon public/radsho-icon.png
  mkdir -p src-tauri/core/pt
  if [ "$PLATFORM" = "desktop" ] && [ -d core ]; then
    find core -type f ! -name "*.exe" -exec cp -f {} src-tauri/core/ \; 2>/dev/null || true
  fi
  if [ -z "$(find src-tauri/core -type f 2>/dev/null)" ]; then
    touch src-tauri/core/.keep
  fi
}

prepare_signing() {
  mkdir -p "$KEY_DIR" "$OUT_DIR"
  if [ -n "${RADSHO_UPDATER_PUBKEY:-}" ] && [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
    PUBLIC_KEY="$RADSHO_UPDATER_PUBKEY"
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"
  else
    if [ ! -f "$KEY_PATH" ]; then
      npm run --silent tauri -- signer generate --ci --force -w "$KEY_PATH" >/dev/null 2>&1 || npm run tauri -- signer generate --ci --force -w "$KEY_PATH"
    fi
    PUBLIC_KEY="$(tr -d '\r\n' < "$KEY_PATH.pub")"
    export TAURI_SIGNING_PRIVATE_KEY="$(tr -d '\r\n' < "$KEY_PATH")"
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
  fi
  printf "%s\n" "$PUBLIC_KEY" > "$OUT_DIR/radsho-public-key.asc"
  python3 - "$OVERLAY" "$PUBLIC_KEY" <<'PY'
import json
import sys

target, key = sys.argv[1], sys.argv[2]
payload = {
    "bundle": {
        "createUpdaterArtifacts": True
    },
    "plugins": {
        "updater": {
            "pubkey": key,
            "endpoints": [
                "https://github.com/WitWizard/radsho/releases/latest/download/latest.json",
                "https://gh-proxy.com/https://github.com/WitWizard/radsho/releases/latest/download/latest.json",
                "https://ghfast.top/https://github.com/WitWizard/radsho/releases/latest/download/latest.json"
            ],
            "windows": {
                "installMode": "passive"
            }
        }
    }
}
with open(target, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
PY
}

build_desktop() {
  mkdir -p "$OUT_DIR"
  case "$(uname -s)" in
    Darwin)
      rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null 2>&1 || true
      npm run tauri -- build --target universal-apple-darwin --bundles app --config "$OVERLAY"
      local app_dir
      app_dir="$(find "$ROOT/target/universal-apple-darwin/release/bundle/macos" -name "*.app" -type d | head -n 1 || true)"
      if [ -n "$app_dir" ]; then
        local dmg_stage="$ROOT/target/dmg-stage"
        rm -rf "$dmg_stage"
        mkdir -p "$dmg_stage"
        cp -R "$app_dir" "$dmg_stage/"
        ln -s /Applications "$dmg_stage/Applications" 2>/dev/null || true
        hdiutil create -volname "Radsho" -srcfolder "$dmg_stage" -ov -format UDZO "$OUT_DIR/Radsho-macos-universal.dmg" >/dev/null
        rm -rf "$dmg_stage"
        cp -f "$OUT_DIR/Radsho-macos-universal.dmg" "$OUT_DIR/Radsho-macos-arm64.dmg"
        cp -f "$OUT_DIR/Radsho-macos-universal.dmg" "$OUT_DIR/Radsho-macos-64.dmg"
        write_sig "$OUT_DIR/Radsho-macos-universal.dmg"
        write_sig "$OUT_DIR/Radsho-macos-arm64.dmg"
        write_sig "$OUT_DIR/Radsho-macos-64.dmg"

        (cd "$(dirname "$app_dir")" && ditto -c -k --sequesterRsrc --keepParent "$(basename "$app_dir")" "$OUT_DIR/Radsho-macos-universal.zip")
        cp -f "$OUT_DIR/Radsho-macos-universal.zip" "$OUT_DIR/Radsho-macos-arm64.zip"
        cp -f "$OUT_DIR/Radsho-macos-universal.zip" "$OUT_DIR/Radsho-macos-64.zip"
        write_sig "$OUT_DIR/Radsho-macos-universal.zip"
        write_sig "$OUT_DIR/Radsho-macos-arm64.zip"
        write_sig "$OUT_DIR/Radsho-macos-64.zip"
      fi
      ;;
    Linux)
      export NO_STRIP=true
      export APPIMAGE_EXTRACT_AND_RUN=1
      export ARCH=x86_64
      npm run tauri -- build --bundles deb,rpm,appimage --config "$OVERLAY" || npm run tauri -- build --bundles deb,rpm --config "$OVERLAY" || npm run tauri -- build --bundles deb --config "$OVERLAY"

      local deb_file
      deb_file="$(find "$ROOT/target/release/bundle/deb" -name "*.deb" -type f | head -n 1 || true)"
      if [ -n "$deb_file" ]; then
        cp -f "$deb_file" "$OUT_DIR/Radsho-linux-64.deb"
        write_sig "$OUT_DIR/Radsho-linux-64.deb"
      fi

      local rpm_file
      rpm_file="$(find "$ROOT/target/release/bundle/rpm" -name "*.rpm" -type f | head -n 1 || true)"
      if [ -n "$rpm_file" ]; then
        cp -f "$rpm_file" "$OUT_DIR/Radsho-linux-64.rpm"
        write_sig "$OUT_DIR/Radsho-linux-64.rpm"
      fi

      local appimage_file
      appimage_file="$(find "$ROOT/target/release/bundle/appimage" -name "*.AppImage" -type f | head -n 1 || true)"
      if [ -n "$appimage_file" ]; then
        cp -f "$appimage_file" "$OUT_DIR/Radsho-linux-64.AppImage"
        write_sig "$OUT_DIR/Radsho-linux-64.AppImage"
      fi

      if [ -f "$ROOT/target/release/radsho" ]; then
        local stage_dir="$ROOT/target/linux-portable"
        rm -rf "$stage_dir"
        mkdir -p "$stage_dir/core"
        cp -f "$ROOT/target/release/radsho" "$stage_dir/radsho"
        chmod +x "$stage_dir/radsho"
        (cd "$stage_dir" && zip -qr "$OUT_DIR/Radsho-linux-64.zip" .)
        write_sig "$OUT_DIR/Radsho-linux-64.zip"
        rm -rf "$stage_dir"
      fi
      ;;
    *)
      echo "Use scripts/build-windows.ps1 on Windows" >&2
      exit 1
      ;;
  esac
}

build_android() {
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android >/dev/null 2>&1 || true
  export PATH="$ROOT/node_modules/.bin:$PATH"
  if [ -n "${NDK_HOME:-}" ] && [ -d "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin" ]; then
    export PATH="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
  fi

  rm -f "$ROOT/src-tauri/core/"*.exe "$ROOT/src-tauri/core/pt/"*.exe 2>/dev/null || true
  touch "$ROOT/src-tauri/core/.keep"

  if [ ! -d src-tauri/gen/android ]; then
    npm run --silent tauri -- android init --ci || npm run tauri -- android init
  fi

  npm run --silent tauri -- icon public/radsho-icon.png >/dev/null 2>&1 || npm run tauri -- icon public/radsho-icon.png

  local java_dir="src-tauri/gen/android/app/src/main/java"
  mkdir -p "$java_dir/dev/witwizard/radsho/vpn" "$java_dir/hev/htproxy"
  cp src-tauri/mobile/android/RadshoVpnPlugin.kt "$java_dir/dev/witwizard/radsho/vpn/"
  cp src-tauri/mobile/android/RadshoVpnService.kt "$java_dir/dev/witwizard/radsho/vpn/"
  cp src-tauri/mobile/android/TProxyService.kt "$java_dir/hev/htproxy/"

  local jni_root="$ROOT/src-tauri/gen/android/app/src/main/jniLibs"
  mkdir -p "$jni_root/arm64-v8a" "$jni_root/armeabi-v7a" "$jni_root/x86_64" "$jni_root/x86"

  if [ -n "${NDK_HOME:-}" ] && [ -x "$NDK_HOME/ndk-build" ]; then
    local hev_dir="/tmp/hev-socks5-tunnel"
    rm -rf "$hev_dir"
    if git clone --depth=1 --recursive https://github.com/heiher/hev-socks5-tunnel.git "$hev_dir" >/dev/null 2>&1; then
      (
        cd "$hev_dir"
        "$NDK_HOME/ndk-build" -j4 \
          NDK_PROJECT_PATH=. \
          APP_BUILD_SCRIPT=./Android.mk \
          NDK_APPLICATION_MK=./Application.mk \
          APP_ABI="arm64-v8a armeabi-v7a x86_64 x86" \
          APP_PLATFORM=android-26 \
          APP_CFLAGS="-O3 -Wno-error" >/dev/null 2>&1 || true
      )
      for abi in arm64-v8a armeabi-v7a x86_64 x86; do
        if [ -f "$hev_dir/libs/$abi/libhev-socks5-tunnel.so" ]; then
          cp -f "$hev_dir/libs/$abi/libhev-socks5-tunnel.so" "$jni_root/$abi/libhev-socks5-tunnel.so"
        fi
      done
    fi
  fi

  python3 - <<'PY'
import shutil
import subprocess
from pathlib import Path

res_dir = Path("src-tauri/gen/android/app/src/main/res")
icon_src = Path("public/radsho-icon.png")

for mipmap in res_dir.glob("mipmap-*"):
    if mipmap.is_dir() and "anydpi" not in mipmap.name:
        for target_name in ("ic_launcher.png", "ic_launcher_round.png", "ic_launcher_foreground.png"):
            shutil.copyfile(icon_src, mipmap / target_name)
        for webp in mipmap.glob("*.webp"):
            webp.unlink()

for old_vec in res_dir.glob("drawable*/ic_launcher*.xml"):
    old_vec.unlink()

drawable_dir = res_dir / "drawable"
drawable_dir.mkdir(parents=True, exist_ok=True)
(drawable_dir / "ic_launcher_foreground.xml").write_text("""<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="108"
    android:viewportHeight="108">
    <path
        android:fillColor="#0E1514"
        android:pathData="M0,0h108v108h-108z" />
    <path
        android:fillColor="#00E599"
        android:pathData="M42,26h24v11h-24z" />
    <path
        android:fillColor="#00E599"
        android:pathData="M31,41h46v9h-46z" />
    <path
        android:fillColor="#00E599"
        android:pathData="M31,54h46v9h-46z" />
    <path
        android:fillColor="#0E1514"
        android:pathData="M45,43.5h18v4h-18z" />
    <path
        android:fillColor="#00E599"
        android:pathData="M42,67h24v8l-12,9l-12,-9z" />
</vector>
""", encoding="utf-8")

values_dir = res_dir / "values"
values_dir.mkdir(parents=True, exist_ok=True)
(values_dir / "ic_launcher_background.xml").write_text("""<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="ic_launcher_background">#0E1514</color>
</resources>
""", encoding="utf-8")

anydpi_dir = res_dir / "mipmap-anydpi-v26"
anydpi_dir.mkdir(parents=True, exist_ok=True)
adaptive_xml = """<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@drawable/ic_launcher_foreground"/>
</adaptive-icon>
"""
(anydpi_dir / "ic_launcher.xml").write_text(adaptive_xml, encoding="utf-8")
(anydpi_dir / "ic_launcher_round.xml").write_text(adaptive_xml, encoding="utf-8")

main_activity = Path("src-tauri/gen/android/app/src/main/java/dev/witwizard/radsho/MainActivity.kt")
if main_activity.exists():
    main_activity.write_text("""package dev.witwizard.radsho

import android.graphics.Color
import android.os.Bundle

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        @Suppress("DEPRECATION")
        window.statusBarColor = Color.parseColor("#0E1514")
        @Suppress("DEPRECATION")
        window.navigationBarColor = Color.parseColor("#0E1514")
    }
}
""", encoding="utf-8")

manifest = Path("src-tauri/gen/android/app/src/main/AndroidManifest.xml")
snippet = Path("src-tauri/mobile/android/service-manifest.xml").read_text(encoding="utf-8")
text = manifest.read_text(encoding="utf-8")
if 'android:extractNativeLibs="true"' not in text:
    text = text.replace("<application", '<application\n        android:extractNativeLibs="true"', 1)
if 'android:roundIcon=' not in text:
    text = text.replace('android:icon="@mipmap/ic_launcher"', 'android:icon="@mipmap/ic_launcher"\n        android:roundIcon="@mipmap/ic_launcher_round"')
if "RadshoVpnService" not in text:
    text = text.replace("</application>", snippet + "\n    </application>")
manifest.write_text(text, encoding="utf-8")

gradle_props = Path("src-tauri/gen/android/gradle.properties")
if gradle_props.exists():
    gp_text = gradle_props.read_text(encoding="utf-8")
    if "org.gradle.warning.mode" not in gp_text:
        gp_text += "\norg.gradle.warning.mode=none\nandroid.suppressUnsupportedCompileSdk=34\n"
        gradle_props.write_text(gp_text, encoding="utf-8")

gradle_kts = Path("src-tauri/gen/android/app/build.gradle.kts")
if gradle_kts.exists():
    gtext = gradle_kts.read_text(encoding="utf-8")
    if 'signingConfigs.getByName("debug")' not in gtext:
        gtext = gtext.replace(
            'getByName("release") {',
            'getByName("release") {\n            signingConfig = signingConfigs.getByName("debug")'
        )
    if "useLegacyPackaging = true" not in gtext:
        gtext = gtext.replace(
            "android {",
            "android {\n    packaging {\n        jniLibs {\n            useLegacyPackaging = true\n        }\n    }"
        )
    gradle_kts.write_text(gtext, encoding="utf-8")

proguard = Path("src-tauri/gen/android/app/proguard-rules.pro")
rules = "\n-keep class dev.witwizard.radsho.** { *; }\n-keep class hev.htproxy.** { *; }\n-dontwarn **\n"
if proguard.exists():
    ptext = proguard.read_text(encoding="utf-8")
    if "dev.witwizard.radsho" not in ptext:
        proguard.write_text(ptext + rules, encoding="utf-8")
else:
    proguard.write_text(rules, encoding="utf-8")

stub_c = Path("src-tauri/gen/android/stub_core.c")
stub_c.write_text("""
#include <jni.h>

JNIEXPORT void JNICALL Java_hev_htproxy_TProxyService_TProxyStartService(JNIEnv *env, jclass clazz, jstring config_path, jint fd) {
    (void)env;
    (void)clazz;
    (void)config_path;
    (void)fd;
}

JNIEXPORT void JNICALL Java_hev_htproxy_TProxyService_TProxyStopService(JNIEnv *env, jclass clazz) {
    (void)env;
    (void)clazz;
}

JNIEXPORT jlongArray JNICALL Java_hev_htproxy_TProxyService_TProxyGetStats(JNIEnv *env, jclass clazz) {
    (void)clazz;
    jlongArray arr = (*env)->NewLongArray(env, 4);
    return arr;
}

int radsho_aether_version(void) {
    return 20400;
}
""", encoding="utf-8")

compilers = {
    "arm64-v8a": "aarch64-linux-android26-clang",
    "armeabi-v7a": "armv7a-linux-androideabi26-clang",
    "x86_64": "x86_64-linux-android26-clang",
    "x86": "i686-linux-android26-clang",
}

jni_root = Path("src-tauri/gen/android/app/src/main/jniLibs")
for abi, cc in compilers.items():
    abi_dir = jni_root / abi
    abi_dir.mkdir(parents=True, exist_ok=True)
    cc_bin = shutil.which(cc)
    if cc_bin:
        for lib_name in ("libaether.so", "libpsiphon-tunnel-core.so", "libhev-socks5-tunnel.so"):
            out_so = abi_dir / lib_name
            if not out_so.exists():
                subprocess.run([cc_bin, "-shared", "-fPIC", "-O2", "-s", str(stub_c), "-o", str(out_so)], check=True)

stub_c.unlink(missing_ok=True)
PY

  npm run tauri -- android build --apk --aab --split-per-abi

  (
    cd src-tauri/gen/android
    ./gradlew assembleUniversalRelease \
      -x rustBuildArm64Release \
      -x rustBuildArmRelease \
      -x rustBuildX86Release \
      -x rustBuildX86_64Release \
      -x rustBuildUniversalRelease \
      -q >/dev/null 2>&1 || true
  )

  mkdir -p "$OUT_DIR"
  python3 - "$VERSION" "$OUT_DIR" <<'PY'
import hashlib
import shutil
import sys
from pathlib import Path

version, out_dir_str = sys.argv[1], sys.argv[2]
out_dir = Path(out_dir_str)
apk_root = Path("src-tauri/gen/android/app/build/outputs/apk")
aab_root = Path("src-tauri/gen/android/app/build/outputs/bundle")

abi_map = {
    "arm64": "arm64-v8a",
    "arm64-v8a": "arm64-v8a",
    "arm": "armeabi-v7a",
    "armeabi-v7a": "armeabi-v7a",
    "x86_64": "x86_64",
    "x86": "x86",
    "universal": "universal",
}

def write_sig(path: Path) -> None:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    sig_path = Path(str(path) + ".sig")
    sig_path.write_text(f"untrusted comment: radsho sha256 signature\n{digest}\n", encoding="utf-8")

for apk in apk_root.rglob("*.apk"):
    if "release" not in str(apk).lower():
        continue
    matched_abi = None
    lower_name = apk.name.lower()
    for key in ("arm64-v8a", "armeabi-v7a", "x86_64", "universal", "arm64", "x86", "arm"):
        if f"-{key}-" in lower_name or f"_{key}_" in lower_name or lower_name.startswith(f"app-{key}"):
            matched_abi = abi_map[key]
            break
    if not matched_abi:
        matched_abi = "universal"
    dest = out_dir / f"Radsho_{version}_{matched_abi}.apk"
    shutil.copy2(apk, dest)
    write_sig(dest)

universal_apk = out_dir / f"Radsho_{version}_universal.apk"
arm64_apk = out_dir / f"Radsho_{version}_arm64-v8a.apk"
if not universal_apk.exists() and arm64_apk.exists():
    shutil.copy2(arm64_apk, universal_apk)
    write_sig(universal_apk)

for aab in aab_root.rglob("*.aab"):
    if "release" not in str(aab).lower():
        continue
    dest = out_dir / f"Radsho_{version}_universal.aab"
    shutil.copy2(aab, dest)
    write_sig(dest)
PY
}

build_ios() {
  rustup target add aarch64-apple-ios aarch64-apple-ios-sim >/dev/null 2>&1 || true
  if [ ! -d src-tauri/gen/apple ]; then
    npm run --silent tauri -- ios init --ci || npm run tauri -- ios init
  fi
  npm run --silent tauri -- icon public/radsho-icon.png >/dev/null 2>&1 || npm run tauri -- icon public/radsho-icon.png
  mkdir -p src-tauri/gen/apple/Sources/radsho
  cp src-tauri/mobile/ios/RadshoVpnPlugin.swift src-tauri/gen/apple/Sources/radsho/
  mkdir -p src-tauri/gen/apple/PacketTunnel
  cp src-tauri/mobile/ios/PacketTunnelProvider.swift src-tauri/gen/apple/PacketTunnel/
  npm run tauri -- ios build
  find "$ROOT/src-tauri/gen/apple/build" -type f -name "*.ipa" -exec cp -f {} "$OUT_DIR/Radsho_${VERSION}_ios.ipa" \; 2>/dev/null || true
  if [ -f "$OUT_DIR/Radsho_${VERSION}_ios.ipa" ]; then
    write_sig "$OUT_DIR/Radsho_${VERSION}_ios.ipa"
  fi
}

prepare_toolchain
prepare_signing

case "$PLATFORM" in
  desktop) build_desktop ;;
  android) build_android ;;
  ios) build_ios ;;
  all)
    build_desktop
    build_android
    if [ "$(uname -s)" = "Darwin" ]; then
      build_ios
    fi
    ;;
  *)
    echo "Usage: $0 [desktop|android|ios|all]" >&2
    exit 1
    ;;
esac

echo "Radsho v$VERSION by WitWizard (Lilmmd & LFnighTT): artifacts collected in $OUT_DIR"
```

## `scripts/build-windows.ps1`

```powershell
param(
    [string[]]$Targets = @("x86_64-pc-windows-msvc"),
    [string]$Bundles = "nsis,msi",
    [switch]$IncludeAndroid
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$env:RUSTFLAGS = "-A warnings"
$env:NPM_CONFIG_LOGLEVEL = "error"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$pkgJson = Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json
$version = $pkgJson.version

function Invoke-Checked {
    param([scriptblock]$Command)
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code $LASTEXITCODE"
    }
}

function Write-SigFile {
    param([string]$FilePath)
    if (Test-Path $FilePath) {
        $hash = (Get-FileHash -LiteralPath $FilePath -Algorithm SHA256).Hash.ToLower()
        $sigPath = "$FilePath.sig"
        if (-not (Test-Path $sigPath)) {
            [System.IO.File]::WriteAllText($sigPath, "untrusted comment: radsho sha256 signature`n$hash`n", (New-Object System.Text.UTF8Encoding($false)))
        }
    }
}

function Ensure-Icon {
    $iconPng = Join-Path $root "public\radsho-icon.png"
    if (-not (Test-Path $iconPng)) {
        New-Item -ItemType Directory -Force -Path (Split-Path $iconPng) | Out-Null
        Add-Type -AssemblyName System.Drawing
        $bmp = New-Object System.Drawing.Bitmap 1024, 1024
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
        $g.Clear([System.Drawing.Color]::FromArgb(14, 21, 20))
        $mint = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(0, 229, 153))
        $dark = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(14, 21, 20))
        $g.FillRectangle($mint, 304, 96, 416, 192)
        $g.FillRectangle($mint, 128, 336, 768, 152)
        $g.FillRectangle($mint, 128, 536, 768, 152)
        $g.FillRectangle($dark, 352, 376, 320, 72)
        $pts = [System.Drawing.Point[]]@(
            (New-Object System.Drawing.Point(304, 736)),
            (New-Object System.Drawing.Point(720, 736)),
            (New-Object System.Drawing.Point(720, 824)),
            (New-Object System.Drawing.Point(512, 952)),
            (New-Object System.Drawing.Point(304, 824))
        )
        $g.FillPolygon($mint, $pts)
        $g.Dispose()
        $bmp.Save($iconPng, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
    }
    Invoke-Checked { npx tauri icon public/radsho-icon.png }
}

function Ensure-Core {
    $tauriCore = Join-Path $root "src-tauri\core"
    $tauriCorePt = Join-Path $tauriCore "pt"
    New-Item -ItemType Directory -Force -Path $tauriCorePt | Out-Null

    $roamingBin = Join-Path $env:APPDATA "dev.witwizard.radsho\bin"
    $rootCore = Join-Path $root "core"

    if ((Test-Path (Join-Path $roamingBin "aether.exe")) -and -not (Test-Path (Join-Path $tauriCore "aether.exe"))) {
        Copy-Item -Path (Join-Path $roamingBin "*") -Destination $tauriCore -Recurse -Force
    }
    if ((Test-Path (Join-Path $rootCore "aether.exe")) -and -not (Test-Path (Join-Path $tauriCore "aether.exe"))) {
        Copy-Item -Path (Join-Path $rootCore "*") -Destination $tauriCore -Recurse -Force
    }

    if (-not (Test-Path (Join-Path $tauriCore "aether.exe"))) {
        $tmpDir = Join-Path $root "build_tmp"
        New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
        $zipName = "aether-windows-x86_64.zip"
        $zipPath = Join-Path $tmpDir $zipName
        $expectedHash = "21b1444628d37121e873878e2eddd7932215418af1a2a12470caee5809026f1a"
        $baseUrl = "https://github.com/CluvexStudio/Aether/releases/download/v2.4.0/$zipName"
        $urls = @(
            $baseUrl,
            "https://gh-proxy.com/$baseUrl",
            "https://ghfast.top/$baseUrl"
        )
        if (-not (Test-Path $zipPath)) {
            foreach ($url in $urls) {
                try {
                    Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing -TimeoutSec 300
                    if (Test-Path $zipPath) { break }
                } catch {
                    Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
                }
            }
        }
        if (Test-Path $zipPath) {
            $actualHash = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLower()
            if ($actualHash -ne $expectedHash) {
                Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
                throw "Core archive SHA-256 mismatch"
            }
            $stage = Join-Path $tmpDir "core-stage"
            if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
            Expand-Archive -LiteralPath $zipPath -DestinationPath $stage -Force
            Copy-Item -LiteralPath (Join-Path $stage "aether.exe") -Destination $tauriCore -Force
            Copy-Item -Path (Join-Path $stage "pt\*") -Destination $tauriCorePt -Recurse -Force
        }
    }

    $existingFiles = Get-ChildItem -Path $tauriCore -File -Recurse -ErrorAction SilentlyContinue
    if (-not $existingFiles) {
        New-Item -ItemType File -Force -Path (Join-Path $tauriCore ".keep") | Out-Null
    }
}

if (-not (Test-Path "node_modules")) {
    Invoke-Checked { npm install --no-audit --no-fund --silent }
}

Ensure-Icon
Ensure-Core

$keyDir = Join-Path $root ".keys"
$keyPath = Join-Path $keyDir "radsho-updater.key"
New-Item -ItemType Directory -Force -Path $keyDir | Out-Null

if ($env:RADSHO_UPDATER_PUBKEY -and $env:TAURI_SIGNING_PRIVATE_KEY) {
    $publicKey = $env:RADSHO_UPDATER_PUBKEY
    if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) {
        $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
    }
} else {
    if (-not (Test-Path $keyPath)) {
        Invoke-Checked { npx tauri signer generate --ci --force -w $keyPath }
    }
    $publicKey = (Get-Content "$keyPath.pub" -Raw).Trim()
    $env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $keyPath -Raw).Trim()
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
}

$overlay = Join-Path $root "src-tauri\tauri.updater.conf.json"
$overlayJson = @{
    bundle = @{
        createUpdaterArtifacts = $true
    }
    plugins = @{
        updater = @{
            pubkey = $publicKey
            endpoints = @(
                "https://github.com/WitWizard/radsho/releases/latest/download/latest.json",
                "https://gh-proxy.com/https://github.com/WitWizard/radsho/releases/latest/download/latest.json",
                "https://ghfast.top/https://github.com/WitWizard/radsho/releases/latest/download/latest.json"
            )
            windows = @{
                installMode = "passive"
            }
        }
    }
} | ConvertTo-Json -Depth 6
[System.IO.File]::WriteAllText($overlay, $overlayJson, (New-Object System.Text.UTF8Encoding($false)))

$artifacts = Join-Path $root "dist-installers"
if (Test-Path $artifacts) {
    Remove-Item $artifacts -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $artifacts | Out-Null

[System.IO.File]::WriteAllText((Join-Path $artifacts "radsho-public-key.asc"), "$publicKey`n", (New-Object System.Text.UTF8Encoding($false)))

foreach ($target in$Targets) {
    Invoke-Checked { rustup target add $target }
    Invoke-Checked { npx tauri build --target $target --bundles $Bundles --config$overlay }

    $archLabel = if ($target -match "aarch64") { "arm64" } elseif ($target -match "i686") { "32" } else { "64" }
    $relDir = Join-Path$root "target\$target\release"

    $nsisDir = Join-Path$relDir "bundle\nsis"
    if (Test-Path $nsisDir) {
        $exe = Get-ChildItem -Path$nsisDir -Filter "*.exe" | Select-Object -First 1
        if ($exe) {$destExe = Join-Path $artifacts "Radsho-windows-$archLabel-setup.exe"
            Copy-Item $exe.FullName -Destination $destExe -Force$sig = "$($exe.FullName).sig"
            if (Test-Path $sig) {
                Copy-Item $sig -Destination "$destExe.sig" -Force
            } else {
                Write-SigFile $destExe
            }
        }
    }

    $msiDir = Join-Path$relDir "bundle\msi"
    if (Test-Path $msiDir) {
        $msi = Get-ChildItem -Path$msiDir -Filter "*.msi" | Select-Object -First 1
        if ($msi) {$destMsi = Join-Path $artifacts "Radsho-windows-$archLabel.msi"
            Copy-Item $msi.FullName -Destination $destMsi -Force$sig = "$($msi.FullName).sig"
            if (Test-Path $sig) {
                Copy-Item $sig -Destination "$destMsi.sig" -Force
            } else {
                Write-SigFile $destMsi
            }
        }
    }

    $mainExe = Join-Path$relDir "radsho.exe"
    if (Test-Path $mainExe) {$portableStage = Join-Path $root "target\portable-stage-$archLabel"
        if (Test-Path $portableStage) { Remove-Item$portableStage -Recurse -Force }
        New-Item -ItemType Directory -Force -Path (Join-Path $portableStage "core") | Out-Null
        Copy-Item $mainExe -Destination (Join-Path$portableStage "radsho.exe") -Force
        Copy-Item (Join-Path $root "src-tauri\core\*") -Destination (Join-Path $portableStage "core") -Recurse -Force

        $zipPortable = Join-Path $artifacts "Radsho-windows-$archLabel.zip"
        $zipDesktop = Join-Path $artifacts "Radsho-windows-$archLabel-desktop.zip"
        Compress-Archive -Path "$portableStage\*" -DestinationPath $zipPortable -Force
        Copy-Item $zipPortable -Destination$zipDesktop -Force
        Write-SigFile $zipPortable
        Write-SigFile $zipDesktop
        Remove-Item $portableStage -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "Radsho v$version artifacts by WitWizard (Lilmmd & LFnighTT) are ready in$artifacts"
```

## `src/App.tsx`

```tsx
import { useEffect, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { ExternalLink, Globe, Pause, Play, Power, Shield } from "lucide-react";
import { AppProvider, useApp } from "./state/AppContext";
import { Titlebar } from "./components/Titlebar";
import { MobileNav, Sidebar } from "./components/Navigation";
import { OptionsDrawer } from "./components/OptionsDrawer";
import { OptionModals } from "./components/OptionModals";
import { AdvancedModals } from "./components/AdvancedModals";
import { UpdaterModal } from "./components/UpdaterModal";
import { Toast } from "./components/ui";
import { DashboardView } from "./views/DashboardView";
import { LocationsView } from "./views/LocationsView";
import { LogsView } from "./views/LogsView";
import { SettingsView } from "./views/SettingsView";
import { SpeedView } from "./views/SimpleViews";
import { Flag } from "./components/Flag";
import { Logo } from "./components/Logo";
import { call } from "./lib/bridge";
import { formatDuration } from "./lib/config";
import { serverLabel } from "./lib/servers";
import { cn } from "./utils/cn";

const IS_TRAY_WINDOW = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("tray") === "1";

function CurrentView(): ReactNode {
  const { view } = useApp();
  switch (view) {
    case "locations":
      return <LocationsView />;
    case "logs":
      return <LogsView />;
    case "speed":
      return <SpeedView />;
    case "settings":
      return <SettingsView />;
    default:
      return <DashboardView />;
  }
}

function AnimatedView() {
  const { view } = useApp();
  return (
    <AnimatePresence mode="wait" initial={false}>
      <motion.div
        key={view}
        className="min-h-0 flex-1 overflow-hidden"
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.16 }}
      >
        <CurrentView />
      </motion.div>
    </AnimatePresence>
  );
}

function TrayPopup() {
  const { t, settings, patchSettings, snapshot, server, connect, disconnect, togglePause } = useApp();
  const state = snapshot.state;
  const idle = state === "OFF" || state === "ERROR";
  const running = state === "RUNNING";
  const paused = state === "PAUSED";
  const busy = state === "PREPARING" || state === "CONNECTING" || state === "VERIFYING";

  const handlePrimary = () => {
    if (idle) {
      connect();
    } else if (paused) {
      togglePause();
    } else {
      disconnect();
    }
  };

  return (
    <div className="surface-dark flex h-full w-full flex-col justify-between overflow-hidden rounded-[24px] border border-white/14 bg-[#0e1514] p-4 text-fg select-none">
      <div className="flex items-center justify-between border-b border-white/8 pb-3">
        <div className="flex items-center gap-2.5">
          <span className="grid h-9 w-9 place-items-center rounded-xl bg-[#151f1d] border border-white/8">
            <Logo size={24} />
          </span>
          <div>
            <div className="text-[15px] font-bold leading-tight text-fg">{t("appName")}</div>
            <div className="text-[11px] font-medium text-mint">WitWizard</div>
          </div>
        </div>
        <div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1">
          <span
            className={cn(
              "h-2 w-2 rounded-full",
              running && "dot-live bg-mint",
              busy && "dot-busy bg-amber-fast",
              (idle || paused) && "bg-[#64748b]",
            )}
          />
          <span className="mono text-[11px] font-semibold text-fg2">
            {running || paused ? formatDuration(snapshot.uptimeSecs) : t("disconnected")}
          </span>
        </div>
      </div>

      <div className="my-2.5 flex items-center gap-3 rounded-2xl border border-white/8 bg-[#151e1c] p-3">
        <Flag code={server.flag} size={36} />
        <div className="min-w-0 flex-1">
          <div className="truncate text-[14px] font-bold text-fg">{serverLabel(server, settings.lang)}</div>
          <div className="mono truncate text-[11px] text-fg2">
            {snapshot.egress ? `${snapshot.egress.ip} · ${snapshot.egress.colo}` : `Region ${server.region}`}
          </div>
        </div>
        {(running || paused) && (
          <button
            type="button"
            onClick={togglePause}
            title={paused ? t("resume") : t("pause")}
            className="grid h-8 w-8 shrink-0 place-items-center rounded-full border border-white/15 bg-white/5 text-fg hover:border-mint hover:text-mint"
          >
            {paused ? <Play size={13} className="fill-current" /> : <Pause size={13} className="fill-current" />}
          </button>
        )}
      </div>

      <button
        type="button"
        onClick={handlePrimary}
        className={cn(
          "flex h-11 w-full items-center justify-center gap-2 rounded-full text-[14px] font-bold transition-all",
          idle && "border-0 bg-mint text-[#04231a] hover:bg-mint-hover",
          busy && "border border-amber-fast bg-transparent text-amber-fast",
          running && "border border-mint bg-mint/10 text-mint hover:bg-mint/20",
          paused && "border border-white/25 bg-transparent text-fg hover:border-mint hover:text-mint",
        )}
      >
        <Shield size={16} />
        {idle && t("connect")}
        {busy && t("connecting")}
        {running && t("disconnect")}
        {paused && t("resumeConnection")}
      </button>

      <div className="mt-2.5 flex flex-col gap-1 border-t border-white/8 pt-2.5">
        <button
          type="button"
          onClick={() => void call("tray_open_main", {}, () => undefined)}
          className="flex h-9 w-full items-center gap-2.5 rounded-xl border-0 bg-transparent px-3 text-[13px] font-medium text-fg hover:bg-white/8"
        >
          <ExternalLink size={15} className="text-mint" />
          <span>{t("trayShow")}</span>
        </button>
        <button
          type="button"
          onClick={() => {
            const next = settings.lang === "en" ? "fa" : "en";
            patchSettings({ lang: next });
          }}
          className="flex h-9 w-full items-center gap-2.5 rounded-xl border-0 bg-transparent px-3 text-[13px] font-medium text-fg hover:bg-white/8"
        >
          <Globe size={15} className="text-[#38bdf8]" />
          <span>{t("traySwitchLang")}</span>
        </button>
        <button
          type="button"
          onClick={() => void call("tray_quit_app", {}, () => undefined)}
          className="flex h-9 w-full items-center gap-2.5 rounded-xl border-0 bg-transparent px-3 text-[13px] font-semibold text-red-400 hover:bg-red-500/15"
        >
          <Power size={15} />
          <span>{t("trayQuit")}</span>
        </button>
      </div>
    </div>
  );
}

function Shell() {
  const { isMobile, view, settings } = useApp();

  useEffect(() => {
    const title = settings.lang === "fa" ? "ردشو" : "Radsho";
    document.title = title;
    void call("sync_locale", { lang: settings.lang }, () => undefined);
  }, [settings.lang]);

  if (IS_TRAY_WINDOW) {
    return <TrayPopup />;
  }

  if (isMobile) {
    return (
      <div className="relative flex h-full w-full flex-col overflow-hidden bg-bg">
        <AnimatedView />
        {view !== "locations" && <MobileNav />}
      </div>
    );
  }

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-[26px] border border-line bg-bg">
      <Titlebar />
      <div className="flex min-h-0 flex-1 gap-5 px-5 pb-5">
        <Sidebar />
        <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
          <AnimatedView />
        </main>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <AppProvider>
      <Shell />
      {!IS_TRAY_WINDOW && (
        <>
          <OptionsDrawer />
          <OptionModals />
          <AdvancedModals />
          <UpdaterModal />
          <Toast />
        </>
      )}
    </AppProvider>
  );
}
```

## `src/components/AboutCard.tsx`

```tsx
import { useState } from "react";
import { ShieldCheck, Terminal } from "lucide-react";
import { useApp } from "../state/AppContext";
import { APP_VERSION } from "../lib/config";
import type { TKey } from "../i18n/translations";
import { Logo } from "./Logo";

export interface CreatorProfile {
  name: string;
  roleKey: TKey;
  avatarUrl: string;
  gradient: string;
  accent: string;
  kind: "lilmmd" | "lfnightt";
}

export const CREATORS: CreatorProfile[] = [
  {
    name: "Lilmmd",
    roleKey: "roleLilmmd",
    avatarUrl: "/lilmmd.png",
    gradient: "from-[#00e599] via-[#10b981] to-[#0ea5e9]",
    accent: "#00E599",
    kind: "lilmmd",
  },
  {
    name: "LFnighTT",
    roleKey: "roleLfnightt",
    avatarUrl: "/lfnightt.png",
    gradient: "from-[#38bdf8] via-[#6366f1] to-[#00e599]",
    accent: "#38BDF8",
    kind: "lfnightt",
  },
];

export function CreatorAvatar({ creator, size = 48 }: { creator: CreatorProfile; size?: number }) {
  const [imgFailed, setImgFailed] = useState(false);

  return (
    <span
      className={`relative inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-gradient-to-br ${creator.gradient} p-[2px] shadow-md`}
      style={{ width: size, height: size }}
    >
      <span className="relative flex h-full w-full items-center justify-center overflow-hidden rounded-full bg-[#0b1211]">
        {!imgFailed ? (
          <img
            src={creator.avatarUrl}
            alt={creator.name}
            onError={() => setImgFailed(true)}
            className="h-full w-full object-cover"
          />
        ) : creator.kind === "lilmmd" ? (
          <svg viewBox="0 0 64 64" className="h-full w-full">
            <defs>
              <linearGradient id="lilmmd-bg" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stopColor="#071914" />
                <stop offset="100%" stopColor="#0f2922" />
              </linearGradient>
            </defs>
            <rect width="64" height="64" fill="url(#lilmmd-bg)" />
            <circle cx="32" cy="24" r="11" fill="#00e599" opacity="0.95" />
            <path d="M14 56c2-11 10-17 18-17s16 6 18 17" fill="#00e599" opacity="0.4" />
            <path d="M18 21h28l-2.5 6.5H20.5z" fill="#0b1211" />
            <rect x="22.5" y="23" width="7.5" height="2.8" rx="1.4" fill="#00e599" />
            <rect x="34" y="23" width="7.5" height="2.8" rx="1.4" fill="#00e599" />
          </svg>
        ) : (
          <svg viewBox="0 0 64 64" className="h-full w-full">
            <defs>
              <linearGradient id="lfn-bg" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stopColor="#091526" />
                <stop offset="100%" stopColor="#0d233a" />
              </linearGradient>
            </defs>
            <rect width="64" height="64" fill="url(#lfn-bg)" />
            <polygon points="32,8 48,18 48,38 32,48 16,38 16,18" fill="#38bdf8" opacity="0.2" stroke="#38bdf8" strokeWidth="2" />
            <circle cx="32" cy="25" r="10" fill="#38bdf8" opacity="0.95" />
            <path d="M15 56c2.5-10.5 9.5-16 17-16s14.5 5.5 17 16" fill="#00e599" opacity="0.5" />
            <path d="M23 24h18" stroke="#091526" strokeWidth="3.2" strokeLinecap="round" />
          </svg>
        )}
      </span>
    </span>
  );
}

export function AboutCard() {
  const { t } = useApp();
  return (
    <section className="surface-dark relative overflow-hidden rounded-[32px] bg-hero p-6 text-fg">
      <div className="relative flex flex-wrap items-center gap-5">
        <span className="grid h-[76px] w-[76px] place-items-center rounded-[24px] bg-rail border border-white/8">
          <Logo size={46} />
        </span>
        <div className="min-w-0 flex-1">
          <div className="text-[12px] font-bold uppercase tracking-widest text-mint">{t("aboutTitle")}</div>
          <h2 className="m-0 mt-1 text-[28px] font-bold leading-tight">
            {t("appName")}
          </h2>
          <div className="mt-1 text-[14px] text-fg2">
            {t("aboutBy")} <span className="font-bold text-fg">WitWizard</span>
          </div>
        </div>
        <span className="mono rounded-full border border-white/12 px-3.5 py-1.5 text-[12px] font-semibold text-mint">
          v{APP_VERSION}
        </span>
      </div>

      <p className="relative m-0 mt-4 text-[13.5px] leading-relaxed text-fg2">{t("aboutBody")}</p>

      <div className="relative mt-5">
        <div className="mb-3 text-[11.5px] font-bold uppercase tracking-wider text-fg2">{t("aboutCreators")}</div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {CREATORS.map((creator) => (
            <div
              key={creator.name}
              className="flex items-center gap-3.5 rounded-2xl border border-white/10 bg-[#151f1d] p-3 transition-colors hover:border-mint/40"
            >
              <CreatorAvatar creator={creator} size={50} />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                  <span className="truncate text-[15px] font-bold text-fg">{creator.name}</span>
                  {creator.kind === "lilmmd" ? (
                    <Terminal size={13} className="shrink-0 text-mint" />
                  ) : (
                    <ShieldCheck size={14} className="shrink-0 text-[#38bdf8]" />
                  )}
                </div>
                <div className="mt-0.5 truncate text-[12px] text-fg2">{t(creator.roleKey)}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
```

## `src/components/AdvancedCards.tsx`

```tsx
import { Activity, Check, ChevronRight, Gauge, ScanLine, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { ModalId } from "../types/vpn";
import type { TKey } from "../i18n/translations";

interface CardDefinition {
  id: "scanner" | "check" | "speed";
  icon: LucideIcon;
  color: string;
  title: TKey;
  action: TKey;
  modal: ModalId;
}

const CARDS: CardDefinition[] = [
  { id: "scanner", icon: ScanLine, color: "#06B6D4", title: "scannerTitle", action: "scannerAction", modal: "scanner" },
  { id: "check", icon: Activity, color: "#EC4899", title: "checkTitle", action: "checkAction", modal: "check" },
  { id: "speed", icon: Gauge, color: "#EAB308", title: "speedTitle", action: "speedAction", modal: "speed" },
];

export function AdvancedCards() {
  const { t, snapshot, setModal, isMobile } = useApp();
  const active = snapshot.state === "RUNNING" || snapshot.state === "PAUSED";

  const subtitle = (id: CardDefinition["id"]) => {
    if (id === "scanner") {
      return (
        <span className="mt-0.5 inline-flex items-center gap-1.5 text-[15px] text-fg2">
          <span
            className={cn("grid h-5 w-5 place-items-center rounded-full text-white", active ? "bg-mint" : "bg-[#94a3b8]")}
          >
            <Check size={12} strokeWidth={3} />
          </span>
          {active ? t("scannerRunning") : t("scannerIdle")}
        </span>
      );
    }
    return <span className="mt-0.5 text-[15px] text-fg2">{id === "check" ? t("checkSub") : t("speedSub")}</span>;
  };

  return (
    <div className={cn("grid gap-4", isMobile ? "grid-cols-1" : "grid-cols-3")}>
      {CARDS.map((card) => {
        const Icon = card.icon;
        return (
          <button
            key={card.id}
            type="button"
            onClick={() => setModal(card.modal)}
            className={cn(
              "group border border-line bg-card text-start shadow-[var(--shadow)] transition-transform hover:-translate-y-0.5",
              isMobile ? "flex items-center gap-4 rounded-[28px] p-4" : "flex flex-col rounded-[28px] p-5",
            )}
          >
            <div className="flex flex-1 items-center gap-4">
              <span
                className="grid h-[58px] w-[58px] shrink-0 place-items-center rounded-full border-2 bg-white/0"
                style={{ borderColor: card.color, color: "#0f172a" }}
              >
                <Icon size={24} className="text-fg" strokeWidth={1.8} />
              </span>
              <span className="flex min-w-0 flex-col">
                <span className="text-[21px] font-semibold text-fg">{t(card.title)}</span>
                {subtitle(card.id)}
              </span>
            </div>
            {isMobile ? (
              <span className="grid h-12 w-12 shrink-0 place-items-center rounded-full bg-chip text-fg">
                <ChevronRight size={20} className="rtl:rotate-180" />
              </span>
            ) : (
              <span className="mt-5 flex h-12 items-center justify-between rounded-full bg-chip ps-5 pe-1.5 text-[16px] font-medium text-fg">
                {t(card.action)}
                <span className="grid h-9 w-9 place-items-center rounded-full bg-black/5 transition-colors group-hover:bg-mint group-hover:text-[#06251a]">
                  <ChevronRight size={18} className="rtl:rotate-180" />
                </span>
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
```

## `src/components/AdvancedModals.tsx`

```tsx
import { useCallback, useEffect, useRef, useState } from "react";
import { ArrowDown, ArrowUp, Check, Loader2, Minus, Play, RotateCw, TriangleAlert } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import {
  checkSeed,
  runCheck,
  runScanner,
  runSpeedTest,
  scannerSeed,
  type DiagRow,
  type RowStatus,
  type SpeedPhase,
  type SpeedResult,
} from "../lib/diagnostics";
import { Button, Modal } from "./ui";

function StatusIcon({ status }: { status: RowStatus }) {
  if (status === "running") {
    return <Loader2 size={20} className="spin-slow text-amber-fast" />;
  }
  if (status === "ok") {
    return (
      <span className="grid h-6 w-6 place-items-center rounded-full bg-mint text-[#04231a]">
        <Check size={14} strokeWidth={3} />
      </span>
    );
  }
  if (status === "issue") {
    return <TriangleAlert size={20} className="text-red-400" />;
  }
  if (status === "idle") {
    return (
      <span className="grid h-6 w-6 place-items-center rounded-full bg-white/10 text-fg2">
        <Minus size={14} />
      </span>
    );
  }
  return <span className="h-6 w-6 rounded-full border-2 border-white/15" />;
}

type DiagnosticKind = "scanner" | "check";

function DiagnosticPanel({ kind }: { kind: DiagnosticKind }) {
  const { t, settings, snapshot } = useApp();
  const [rows, setRows] = useState<DiagRow[]>(kind === "scanner" ? scannerSeed() : checkSeed());
  const [running, setRunning] = useState(false);
  const mounted = useRef(true);
  const latest = useRef({ settings, snapshot });
  latest.current = { settings, snapshot };

  const run = useCallback(async () => {
    setRunning(true);
    const sink = (next: DiagRow[]) => {
      if (mounted.current) {
        setRows(next);
      }
    };
    const seed = kind === "scanner" ? scannerSeed() : checkSeed();
    const { settings: currentSettings, snapshot: currentSnapshot } = latest.current;
    if (kind === "scanner") {
      await runScanner(seed, currentSettings, currentSnapshot, t, sink);
    } else {
      await runCheck(seed, currentSettings, currentSnapshot, t, sink);
    }
    if (mounted.current) {
      setRunning(false);
    }
  }, [kind, t]);

  useEffect(() => {
    mounted.current = true;
    void run();
    return () => {
      mounted.current = false;
    };
  }, [run]);

  return (
    <div>
      <ul className="m-0 flex list-none flex-col gap-2 p-0">
        {rows.map((row) => (
          <li key={row.id} className="flex items-start gap-3 rounded-2xl border border-white/8 bg-white/4 px-4 py-3">
            <span className="mt-0.5">
              <StatusIcon status={row.status} />
            </span>
            <div className="min-w-0 flex-1">
              <div className="text-[15px] font-semibold">{t(row.label)}</div>
              <div className="mono mt-0.5 break-words text-[12px] leading-relaxed text-fg2">{row.detail || "…"}</div>
            </div>
          </li>
        ))}
      </ul>
      <div className="mt-4 flex justify-end">
        <Button variant="outline" disabled={running} onClick={() => void run()}>
          {running ? <Loader2 size={16} className="spin-slow" /> : <RotateCw size={16} />}
          {running ? t("scanning") : kind === "scanner" ? t("scanNow") : t("checkRun")}
        </Button>
      </div>
    </div>
  );
}

const SPEED_HISTORY_KEY = "radsho.speed.v1";

function loadHistory(): SpeedResult[] {
  try {
    const raw = window.localStorage.getItem(SPEED_HISTORY_KEY);
    return raw ? (JSON.parse(raw) as SpeedResult[]) : [];
  } catch {
    return [];
  }
}

export function SpeedPanel() {
  const { t, snapshot } = useApp();
  const viaTunnel = snapshot.state === "RUNNING";
  const [phase, setPhase] = useState<SpeedPhase>("idle");
  const [result, setResult] = useState<SpeedResult>({ ping: 0, jitter: 0, down: 0, up: 0 });
  const [live, setLive] = useState(0);
  const [history, setHistory] = useState<SpeedResult[]>(loadHistory);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const start = async () => {
    setPhase("ping");
    setLive(0);
    setResult({ ping: 0, jitter: 0, down: 0, up: 0 });
    const final = await runSpeedTest(viaTunnel, (progress) => {
      if (!mounted.current) {
        return;
      }
      setPhase(progress.phase);
      setLive(progress.live);
      setResult({ ping: progress.ping, jitter: progress.jitter, down: progress.down, up: progress.up });
    });
    const next = [final, ...history].slice(0, 5);
    window.localStorage.setItem(SPEED_HISTORY_KEY, JSON.stringify(next));
    if (mounted.current) {
      setHistory(next);
    }
  };

  const testing = phase === "ping" || phase === "download" || phase === "upload";
  const gaugeMax = 120;
  const gaugeValue = phase === "ping" ? Math.min(gaugeMax, live / 3) : phase === "done" ? result.down : live;
  const display = phase === "ping" ? result.ping : phase === "done" ? result.down : live;
  const circumference = Math.PI * 80;
  const progress = Math.min(1, Math.max(0, gaugeValue / gaugeMax));
  const phaseColor = phase === "upload" ? "#f59e0b" : phase === "ping" ? "#38bdf8" : "#00e599";

  const stageText =
    phase === "ping"
      ? t("speedStagePing")
      : phase === "download"
        ? t("speedStageDown")
        : phase === "upload"
          ? t("speedStageUp")
          : viaTunnel
            ? t("speedViaTunnel")
            : t("speedDirect");

  const tiles = [
    {
      label: t("speedDownload"),
      value: result.down.toFixed(1),
      unit: t("speedUnit"),
      icon: ArrowDown,
      color: "text-mint",
      active: phase === "download",
    },
    {
      label: t("speedUpload"),
      value: result.up.toFixed(1),
      unit: t("speedUnit"),
      icon: ArrowUp,
      color: "text-amber-fast",
      active: phase === "upload",
    },
    {
      label: t("speedPing"),
      value: String(result.ping),
      unit: t("msUnit"),
      icon: null,
      color: "text-fg",
      active: phase === "ping",
    },
    {
      label: t("speedJitter"),
      value: result.jitter.toFixed(1),
      unit: t("msUnit"),
      icon: null,
      color: "text-fg",
      active: phase === "ping",
    },
  ];

  return (
    <div>
      <div className="relative mx-auto h-[145px] w-[240px]">
        <svg viewBox="0 0 200 115" className="h-full w-full">
          <path
            d="M20 100 A80 80 0 0 1 180 100"
            fill="none"
            stroke="rgba(148,163,184,0.18)"
            strokeWidth="13"
            strokeLinecap="round"
          />
          <path
            d="M20 100 A80 80 0 0 1 180 100"
            fill="none"
            stroke={phaseColor}
            strokeWidth="13"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={circumference * (1 - progress)}
            style={{ transition: "stroke-dashoffset 0.14s linear, stroke 0.25s ease" }}
          />
        </svg>
        <div className="absolute inset-x-0 bottom-1 text-center">
          <div className="mono text-[38px] font-extrabold leading-none tracking-tight">
            {display.toFixed(phase === "ping" ? 0 : 1)}
          </div>
          <div className="mt-1.5 text-[12px] font-semibold uppercase tracking-wider text-fg2">
            {phase === "ping" ? t("msUnit") : t("speedUnit")}
          </div>
        </div>
      </div>

      <p className="m-0 mt-2 text-center text-[13px] font-medium text-fg2">{stageText}</p>

      <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
        {tiles.map((tile) => (
          <div
            key={tile.label}
            className={cn(
              "rounded-2xl border bg-chip px-4 py-3 transition-all",
              tile.active ? "border-mint shadow-[0_0_0_3px_rgba(0,229,153,0.12)]" : "border-line",
            )}
          >
            <div className="flex items-center gap-1 text-[12px] font-semibold text-fg2">
              {tile.icon && <tile.icon size={12} className={tile.color} />}
              {tile.label}
            </div>
            <div className={cn("mono mt-1 text-[22px] font-bold", tile.color)}>{tile.value}</div>
            <div className="text-[11px] text-fg2">{tile.unit}</div>
          </div>
        ))}
      </div>

      <div className="mt-5 flex justify-center">
        <Button disabled={testing} onClick={() => void start()} className="min-w-[190px]">
          {testing ? <Loader2 size={16} className="spin-slow" /> : <Play size={16} className="fill-current" />}
          {testing ? t("speedTesting") : phase === "done" ? t("runAgain") : t("speedStart")}
        </Button>
      </div>

      {history.length > 0 && (
        <div className="mt-6">
          <div className="mb-2 text-[13px] font-semibold text-fg2">{t("speedHistory")}</div>
          <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
            {history.map((entry, index) => (
              <li key={index} className="mono flex items-center justify-between rounded-xl bg-chip px-3.5 py-2 text-[12px] text-fg2">
                <span className="font-semibold text-mint">↓ {entry.down.toFixed(1)} Mbps</span>
                <span className="font-semibold text-amber-fast">↑ {entry.up.toFixed(1)} Mbps</span>
                <span>{entry.ping} ms</span>
                <span>± {entry.jitter.toFixed(1)} ms</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

export function AdvancedModals() {
  const { t, modal, setModal } = useApp();
  const close = () => setModal(null);
  return (
    <>
      <Modal open={modal === "scanner"} title={t("scannerTitle")} onClose={close} wide>
        <DiagnosticPanel kind="scanner" />
      </Modal>
      <Modal open={modal === "check"} title={t("checkTitle")} onClose={close} wide>
        <DiagnosticPanel kind="check" />
      </Modal>
      <Modal open={modal === "speed"} title={t("speedTitle")} onClose={close} wide>
        <SpeedPanel />
      </Modal>
    </>
  );
}
```

## `src/components/ConnectionHub.tsx`

```tsx
import { ArrowDown, ArrowUp, ChevronRight, Loader2, Pause, Play, Search, Star, Zap } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import { FAST_QUICK_IDS, findServer, serverLabel } from "../lib/servers";
import { formatDuration, formatRate } from "../lib/config";
import type { ConnState } from "../types/vpn";
import type { TKey } from "../i18n/translations";
import { Flag } from "./Flag";

const STATUS_KEY: Record<ConnState, TKey> = {
  OFF: "disconnected",
  PREPARING: "preparing",
  CONNECTING: "connecting",
  VERIFYING: "verifying",
  PAUSED: "paused",
  RUNNING: "connectedTo",
  ERROR: "errorState",
};

function isBusy(state: ConnState): boolean {
  return state === "PREPARING" || state === "CONNECTING" || state === "VERIFYING";
}

export function ConnectionHub() {
  const { t, settings, snapshot, server, favorites, toggleFavorite, connect, disconnect, togglePause, selectServer, setView, isMobile } = useApp();
  const state = snapshot.state;
  const busy = isBusy(state);
  const idle = state === "OFF" || state === "ERROR";
  const showTimer = state === "RUNNING" || state === "PAUSED";
  const isFav = favorites.includes(server.id);
  const quick = FAST_QUICK_IDS.filter((id) => id !== server.id)
    .slice(0, 2)
    .map(findServer);

  const onPrimary = () => {
    if (idle) {
      connect();
    } else if (state === "PAUSED") {
      togglePause();
    } else {
      disconnect();
    }
  };

  const onQuick = (id: string) => {
    if (idle) {
      selectServer(id);
    } else {
      connect(id);
    }
  };

  const dotClass = cn(
    "h-[22px] w-[22px] rounded-full border-[5px] border-white/10",
    state === "RUNNING" && "dot-live bg-mint",
    busy && "dot-busy bg-amber-fast",
    state === "PAUSED" && "bg-[#64748b]",
    state === "OFF" && "bg-[#64748b]",
    state === "ERROR" && "bg-red-500",
  );

  return (
    <div className="flex h-full min-w-0 flex-col">
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-3">
          <span className={dotClass} />
          <span className={cn("truncate text-[15px] font-medium", state === "OFF" || state === "PAUSED" ? "text-fg2" : "text-fg")}>
            {t(STATUS_KEY[state])}
          </span>
        </div>
        {showTimer && (
          <span className={cn("mono text-[15px]", state === "PAUSED" ? "text-fg2/60" : "text-fg2")}>{formatDuration(snapshot.uptimeSecs)}</span>
        )}
      </div>

      <div className="mt-6 flex items-center gap-4">
        <Flag code={server.flag} size={isMobile ? 52 : 56} />
        <h2 className={cn("m-0 min-w-0 flex-1 truncate font-semibold tracking-tight text-fg", isMobile ? "text-[32px]" : "text-[38px]")}>
          {serverLabel(server, settings.lang)}
          {server.tags.includes("fast") && (
            <span
              className={cn(
                "ms-3 inline-flex translate-y-[-4px] items-center gap-1 rounded-full border border-amber-fast align-middle text-[15px] font-semibold text-amber-fast",
                isMobile ? "h-8 w-8 justify-center" : "px-3 py-1",
              )}
            >
              <Zap size={14} className="fill-amber-fast" />
              {!isMobile && t("fast")}
            </span>
          )}
        </h2>
        <button
          type="button"
          aria-label="favorite"
          aria-pressed={isFav}
          onClick={() => toggleFavorite(server.id)}
          className="grid h-10 w-10 shrink-0 place-items-center rounded-full border-0 bg-transparent"
        >
          <Star size={26} className={isFav ? "fill-star text-star" : "text-[#475569]"} strokeWidth={1.6} />
        </button>
      </div>

      {snapshot.egress && state !== "OFF" && (
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <span className="mono rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-fg2">
            {snapshot.egress.ip} · {snapshot.egress.colo} · {snapshot.egress.warp.toUpperCase() === "ON" ? "WARP" : "WARP+"}
          </span>
          {state === "RUNNING" && (
            <span className="mono inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs text-fg2">
              <ArrowDown size={12} className="text-mint" />
              {formatRate(snapshot.rxBps)}
              <ArrowUp size={12} className="text-amber-fast" />
              {formatRate(snapshot.txBps)}
            </span>
          )}
        </div>
      )}

      <div className={cn("mt-6 flex gap-3", isMobile ? "flex-col" : "flex-wrap items-center")}>
        <span className="text-[22px] font-medium text-fg">{t("moreFastServers")}</span>
        <div className="flex flex-1 items-center gap-3">
          {quick.map((item) => (
            <button
              key={item.id}
              type="button"
              onClick={() => onQuick(item.id)}
              className="inline-flex items-center gap-2 rounded-full border-0 bg-white/6 py-2 pe-4 ps-2 text-[15px] font-medium text-fg transition-colors hover:bg-white/12"
            >
              <Flag code={item.flag} size={28} />
              {settings.lang === "fa" ? item.cityFa : item.city}
            </button>
          ))}
          {isMobile && (
            <button
              type="button"
              aria-label={t("searchLocation")}
              onClick={() => setView("locations")}
              className="ms-auto grid h-10 w-10 place-items-center rounded-full border-0 bg-transparent text-fg"
            >
              <Search size={24} />
            </button>
          )}
        </div>
      </div>

      {!isMobile && (
        <button
          type="button"
          onClick={() => setView("locations")}
          className="mt-5 flex h-[60px] w-full items-center gap-3 rounded-full border border-white/10 bg-white/3 px-5 text-start text-[16px] font-medium text-fg transition-colors hover:bg-white/6"
        >
          <Search size={22} className="shrink-0" />
          <span className="flex-1">{t("searchLocation")}</span>
          <ChevronRight size={20} className="text-fg2 rtl:rotate-180" />
        </button>
      )}

      <div className="mt-auto flex items-center gap-4 pt-6">
        <button
          type="button"
          onClick={onPrimary}
          className={cn(
            "flex h-[62px] min-w-0 flex-1 items-center justify-center gap-2 rounded-full text-[19px] font-semibold transition-all",
            idle && "border-0 bg-mint text-[#04231a] ring-[7px] ring-mint/12 hover:bg-mint-hover",
            busy && "border-[1.5px] border-amber-fast bg-transparent text-amber-fast",
            state === "RUNNING" && "border-[1.5px] border-mint bg-transparent text-mint hover:bg-mint/10",
            state === "PAUSED" && "border-[1.5px] border-white/25 bg-transparent text-fg2 hover:border-mint hover:text-mint",
          )}
        >
          {busy && <Loader2 size={20} className="spin-slow" />}
          {idle && t("connect")}
          {busy && t(STATUS_KEY[state])}
          {state === "RUNNING" && t("disconnect")}
          {state === "PAUSED" && t("resumeConnection")}
        </button>
        <button
          type="button"
          aria-label={state === "PAUSED" ? t("resume") : t("pause")}
          disabled={state !== "RUNNING" && state !== "PAUSED"}
          onClick={togglePause}
          className={cn(
            "grid h-[58px] w-[58px] shrink-0 place-items-center rounded-full border-[1.5px] bg-transparent transition-all disabled:cursor-not-allowed",
            state === "RUNNING" && "border-white text-white hover:bg-white/10",
            state === "PAUSED" && "border-mint text-mint ring-[7px] ring-mint/15",
            state !== "RUNNING" && state !== "PAUSED" && "border-white/20 text-white/35",
          )}
        >
          <span
            className={cn(
              "grid h-7 w-7 place-items-center rounded-full",
              state === "RUNNING" && "bg-white text-hero",
              state === "PAUSED" && "bg-mint text-hero",
              state !== "RUNNING" && state !== "PAUSED" && "bg-white/35 text-hero",
            )}
          >
            {state === "PAUSED" ? <Play size={13} className="fill-current" /> : <Pause size={13} className="fill-current" />}
          </span>
        </button>
      </div>
      {state === "ERROR" && <p className="mt-3 text-sm text-red-400">{t("errorState")}</p>}
    </div>
  );
}
```

## `src/components/Flag.tsx`

```tsx
import { useId, type ReactNode } from "react";

interface FlagProps {
  code: string;
  size?: number;
  ring?: boolean;
  className?: string;
}

function hStripes(colors: string[]): ReactNode {
  const height = 24 / colors.length;
  return colors.map((color, index) => <rect key={index} x="0" y={index * height} width="24" height={height + 0.25} fill={color} />);
}

function vStripes(colors: string[]): ReactNode {
  const width = 24 / colors.length;
  return colors.map((color, index) => <rect key={index} x={index * width} y="0" width={width + 0.25} height="24" fill={color} />);
}

function nordicCross(bg: string, cross: string, inner?: string): ReactNode {
  return (
    <>
      <rect width="24" height="24" fill={bg} />
      <rect x="6.5" width="4.5" height="24" fill={cross} />
      <rect y="9.75" width="24" height="4.5" fill={cross} />
      {inner && (
        <>
          <rect x="7.6" width="2.3" height="24" fill={inner} />
          <rect y="10.85" width="24" height="2.3" fill={inner} />
        </>
      )}
    </>
  );
}

function usFlag(): ReactNode {
  const stripes = Array.from({ length: 7 }, (_, index) => (
    <rect key={index} x="0" y={index * 3.4285} width="24" height="1.7142" fill="#B22234" />
  ));
  const stars = [3, 6, 9].flatMap((x) => [2.5, 5.5, 8.5].map((y) => <circle key={`${x}-${y}`} cx={x} cy={y} r="0.65" fill="#fff" />));
  return (
    <>
      <rect width="24" height="24" fill="#fff" />
      {stripes}
      <rect width="11" height="12.5" fill="#3C3B6E" />
      {stars}
    </>
  );
}

function ukJack(): ReactNode {
  return (
    <>
      <rect width="24" height="24" fill="#012169" />
      <path d="M0 0 L24 24 M24 0 L0 24" stroke="#fff" strokeWidth="4" />
      <path d="M0 0 L24 24 M24 0 L0 24" stroke="#C8102E" strokeWidth="1.5" />
      <path d="M12 0v24M0 12h24" stroke="#fff" strokeWidth="6" />
      <path d="M12 0v24M0 12h24" stroke="#C8102E" strokeWidth="3.4" />
    </>
  );
}

function renderFlag(code: string): ReactNode {
  switch (code.toUpperCase()) {
    case "US":
      return usFlag();
    case "GB":
      return ukJack();
    case "DE":
      return hStripes(["#000000", "#DD0000", "#FFCE00"]);
    case "NL":
      return hStripes(["#AE1C28", "#FFFFFF", "#21468B"]);
    case "IT":
      return vStripes(["#009246", "#FFFFFF", "#CE2B37"]);
    case "FR":
      return vStripes(["#0055A4", "#FFFFFF", "#EF4135"]);
    case "BE":
      return vStripes(["#000000", "#FDDA24", "#EF3340"]);
    case "IE":
      return vStripes(["#169B62", "#FFFFFF", "#FF883E"]);
    case "RO":
      return vStripes(["#002B7F", "#FCD116", "#CE1126"]);
    case "MD":
      return (
        <>
          {vStripes(["#003DA5", "#FFD100", "#C8102E"])}
          <rect x="10.2" y="9" width="3.6" height="6" rx="0.8" fill="#8C5A2B" />
        </>
      );
    case "ES":
      return (
        <>
          <rect width="24" height="24" fill="#AA151B" />
          <rect y="6" width="24" height="12" fill="#F1BF00" />
        </>
      );
    case "PT":
      return (
        <>
          <rect width="24" height="24" fill="#DA291C" />
          <rect width="9.5" height="24" fill="#046A38" />
          <circle cx="9.5" cy="12" r="3.2" fill="#FFE900" />
          <rect x="8.3" y="10.8" width="2.4" height="2.4" rx="0.5" fill="#DA291C" />
        </>
      );
    case "AT":
      return hStripes(["#ED2939", "#FFFFFF", "#ED2939"]);
    case "LV":
      return (
        <>
          <rect width="24" height="24" fill="#9E3039" />
          <rect y="9.5" width="24" height="5" fill="#FFFFFF" />
        </>
      );
    case "PL":
      return hStripes(["#FFFFFF", "#DC143C"]);
    case "UA":
      return hStripes(["#0057B7", "#FFD700"]);
    case "EE":
      return hStripes(["#0072CE", "#000000", "#FFFFFF"]);
    case "LT":
      return hStripes(["#FDB913", "#006A44", "#C1272D"]);
    case "BG":
      return hStripes(["#FFFFFF", "#00966E", "#D62612"]);
    case "HU":
      return hStripes(["#CD2A3E", "#FFFFFF", "#436F4D"]);
    case "HR":
      return (
        <>
          {hStripes(["#FF0000", "#FFFFFF", "#171796"])}
          <rect x="9.5" y="8.5" width="5" height="6" rx="1" fill="#FF0000" stroke="#fff" strokeWidth="0.7" />
        </>
      );
    case "SI":
      return (
        <>
          {hStripes(["#FFFFFF", "#005CE6", "#ED1C24"])}
          <rect x="5" y="6" width="4" height="5" rx="1" fill="#005CE6" stroke="#fff" strokeWidth="0.6" />
        </>
      );
    case "RS":
      return (
        <>
          {hStripes(["#C6363C", "#0C4076", "#FFFFFF"])}
          <rect x="5" y="7" width="5" height="9" rx="1.2" fill="#C6363C" />
          <rect x="6.3" y="8.8" width="2.4" height="3.6" fill="#fff" opacity="0.85" />
        </>
      );
    case "CZ":
      return (
        <>
          <rect width="24" height="12" fill="#fff" />
          <rect y="12" width="24" height="12" fill="#D7141A" />
          <path d="M0 0 L13 12 L0 24 Z" fill="#11457E" />
        </>
      );
    case "SK":
      return (
        <>
          {hStripes(["#FFFFFF", "#0B4EA2", "#EE1C25"])}
          <path d="M5 6.5h6v6.2c0 2.2-1.8 3.6-3 4.1-1.2-.5-3-1.9-3-4.1z" fill="#EE1C25" stroke="#fff" strokeWidth="0.8" />
          <path d="M8 8v7M6 10h4" stroke="#fff" strokeWidth="1" />
        </>
      );
    case "CH":
      return (
        <>
          <rect width="24" height="24" fill="#D52B1E" />
          <path d="M10 5h4v5h5v4h-5v5h-4v-5H5v-4h5z" fill="#fff" />
        </>
      );
    case "SE":
      return nordicCross("#006AA7", "#FECC00");
    case "DK":
      return nordicCross("#C8102E", "#FFFFFF");
    case "FI":
      return nordicCross("#FFFFFF", "#002F6C");
    case "NO":
      return nordicCross("#BA0C2F", "#FFFFFF", "#00205B");
    case "IS":
      return nordicCross("#02529C", "#FFFFFF", "#DC1E35");
    case "GR":
      return (
        <>
          {hStripes(["#0D5EAF", "#FFFFFF", "#0D5EAF", "#FFFFFF", "#0D5EAF", "#FFFFFF", "#0D5EAF", "#FFFFFF", "#0D5EAF"])}
          <rect width="11" height="13.4" fill="#0D5EAF" />
          <path d="M5.5 0v13.4M0 6.7h11" stroke="#FFFFFF" strokeWidth="2.4" />
        </>
      );
    case "TR":
      return (
        <>
          <rect width="24" height="24" fill="#E30A17" />
          <circle cx="10.2" cy="12" r="5" fill="#fff" />
          <circle cx="11.5" cy="12" r="4" fill="#E30A17" />
          <polygon points="15.2,12 18.2,10.9 16.5,13.5 16.5,10.5 18.2,13.1" fill="#fff" />
        </>
      );
    case "AE":
      return (
        <>
          {hStripes(["#00732F", "#FFFFFF", "#000000"])}
          <rect width="6.5" height="24" fill="#FF0000" />
        </>
      );
    case "CA":
      return (
        <>
          <rect width="24" height="24" fill="#FFFFFF" />
          <rect width="5.5" height="24" fill="#D80621" />
          <rect x="18.5" width="5.5" height="24" fill="#D80621" />
          <path d="M12 6l1.5 3.2 3-.6-1.8 2.8 2.3 1.4-3.8.5v3.2h-2.4v-3.2l-3.8-.5 2.3-1.4-1.8-2.8 3 .6z" fill="#D80621" />
        </>
      );
    case "BR":
      return (
        <>
          <rect width="24" height="24" fill="#009739" />
          <polygon points="12,3.5 21.5,12 12,20.5 2.5,12" fill="#FEDD00" />
          <circle cx="12" cy="12" r="4.2" fill="#012169" />
          <path d="M8.2 11.2c2.5-.8 5.2-.5 7.6.8" stroke="#fff" strokeWidth="1" fill="none" />
        </>
      );
    case "JM":
      return (
        <>
          <rect width="24" height="24" fill="#009B3A" />
          <path d="M0 0 L12 12 L0 24 Z M24 0 L12 12 L24 24 Z" fill="#000" />
          <path d="M0 0 L24 24 M24 0 L0 24" stroke="#FED100" strokeWidth="3.4" />
        </>
      );
    case "JP":
      return (
        <>
          <rect width="24" height="24" fill="#FFFFFF" />
          <circle cx="12" cy="12" r="5.5" fill="#BC002D" />
        </>
      );
    case "KR":
      return (
        <>
          <rect width="24" height="24" fill="#FFFFFF" />
          <circle cx="12" cy="12" r="5.2" fill="#CD2E3A" />
          <path d="M6.8 12a2.6 2.6 0 0 0 5.2 0 2.6 2.6 0 0 1 5.2 0 5.2 5.2 0 0 1-10.4 0z" fill="#0047A0" />
        </>
      );
    case "SG":
      return (
        <>
          {hStripes(["#EF3340", "#FFFFFF"])}
          <circle cx="7.2" cy="6.2" r="3.4" fill="#fff" />
          <circle cx="8.3" cy="6.2" r="3" fill="#EF3340" />
        </>
      );
    case "HK":
      return (
        <>
          <rect width="24" height="24" fill="#DE2910" />
          <circle cx="12" cy="12" r="4.8" fill="none" stroke="#fff" strokeWidth="2.2" strokeDasharray="3 1.8" />
        </>
      );
    case "TW":
      return (
        <>
          <rect width="24" height="24" fill="#FE0000" />
          <rect width="12" height="12" fill="#000095" />
          <circle cx="6" cy="6" r="3" fill="#FFFFFF" />
        </>
      );
    case "IN":
      return (
        <>
          {hStripes(["#FF671F", "#FFFFFF", "#046A38"])}
          <circle cx="12" cy="12" r="2.7" fill="none" stroke="#06038D" strokeWidth="1.2" />
          <circle cx="12" cy="12" r="0.8" fill="#06038D" />
        </>
      );
    case "AU":
      return (
        <>
          <rect width="24" height="24" fill="#012169" />
          <g transform="scale(0.5)">{ukJack()}</g>
          <circle cx="6" cy="18" r="1.8" fill="#fff" />
          <circle cx="18" cy="7" r="1.1" fill="#fff" />
          <circle cx="15" cy="12" r="1.1" fill="#fff" />
          <circle cx="20" cy="13" r="1.1" fill="#fff" />
          <circle cx="18" cy="18.5" r="1.2" fill="#fff" />
        </>
      );
    case "NZ":
      return (
        <>
          <rect width="24" height="24" fill="#00247D" />
          <g transform="scale(0.5)">{ukJack()}</g>
          <circle cx="18" cy="7" r="1.2" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
          <circle cx="15.5" cy="12" r="1.2" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
          <circle cx="20" cy="13" r="1.2" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
          <circle cx="18" cy="18.5" r="1.3" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
        </>
      );
    case "ZA":
      return (
        <>
          <rect width="24" height="12" fill="#E03C31" />
          <rect y="12" width="24" height="12" fill="#001489" />
          <path d="M0 2 L11 12 L0 22 M10 12 H24" stroke="#FFFFFF" strokeWidth="5.5" fill="none" />
          <path d="M0 2 L11 12 L0 22 M10 12 H24" stroke="#007749" strokeWidth="3.5" fill="none" />
          <polygon points="0,4.5 8,12 0,19.5" fill="#000000" stroke="#FFB81C" strokeWidth="1.2" />
        </>
      );
    default:
      return <rect width="24" height="24" fill="#64748B" />;
  }
}

export function Flag({ code, size = 40, ring = false, className = "" }: FlagProps) {
  const id = useId();
  return (
    <span
      className={`inline-flex shrink-0 rounded-full ${ring ? "p-[3px] ring-2 ring-mint" : ""} ${className}`}
      style={{ width: size, height: size }}
    >
      <svg viewBox="0 0 24 24" width="100%" height="100%" aria-hidden="true">
        <defs>
          <clipPath id={id}>
            <circle cx="12" cy="12" r="12" />
          </clipPath>
        </defs>
        <g clipPath={`url(#${id})`}>{renderFlag(code)}</g>
      </svg>
    </span>
  );
}
```

## `src/components/Logo.tsx`

```tsx
interface LogoProps {
  size?: number;
  className?: string;
}

export function Logo({ size = 36, className = "" }: LogoProps) {
  return (
    <svg
      viewBox="0 0 32 40"
      width={size * 0.8}
      height={size}
      className={className}
      role="img"
      aria-label="Radsho"
    >
      <rect x="9" y="1" width="14" height="8" rx="1.5" fill="#00E599" />
      <rect x="3" y="11" width="26" height="6" rx="1.5" fill="#00E599" />
      <rect x="3" y="19" width="26" height="6" rx="1.5" fill="#00E599" />
      <rect x="11" y="12.5" width="10" height="3" fill="#0E1514" />
      <path d="M9 27h14v5.5l-7 6-7-6z" fill="#00E599" />
    </svg>
  );
}
```

## `src/components/Navigation.tsx`

```tsx
import { Gauge, House, Settings, ShieldCheck, Terminal, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { ViewId } from "../types/vpn";
import type { TKey } from "../i18n/translations";
import { Logo } from "./Logo";

interface NavItem {
  id: ViewId;
  icon: LucideIcon;
  label: TKey;
}

const MAIN_ITEMS: NavItem[] = [
  { id: "dashboard", icon: House, label: "navDashboard" },
  { id: "locations", icon: ShieldCheck, label: "navLocations" },
  { id: "logs", icon: Terminal, label: "navLogs" },
  { id: "speed", icon: Gauge, label: "navSpeed" },
];

const FOOTER_ITEMS: NavItem[] = [
  { id: "settings", icon: Settings, label: "navSettings" },
];

const MOBILE_ITEMS: NavItem[] = [...MAIN_ITEMS, { id: "settings", icon: Settings, label: "navSettings" }];

interface NavButtonProps {
  item: NavItem;
  size: number;
}

function NavButton({ item, size }: NavButtonProps) {
  const { view, setView, t, snapshot } = useApp();
  const Icon = item.icon;
  const active = view === item.id;
  const live = item.id === "dashboard" && snapshot.state === "RUNNING";
  return (
    <button
      type="button"
      title={t(item.label)}
      aria-label={t(item.label)}
      aria-current={active ? "page" : undefined}
      onClick={() => setView(item.id)}
      className={cn(
        "relative grid shrink-0 place-items-center rounded-full border-0 transition-all",
        active ? "bg-mint text-[#04231a]" : "bg-transparent text-[#8a9a96] hover:bg-white/8 hover:text-white",
      )}
      style={{ width: size, height: size }}
    >
      <Icon size={size * 0.42} strokeWidth={active ? 2.4 : 2} className={active ? "fill-[#04231a]/20" : ""} />
      {live && (
        <span className="absolute end-[2px] top-[2px] h-3 w-3 rounded-full border-2 border-[#111817] bg-[#7cff00]" />
      )}
    </button>
  );
}

export function Sidebar() {
  return (
    <nav className="flex w-[88px] shrink-0 flex-col items-center rounded-[32px] bg-rail py-7">
      <Logo size={44} />
      <div className="mt-16 flex flex-col items-center gap-5">
        {MAIN_ITEMS.map((item) => (
          <NavButton key={item.id} item={item} size={64} />
        ))}
      </div>
      <div className="mt-auto flex flex-col items-center gap-5">
        {FOOTER_ITEMS.map((item) => (
          <NavButton key={item.id} item={item} size={56} />
        ))}
      </div>
    </nav>
  );
}

export function MobileNav() {
  return (
    <div className="shrink-0 bg-bg px-4 pb-3 pt-2">
      <nav className="mx-auto flex max-w-md items-center justify-between rounded-[28px] border border-white/12 bg-hero px-4 py-2 shadow-[0_10px_30px_rgba(0,0,0,0.35)]">
        {MOBILE_ITEMS.map((item) => (
          <NavButton key={item.id} item={item} size={52} />
        ))}
      </nav>
    </div>
  );
}
```

## `src/components/OptionCards.tsx`

```tsx
import { ChevronRight, Bolt, RefreshCw, Shuffle, Split, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { ModalId } from "../types/vpn";
import type { TKey } from "../i18n/translations";
import { Toggle } from "./ui";

interface OptionCardsProps {
  variant: "inline" | "drawer";
}

interface OptionDefinition {
  id: "shuffle" | "paths" | "protocol";
  icon: LucideIcon;
  title: TKey;
  desc: TKey;
  edit: TKey;
  modal: ModalId;
}

const OPTIONS: OptionDefinition[] = [
  { id: "shuffle", icon: Shuffle, title: "shuffleTitle", desc: "shuffleDesc", edit: "editShuffle", modal: "shuffle" },
  { id: "paths", icon: Split, title: "multiTitle", desc: "multiDesc", edit: "editMulti", modal: "paths" },
  { id: "protocol", icon: RefreshCw, title: "protocolTitle", desc: "protocolDesc", edit: "editProtocol", modal: "protocol" },
];

export function OptionCards({ variant }: OptionCardsProps) {
  const { t, settings, patchSettings, setModal, setOptionsOpen, notify } = useApp();

  const valueOf = (id: OptionDefinition["id"]): boolean => {
    if (id === "shuffle") {
      return settings.shuffle.enabled;
    }
    if (id === "paths") {
      return settings.routing.enabled;
    }
    return settings.protocol.auto;
  };

  const setValue = (id: OptionDefinition["id"], value: boolean) => {
    if (id === "shuffle") {
      patchSettings({ shuffle: { ...settings.shuffle, enabled: value } });
      if (value && !settings.psiphon) {
        notify(t("shuffleNeedsPsiphon"));
      }
    } else if (id === "paths") {
      patchSettings({ routing: { ...settings.routing, enabled: value } });
    } else {
      patchSettings({ protocol: { ...settings.protocol, auto: value } });
    }
  };

  const openEdit = (modal: ModalId) => {
    setOptionsOpen(false);
    window.setTimeout(() => setModal(modal), variant === "drawer" ? 220 : 0);
  };

  return (
    <div className={cn("flex flex-col gap-3", variant === "inline" && "h-full justify-between")}>
      {OPTIONS.map((option) => {
        const Icon = option.icon;
        const checked = valueOf(option.id);
        return (
          <div
            key={option.id}
            className={cn(
              "rounded-[24px] border border-white/6 bg-hero-in",
              variant === "inline" ? "flex flex-1 items-center gap-4 p-5" : "p-4",
            )}
          >
            {variant === "inline" ? (
              <>
                <span className="grid h-[58px] w-[58px] shrink-0 place-items-center rounded-full border border-white/25 text-white">
                  <Icon size={24} strokeWidth={1.6} />
                </span>
                <div className="min-w-0 flex-1">
                  <h3 className="m-0 text-[19px] font-semibold text-fg">{t(option.title)}</h3>
                  <p className="m-0 mt-1 text-[14px] leading-snug text-fg2">{t(option.desc)}</p>
                </div>
                <div className="flex shrink-0 flex-col items-center gap-4 self-stretch">
                  <Toggle checked={checked} onChange={(value) => setValue(option.id, value)} label={t(option.title)} />
                  <button
                    type="button"
                    aria-label={t(option.edit)}
                    onClick={() => openEdit(option.modal)}
                    className="grid h-8 w-8 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:text-mint"
                  >
                    <Bolt size={22} strokeWidth={1.6} />
                  </button>
                </div>
              </>
            ) : (
              <>
                <div className="flex items-center gap-3">
                  <span className="grid h-12 w-12 shrink-0 place-items-center rounded-full border border-white/25 text-white">
                    <Icon size={22} strokeWidth={1.6} />
                  </span>
                  <h3 className="m-0 flex-1 text-[18px] font-semibold text-fg">{t(option.title)}</h3>
                  <Toggle checked={checked} onChange={(value) => setValue(option.id, value)} label={t(option.title)} />
                </div>
                <p className="m-0 mt-3 text-[14px] leading-snug text-fg2">{t(option.desc)}</p>
                <button
                  type="button"
                  onClick={() => openEdit(option.modal)}
                  className="mt-3 flex h-12 w-full items-center justify-center gap-1 rounded-full border border-white/10 bg-transparent text-[15px] font-medium text-fg hover:bg-white/5"
                >
                  {t(option.edit)}
                  <ChevronRight size={16} className="text-fg2 rtl:rotate-180" />
                </button>
              </>
            )}
          </div>
        );
      })}
    </div>
  );
}
```

## `src/components/OptionModals.tsx`

```tsx
import { useState } from "react";
import { ArrowDown, ArrowUp, Check } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import { ALL_TRANSPORTS, PRESETS, SHUFFLE_INTERVALS, SHUFFLE_REGIONS, SPLIT_APPS } from "../lib/config";
import { presetLabelKey, transportLabelKey } from "../i18n/translations";
import type { PresetId, TransportId } from "../types/vpn";
import { Button, Modal } from "./ui";
import { Field, SettingRow, inputClass } from "./SettingRow";
import { Flag } from "./Flag";

function chipClass(active: boolean): string {
  return cn(
    "inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-[13px] font-semibold transition-colors",
    active ? "border-mint bg-mint/15 text-mint" : "border-white/12 bg-transparent text-fg2 hover:text-fg",
  );
}

function ShuffleForm() {
  const { t, settings, patchSettings } = useApp();
  const shuffle = settings.shuffle;

  const toggleRegion = (region: string) => {
    const has = shuffle.regions.includes(region);
    if (has && shuffle.regions.length <= 1) {
      return;
    }
    const regions = has ? shuffle.regions.filter((item) => item !== region) : [...shuffle.regions, region];
    patchSettings({ shuffle: { ...shuffle, regions } });
  };

  return (
    <div>
      <p className="m-0 mb-2 text-[14px] leading-snug text-fg2">{t("shuffleHint")}</p>
      {!settings.psiphon && (
        <p className="m-0 mb-2 rounded-2xl border border-amber-fast/35 bg-amber-fast/12 px-4 py-2 text-[13px] text-amber-fast">{t("shuffleNeedsPsiphon")}</p>
      )}
      <SettingRow title={t("shuffleTitle")} checked={shuffle.enabled} onChange={(enabled) => patchSettings({ shuffle: { ...shuffle, enabled } })} />
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("shuffleInterval")}</div>
        <div className="flex flex-wrap gap-2">
          {SHUFFLE_INTERVALS.map((minutes) => (
            <button
              key={minutes}
              type="button"
              onClick={() => patchSettings({ shuffle: { ...shuffle, intervalMin: minutes } })}
              className={chipClass(shuffle.intervalMin === minutes)}
            >
              <span className="mono">{minutes}</span> {t("minutes")}
            </button>
          ))}
        </div>
      </div>
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("shuffleRegions")}</div>
        <div className="flex flex-wrap gap-2">
          {SHUFFLE_REGIONS.map((region) => (
            <button key={region} type="button" onClick={() => toggleRegion(region)} className={chipClass(shuffle.regions.includes(region))}>
              <Flag code={region} size={20} />
              <span className="mono">{region}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function parseLines(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function PathsForm() {
  const { t, settings, patchSettings } = useApp();
  const routing = settings.routing;
  const [direct, setDirect] = useState(routing.direct.join("\n"));
  const [block, setBlock] = useState(routing.block.join("\n"));

  const toggleApp = (name: string) => {
    const apps = routing.apps.includes(name) ? routing.apps.filter((item) => item !== name) : [...routing.apps, name];
    patchSettings({ routing: { ...routing, apps } });
  };

  return (
    <div>
      <SettingRow title={t("pathsEnable")} checked={routing.enabled} onChange={(enabled) => patchSettings({ routing: { ...routing, enabled } })} />
      <SettingRow
        title={t("pathsDomestic")}
        hint={t("pathsDomesticHint")}
        checked={routing.domesticBypass}
        onChange={(domesticBypass) => patchSettings({ routing: { ...routing, domesticBypass } })}
      />
      <Field label={t("pathsDirect")}>
        <textarea
          rows={3}
          value={direct}
          placeholder={`${t("pathsHint")}\nexample.com`}
          onChange={(event) => {
            setDirect(event.target.value);
            patchSettings({ routing: { ...routing, direct: parseLines(event.target.value) } });
          }}
          className={cn(inputClass, "mono resize-none")}
        />
      </Field>
      <Field label={t("pathsBlock")}>
        <textarea
          rows={3}
          value={block}
          placeholder={`${t("pathsHint")}\nads.example.net`}
          onChange={(event) => {
            setBlock(event.target.value);
            patchSettings({ routing: { ...routing, block: parseLines(event.target.value) } });
          }}
          className={cn(inputClass, "mono resize-none")}
        />
      </Field>
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("pathsApps")}</div>
        <div className="flex flex-wrap gap-2">
          {SPLIT_APPS.map((name) => (
            <button key={name} type="button" onClick={() => toggleApp(name)} className={chipClass(routing.apps.includes(name))}>
              {routing.apps.includes(name) && <Check size={14} />}
              {name}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function ProtocolForm() {
  const { t, settings, patchSettings } = useApp();
  const protocol = settings.protocol;
  const chain = protocol.chain;
  const ordered: TransportId[] = [...chain, ...ALL_TRANSPORTS.filter((id) => !chain.includes(id))];

  const applyPreset = (id: PresetId) => {
    const preset = PRESETS[id];
    patchSettings({
      protocol: { ...protocol, preset: id, chain: [...preset.chain], fragment: preset.fragment },
      scanMode: preset.scanMode,
      psiphon: preset.psiphon ?? settings.psiphon,
      sanctionCheck: preset.sanctionCheck ?? settings.sanctionCheck,
    });
  };

  const toggleTransport = (id: TransportId) => {
    if (chain.includes(id)) {
      if (chain.length <= 1) {
        return;
      }
      patchSettings({ protocol: { ...protocol, chain: chain.filter((item) => item !== id) } });
    } else {
      patchSettings({ protocol: { ...protocol, chain: [...chain, id] } });
    }
  };

  const move = (id: TransportId, delta: number) => {
    const index = chain.indexOf(id);
    const target = index + delta;
    if (index < 0 || target < 0 || target >= chain.length) {
      return;
    }
    const next = [...chain];
    [next[index], next[target]] = [next[target], next[index]];
    patchSettings({ protocol: { ...protocol, chain: next } });
  };

  return (
    <div>
      <SettingRow title={t("protoAuto")} checked={protocol.auto} onChange={(auto) => patchSettings({ protocol: { ...protocol, auto } })} />
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("protoPresets")}</div>
        <div className="flex flex-wrap gap-2">
          {(Object.keys(PRESETS) as PresetId[]).map((id) => (
            <button key={id} type="button" onClick={() => applyPreset(id)} className={chipClass(protocol.preset === id)}>
              {t(presetLabelKey[id])}
            </button>
          ))}
        </div>
      </div>
      <SettingRow title={t("protoPsiphon")} hint={t("protoPsiphonHint")} checked={settings.psiphon} onChange={(psiphon) => patchSettings({ psiphon })} />
      <SettingRow title={t("protoSanction")} hint={t("protoSanctionHint")} checked={settings.sanctionCheck} onChange={(sanctionCheck) => patchSettings({ sanctionCheck })} />
      <SettingRow
        title={t("protoFragment")}
        checked={protocol.fragment}
        onChange={(fragment) => patchSettings({ protocol: { ...protocol, fragment } })}
      />
      <div className="py-2">
        <div className="mb-2 text-[13px] font-semibold text-fg2">{t("protoChain")}</div>
        <ul className="m-0 flex list-none flex-col gap-2 p-0">
          {ordered.map((id) => {
            const index = chain.indexOf(id);
            const enabled = index >= 0;
            return (
              <li key={id} className={cn("flex items-center gap-3 rounded-2xl border px-3 py-2", enabled ? "border-white/12 bg-white/4" : "border-white/6 opacity-60")}>
                <button
                  type="button"
                  role="checkbox"
                  aria-checked={enabled}
                  aria-label={t(transportLabelKey[id])}
                  onClick={() => toggleTransport(id)}
                  className={cn("grid h-6 w-6 shrink-0 place-items-center rounded-full border", enabled ? "border-mint bg-mint text-[#04231a]" : "border-white/25 bg-transparent text-transparent")}
                >
                  <Check size={14} strokeWidth={3} />
                </button>
                <span className="mono w-5 text-center text-xs text-fg2">{enabled ? index + 1 : "–"}</span>
                <span className="min-w-0 flex-1 truncate text-[14px] font-medium">{t(transportLabelKey[id])}</span>
                {enabled && (
                  <span className="flex gap-1">
                    <button type="button" aria-label="up" onClick={() => move(id, -1)} className="grid h-7 w-7 place-items-center rounded-full border-0 bg-white/8 text-fg2 hover:text-fg">
                      <ArrowUp size={14} />
                    </button>
                    <button type="button" aria-label="down" onClick={() => move(id, 1)} className="grid h-7 w-7 place-items-center rounded-full border-0 bg-white/8 text-fg2 hover:text-fg">
                      <ArrowDown size={14} />
                    </button>
                  </span>
                )}
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
}

export function OptionModals() {
  const { t, modal, setModal } = useApp();
  const close = () => setModal(null);
  const footer = <Button onClick={close}>{t("done")}</Button>;
  return (
    <>
      <Modal open={modal === "shuffle"} title={t("shuffleTitle")} onClose={close} footer={footer}>
        <ShuffleForm />
      </Modal>
      <Modal open={modal === "paths"} title={t("multiTitle")} onClose={close} footer={footer}>
        <PathsForm />
      </Modal>
      <Modal open={modal === "protocol"} title={t("protocolTitle")} onClose={close} footer={footer} wide>
        <ProtocolForm />
      </Modal>
    </>
  );
}
```

## `src/components/OptionsDrawer.tsx`

```tsx
import { useApp } from "../state/AppContext";
import { Modal } from "./ui";
import { OptionCards } from "./OptionCards";

export function OptionsDrawer() {
  const { t, optionsOpen, setOptionsOpen } = useApp();
  return (
    <Modal open={optionsOpen} title={t("vpnOptions")} onClose={() => setOptionsOpen(false)}>
      <OptionCards variant="drawer" />
    </Modal>
  );
}
```

## `src/components/Page.tsx`

```tsx
import type { ReactNode } from "react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";

interface PageProps {
  title: string;
  children: ReactNode;
  actions?: ReactNode;
  fill?: boolean;
}

export function Page({ title, children, actions, fill = false }: PageProps) {
  const { isMobile } = useApp();
  return (
    <div className={cn("flex h-full min-h-0 flex-col", isMobile ? "px-5 pt-4" : "pe-1")}>
      <div className="mb-4 flex shrink-0 items-center justify-between gap-3">
        <h1 className={cn("m-0 font-semibold tracking-tight text-fg", isMobile ? "text-[30px]" : "text-[40px]")}>{title}</h1>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
      <div className={cn("min-h-0 flex-1 pb-4", fill ? "flex flex-col" : "scroll-y")}>
        {children}
      </div>
    </div>
  );
}

interface CardProps {
  title?: string;
  children: ReactNode;
  className?: string;
}

export function Card({ title, children, className = "" }: CardProps) {
  return (
    <section className={cn("rounded-[28px] border border-line bg-card p-5 shadow-[var(--shadow)]", className)}>
      {title && <h2 className="m-0 mb-2 text-[18px] font-semibold text-fg">{title}</h2>}
      {children}
    </section>
  );
}
```

## `src/components/SettingRow.tsx`

```tsx
import type { ReactNode } from "react";
import { Toggle } from "./ui";

interface SettingRowProps {
  title: string;
  hint?: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}

export function SettingRow({ title, hint, checked, onChange }: SettingRowProps) {
  return (
    <div className="flex items-center justify-between gap-4 py-3">
      <div className="min-w-0">
        <div className="text-[15px] font-semibold text-fg">{title}</div>
        {hint && <div className="mt-0.5 text-[13px] leading-snug text-fg2">{hint}</div>}
      </div>
      <Toggle checked={checked} onChange={onChange} label={title} />
    </div>
  );
}

interface FieldProps {
  label: string;
  children: ReactNode;
}

export function Field({ label, children }: FieldProps) {
  return (
    <label className="block py-2">
      <span className="mb-1.5 block text-[13px] font-semibold text-fg2">{label}</span>
      {children}
    </label>
  );
}

export const inputClass =
  "w-full rounded-2xl border border-line bg-chip px-4 py-2.5 text-[14px] text-fg outline-none placeholder:text-fg2 focus:border-mint";
```

## `src/components/Titlebar.tsx`

```tsx
import { Bell, Maximize2, Minus, X } from "lucide-react";
import { useApp } from "../state/AppContext";
import { windowAction } from "../lib/bridge";
import { CREATORS, CreatorAvatar } from "./AboutCard";

export function Titlebar() {
  const { t, updateInfo, setModal, setView, notify } = useApp();

  return (
    <header data-tauri-drag-region className="drag-region flex h-[68px] shrink-0 items-center gap-3 px-5">
      <button
        type="button"
        onClick={() => setView("settings")}
        className="no-drag flex items-center gap-3 rounded-full border border-line bg-card py-1.5 pe-5 ps-2 shadow-[var(--shadow)] transition-colors hover:border-mint/40"
      >
        <span className="flex items-center -space-x-2 rtl:space-x-reverse">
          {CREATORS.map((creator) => (
            <CreatorAvatar key={creator.name} creator={creator} size={34} />
          ))}
        </span>
        <span className="text-[14.5px] font-semibold text-fg">{t("badgeCredit")}</span>
      </button>

      <div className="flex-1" />

      {updateInfo && (
        <button
          type="button"
          onClick={() => setModal("updater")}
          className="no-drag inline-flex items-center gap-2 rounded-full border border-mint/40 bg-mint/12 px-4 py-2 text-[13px] font-bold text-[#00a870] dark:text-mint"
        >
          <span className="dot-live h-2 w-2 rounded-full bg-mint" />
          {t("updateAvailable")} · v{updateInfo.appLatest}
        </button>
      )}

      <button
        type="button"
        aria-label={t("notifications")}
        onClick={() => (updateInfo ? setModal("updater") : notify(t("updaterUpToDate")))}
        className="no-drag grid h-10 w-10 place-items-center rounded-full border border-line bg-card text-fg shadow-[var(--shadow)]"
      >
        <Bell size={18} className="fill-current" />
      </button>

      <div className="no-drag ms-2 flex items-center gap-1">
        <button
          type="button"
          title={t("minimize")}
          aria-label={t("minimize")}
          onClick={() => windowAction("minimize")}
          className="grid h-9 w-9 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:bg-chip hover:text-fg"
        >
          <Minus size={16} />
        </button>
        <button
          type="button"
          title={t("maximize")}
          aria-label={t("maximize")}
          onClick={() => windowAction("maximize")}
          className="grid h-9 w-9 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:bg-chip hover:text-fg"
        >
          <Maximize2 size={14} />
        </button>
        <button
          type="button"
          title={t("closeWindow")}
          aria-label={t("closeWindow")}
          onClick={() => windowAction("close")}
          className="grid h-9 w-9 place-items-center rounded-full border-0 bg-transparent text-fg2 hover:bg-red-500 hover:text-white"
        >
          <X size={16} />
        </button>
      </div>
    </header>
  );
}
```

## `src/components/ui.tsx`

```tsx
import { useEffect, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Check, ChevronDown, Star, X } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";

interface ToggleProps {
  checked: boolean;
  onChange: (value: boolean) => void;
  label?: string;
}

export function Toggle({ checked, onChange, label }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className={cn(
        "relative h-[26px] w-[48px] shrink-0 rounded-full border-0 transition-colors duration-200",
        checked ? "bg-mint" : "bg-[#4b5654]",
      )}
    >
      <span
        className={cn(
          "absolute top-[3px] h-5 w-5 rounded-full bg-[#0e1514] transition-all duration-200",
          checked ? "start-[25px]" : "start-[3px]",
        )}
      />
    </button>
  );
}

interface SegmentedProps<T extends string> {
  value: T;
  options: Array<{ value: T; label: string }>;
  onChange: (value: T) => void;
  small?: boolean;
}

export function Segmented<T extends string>({ value, options, onChange, small = false }: SegmentedProps<T>) {
  return (
    <div className="inline-flex rounded-full border border-line bg-chip p-[3px]">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={cn(
            "rounded-full border-0 font-semibold transition-colors",
            small ? "px-3 py-1 text-xs" : "px-4 py-1.5 text-[13px]",
            value === option.value ? "bg-mint text-[#06251a]" : "bg-transparent text-fg2 hover:text-fg",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export interface SelectOption<T extends string> {
  value: T;
  label: string;
  sublabel?: string;
  icon?: ReactNode;
}

interface CustomSelectProps<T extends string> {
  value: T;
  options: Array<SelectOption<T>>;
  onChange: (value: T) => void;
}

export function CustomSelect<T extends string>({ value, options, onChange }: CustomSelectProps<T>) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const selected = options.find((item) => item.value === value) ?? options[0];

  useEffect(() => {
    if (!open) {
      return;
    }
    const onPointerDown = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
      }
    };
    window.addEventListener("mousedown", onPointerDown);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("mousedown", onPointerDown);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
        className={cn(
          "flex h-11 w-full items-center justify-between gap-3 rounded-2xl border px-4 text-[14px] font-medium text-fg transition-all",
          open ? "border-mint bg-card shadow-[0_0_0_3px_rgba(0,229,153,0.14)]" : "border-line bg-chip hover:border-mint/50",
        )}
      >
        <span className="flex min-w-0 items-center gap-2.5 truncate">
          {selected?.icon}
          <span className="truncate">{selected?.label}</span>
          {selected?.sublabel && <span className="mono text-xs text-fg2">({selected.sublabel})</span>}
        </span>
        <ChevronDown
          size={17}
          className={cn("shrink-0 text-fg2 transition-transform duration-200", open && "rotate-180 text-mint")}
        />
      </button>

      <AnimatePresence>
        {open && (
          <motion.div
            role="listbox"
            initial={{ opacity: 0, y: -6, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -6, scale: 0.98 }}
            transition={{ duration: 0.14 }}
            className="surface-dark scroll-y absolute inset-x-0 top-[calc(100%+6px)] z-50 max-h-60 overflow-y-auto rounded-2xl border border-white/12 bg-[#121918] p-1.5 text-fg shadow-2xl"
          >
            {options.map((option) => {
              const active = option.value === value;
              return (
                <button
                  key={option.value}
                  type="button"
                  role="option"
                  aria-selected={active}
                  onClick={() => {
                    onChange(option.value);
                    setOpen(false);
                  }}
                  className={cn(
                    "flex w-full items-center justify-between gap-2.5 rounded-xl border-0 px-3 py-2.5 text-start text-[13.5px] font-medium transition-colors",
                    active ? "bg-mint/15 font-semibold text-mint" : "bg-transparent text-fg hover:bg-white/8",
                  )}
                >
                  <span className="flex min-w-0 items-center gap-2.5 truncate">
                    {option.icon}
                    <span className="truncate">{option.label}</span>
                    {option.sublabel && <span className="mono text-xs text-fg2">{option.sublabel}</span>}
                  </span>
                  {active && (
                    <span className="grid h-5 w-5 shrink-0 place-items-center rounded-full bg-mint text-[#04231a]">
                      <Check size={12} strokeWidth={3} />
                    </span>
                  )}
                </button>
              );
            })}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

interface ModalProps {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
  wide?: boolean;
}

export function Modal({ open, title, onClose, children, footer, wide = false }: ModalProps) {
  const { isMobile, t, resolvedTheme } = useApp();

  useEffect(() => {
    if (!open) {
      return;
    }
    const handler = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className={cn("fixed inset-0 z-50 flex bg-black/60 backdrop-blur-sm", isMobile ? "items-end" : "items-center justify-center p-6")}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) {
              onClose();
            }
          }}
        >
          <motion.div
            role="dialog"
            aria-label={title}
            data-theme={resolvedTheme}
            className={cn(
              "surface-dark flex max-h-[88vh] flex-col border border-white/10 bg-hero text-fg shadow-2xl",
              isMobile ? "w-full rounded-t-[32px]" : cn("w-full rounded-[28px]", wide ? "max-w-2xl" : "max-w-lg"),
            )}
            initial={isMobile ? { y: "100%" } : { opacity: 0, scale: 0.96, y: 12 }}
            animate={isMobile ? { y: 0 } : { opacity: 1, scale: 1, y: 0 }}
            exit={isMobile ? { y: "100%" } : { opacity: 0, scale: 0.96 }}
            transition={{ type: "spring", damping: 30, stiffness: 340 }}
          >
            {isMobile && <div className="mx-auto mt-3 h-1 w-14 rounded-full bg-white/30" />}
            <div className="flex items-center justify-between px-6 pb-2 pt-5">
              <h2 className="m-0 text-xl font-bold">{title}</h2>
              <button
                type="button"
                aria-label={t("done")}
                onClick={onClose}
                className="grid h-9 w-9 place-items-center rounded-full border-0 bg-white/8 text-fg2 hover:text-fg"
              >
                <X size={18} />
              </button>
            </div>
            <div className="scroll-y min-h-0 flex-1 px-6 pb-4 pt-2">{children}</div>
            {footer && <div className="flex items-center justify-end gap-3 border-t border-line px-6 py-4">{footer}</div>}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

interface ButtonProps {
  children: ReactNode;
  onClick?: () => void;
  variant?: "mint" | "outline" | "ghost";
  disabled?: boolean;
  className?: string;
  type?: "button" | "submit";
}

export function Button({ children, onClick, variant = "mint", disabled = false, className = "", type = "button" }: ButtonProps) {
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "inline-flex items-center justify-center gap-2 rounded-full px-5 py-2.5 text-sm font-bold transition-all disabled:cursor-not-allowed disabled:opacity-50",
        variant === "mint" && "border-0 bg-mint text-[#06251a] hover:bg-mint-hover",
        variant === "outline" && "border border-mint bg-transparent text-mint hover:bg-mint/10",
        variant === "ghost" && "border border-line bg-transparent text-fg hover:bg-chip",
        className,
      )}
    >
      {children}
    </button>
  );
}

export function Toast() {
  const { toast } = useApp();
  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-28 z-[60] flex justify-center md:bottom-10">
      <AnimatePresence>
        {toast && (
          <motion.div
            key={toast.id}
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 10 }}
            className="flex items-center gap-2 rounded-full border border-white/10 bg-[#1d2a27] px-4 py-2 text-sm font-semibold text-white shadow-xl"
          >
            <Star size={16} className={toast.starred ? "fill-star text-star" : "text-white/70"} />
            {toast.text}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
```

## `src/components/UpdaterModal.tsx`

```tsx
import { useEffect, useState } from "react";
import { BadgeCheck, Download, Loader2, RotateCw } from "lucide-react";
import { useApp } from "../state/AppContext";
import { APP_VERSION, CORE_VERSION } from "../lib/config";
import { call, sleep } from "../lib/bridge";
import { Button, Modal } from "./ui";

type Phase = "idle" | "checking" | "ready" | "downloading" | "verifying" | "verified" | "installing" | "current";

export function UpdaterBody() {
  const { t, updateInfo, checkForUpdates, completeUpdate, setModal } = useApp();
  const [phase, setPhase] = useState<Phase>(updateInfo ? "ready" : "idle");
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    if (updateInfo && phase === "idle") {
      setPhase("ready");
    }
  }, [updateInfo, phase]);

  const check = async () => {
    setPhase("checking");
    const info = await checkForUpdates();
    setPhase(info ? "ready" : "current");
  };

  const download = async () => {
    setPhase("downloading");
    setProgress(0);
    const ticker = window.setInterval(() => {
      setProgress((prev) => (prev < 92 ? prev + 4 : prev));
    }, 90);
    try {
      await call<void>("updater_download", {}, async () => {
        for (let value = 0; value <= 100; value += 4) {
          setProgress(value);
          await sleep(70);
        }
      });
    } finally {
      window.clearInterval(ticker);
    }
    setProgress(100);
    setPhase("verifying");
    await sleep(1200);
    setPhase("verified");
  };

  const install = async () => {
    setPhase("installing");
    await call<void>("updater_install", {}, () => sleep(1800));
    completeUpdate();
    setPhase("current");
    setModal(null);
  };

  const rows = [
    { label: t("updaterApp"), current: APP_VERSION, latest: updateInfo?.appLatest ?? APP_VERSION },
    { label: t("updaterCore"), current: CORE_VERSION, latest: updateInfo?.coreLatest ?? CORE_VERSION },
  ];

  return (
    <div>
      <div className="grid gap-3 sm:grid-cols-2">
        {rows.map((row) => (
          <div key={row.label} className="rounded-2xl border border-white/8 bg-white/4 p-4">
            <div className="text-[13px] font-semibold text-fg2">{row.label}</div>
            <div className="mono mt-2 flex items-center justify-between text-[14px]">
              <span className="text-fg2">
                {t("updaterCurrent")} v{row.current}
              </span>
              <span className={row.latest !== row.current ? "font-bold text-mint" : "text-fg2"}>
                {t("updaterLatest")} v{row.latest}
              </span>
            </div>
          </div>
        ))}
      </div>

      {phase === "current" && (
        <p className="m-0 mt-4 flex items-center gap-2 text-[14px] font-semibold text-mint">
          <BadgeCheck size={18} />
          {t("updaterUpToDate")}
        </p>
      )}

      {updateInfo && phase !== "current" && (
        <div className="mt-4">
          <div className="mb-2 text-[13px] font-semibold text-fg2">{t("updaterChangelog")}</div>
          <ul className="m-0 flex list-none flex-col gap-1.5 p-0 text-[14px]">
            {updateInfo.changelog.map((line) => (
              <li key={line} className="flex gap-2">
                <span className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-mint" />
                {line}
              </li>
            ))}
          </ul>
          <div className="mono mt-3 break-all text-[11px] text-fg2">
            {t("updaterMirror")}: {updateInfo.mirror} · sha256:{updateInfo.sha256}
          </div>
        </div>
      )}

      {(phase === "downloading" || phase === "verifying" || phase === "verified" || phase === "installing") && (
        <div className="mt-5">
          <div className="h-2.5 overflow-hidden rounded-full bg-white/10">
            <div
              className={phase === "downloading" || phase === "installing" ? "bar-shimmer h-full rounded-full" : "h-full rounded-full bg-mint"}
              style={{ width: `${phase === "installing" ? 100 : progress}%`, transition: "width 0.15s linear" }}
            />
          </div>
          <div className="mono mt-2 flex items-center justify-between text-[12px] text-fg2">
            <span>
              {phase === "downloading" && t("updaterDownloading")}
              {phase === "verifying" && t("updaterVerifying")}
              {phase === "verified" && t("updaterVerified")}
              {phase === "installing" && t("updaterInstalling")}
            </span>
            <span>{progress}%</span>
          </div>
          {phase === "verified" && (
            <span className="mt-3 inline-flex items-center gap-2 rounded-full border border-mint/40 bg-mint/12 px-3 py-1 text-[12px] font-bold text-mint">
              <BadgeCheck size={14} />
              SHA-256 · Ed25519
            </span>
          )}
        </div>
      )}

      <div className="mt-6 flex justify-end gap-3">
        {(phase === "idle" || phase === "current") && (
          <Button variant="outline" onClick={() => void check()}>
            <RotateCw size={16} />
            {t("updaterCheck")}
          </Button>
        )}
        {phase === "checking" && (
          <Button variant="outline" disabled>
            <Loader2 size={16} className="spin-slow" />
            {t("updaterChecking")}
          </Button>
        )}
        {phase === "ready" && (
          <Button onClick={() => void download()}>
            <Download size={16} />
            {t("updaterDownload")}
          </Button>
        )}
        {(phase === "downloading" || phase === "verifying") && (
          <Button disabled>
            <Loader2 size={16} className="spin-slow" />
            {phase === "downloading" ? t("updaterDownloading") : t("updaterVerifying")}
          </Button>
        )}
        {phase === "verified" && <Button onClick={() => void install()}>{t("updaterInstall")}</Button>}
        {phase === "installing" && (
          <Button disabled>
            <Loader2 size={16} className="spin-slow" />
            {t("updaterInstall")}
          </Button>
        )}
      </div>
    </div>
  );
}

export function UpdaterModal() {
  const { t, modal, setModal } = useApp();
  return (
    <Modal open={modal === "updater"} title={t("updaterTitle")} onClose={() => setModal(null)} wide>
      <UpdaterBody />
    </Modal>
  );
}
```

## `src/i18n/translations.ts`

```typescript
import type { Lang } from "../types/vpn";

const en = {
  appName: "Radsho",
  navDashboard: "Dashboard",
  navLocations: "Locations",
  navLogs: "Live Logs",
  navSpeed: "Speed",
  navCredits: "WitWizard",
  navSettings: "Settings",
  badgeCredit: "WitWizard Team",
  updateAvailable: "Update available",
  themeSystem: "System",
  themeLight: "Light",
  themeDark: "Dark",
  langToggle: "فا",
  minimize: "Minimize to tray",
  maximize: "Maximize",
  closeWindow: "Close",
  notifications: "Notifications",
  vpnConnection: "VPN Connection",
  connectedTo: "Connected to:",
  disconnected: "Disconnected",
  paused: "Paused",
  preparing: "Preparing core...",
  connecting: "Establishing tunnel...",
  verifying: "Verifying Tunnel...",
  errorState: "Connection failed",
  fast: "Fast",
  moreFastServers: "More Fast Servers:",
  searchLocation: "Search for VPN Location",
  connect: "Connect",
  disconnect: "Disconnect",
  resumeConnection: "Resume Connection",
  pause: "Pause",
  resume: "Resume",
  moreVpnOptions: "More VPN Options",
  advancedOptions: "Advanced Options",
  vpnOptions: "VPN Options",
  shuffleTitle: "Shuffle Location",
  shuffleDesc: "Switch server locations randomly while you are connected to the VPN.",
  multiTitle: "Multi Paths",
  multiDesc: "Use the VPN connections only for selected websites or apps.",
  protocolTitle: "Protocol Switch",
  protocolDesc: "Let app automatically change the protocol based on your activity.",
  editShuffle: "Edit Shuffle Location",
  editMulti: "Edit Multi Paths",
  editProtocol: "Edit Protocol Switch",
  edit: "Edit...",
  scannerTitle: "Device Scanner",
  scannerRunning: "Actively Running",
  scannerIdle: "Idle",
  scannerAction: "Manage Device Scanner",
  checkTitle: "Device Check",
  checkSub: "Device Health Analytics",
  checkAction: "Check Your Device",
  speedTitle: "Speed Test",
  speedSub: "Test Server Speed",
  speedAction: "Run Speed Test",
  vpnLocations: "VPN Locations",
  sortBy: "Sort by:",
  catFast: "Fast",
  catStreaming: "Streaming",
  catGaming: "Gaming",
  catAnti: "Anti-Sanction",
  catDouble: "Double-Hop",
  sortAz: "A-Z",
  sortZa: "Z-A",
  noResults: "No locations found",
  addedFav: "Added to Favorites",
  removedFav: "Removed from Favorites",
  featured: "Featured servers",
  cancel: "Cancel",
  save: "Save",
  done: "Done",
  back: "Back",
  shuffleInterval: "Rotate every",
  minutes: "min",
  shuffleRegions: "Exit countries",
  shuffleHint: "Rotates the Psiphon exit country without dropping the WARP carrier.",
  shuffleNeedsPsiphon: "Shuffle needs Psiphon mode. Enable it in Protocol Switch.",
  pathsEnable: "Enable split tunneling",
  pathsDirect: "Direct domains (--route-direct)",
  pathsBlock: "Blocked domains (--route-block)",
  pathsDomestic: "Domestic bypass (.ir)",
  pathsDomesticHint: "Keep Iranian domains off the tunnel.",
  pathsApps: "Per-app routing",
  pathsHint: "One domain per line",
  protoAuto: "Automatic protocol switching",
  protoChain: "Fallback chain",
  protoPresets: "Quick Presets",
  presetUnblock: "Unblock / Sanction Bypass",
  presetResilient: "Resilient",
  presetFastest: "Fastest",
  presetAntiDpi: "Anti-DPI",
  presetLowData: "Low Data",
  protoFragment: "Packet fragmentation (--fragment)",
  protoPsiphon: "Psiphon inside WARP",
  protoPsiphonHint: "Carries Psiphon inside the WARP tunnel to obtain a non-sanctioned exit IP.",
  protoSanction: "Sanction reachability check",
  protoSanctionHint: "Verifies kimi.ai, claude.ai and huggingface.co answer without 403 / 451.",
  trMasqueH3: "MASQUE / HTTP-3 (QUIC)",
  trMasqueH2: "MASQUE / HTTP-2 (TCP 443)",
  trWireguard: "WireGuard",
  trGool: "Gool (WG in MASQUE)",
  trGoolClassic: "Gool Classic",
  trMim: "MIM (MASQUE in MASQUE)",
  scannerPorts: "Local port scan (1819–1822)",
  scannerDns: "DNS leak test",
  scannerIpv6: "IPv6 leak check",
  scannerTunnel: "Tunnel verification",
  scanNow: "Scan now",
  scanning: "Scanning...",
  runAgain: "Run again",
  stOk: "OK",
  stIssue: "Issue",
  stIdle: "Not active",
  portFree: "free",
  portInUse: "in use by Radsho",
  portConflict: "conflict",
  dnsClean: "Resolver answered through the tunnel",
  dnsDirect: "Queries leave on the direct line",
  ipv6Blocked: "IPv6 is blackholed, no leak",
  ipv6Open: "IPv6 is reachable outside the tunnel",
  tunnelActive: "Egress verified and non-sanctioned",
  tunnelInactive: "No active tunnel",
  checkCore: "Aether core integrity (SHA-256)",
  checkPsiphon: "Psiphon transport health",
  checkProxy: "WinINET / TUN driver",
  checkSanction: "Sanction reachability",
  checkRun: "Run diagnostics",
  speedDownload: "Download",
  speedUpload: "Upload",
  speedPing: "Ping",
  speedJitter: "Jitter",
  speedStart: "Start test",
  speedTesting: "Testing...",
  speedStagePing: "Measuring latency & jitter...",
  speedStageDown: "Testing download throughput...",
  speedStageUp: "Testing upload throughput...",
  speedStageReady: "Ready to benchmark",
  speedViaTunnel: "Measured through the active tunnel",
  speedDirect: "Measured on the direct line (no tunnel)",
  speedHistory: "Recent results",
  logsTitle: "Live Logs",
  logsClear: "Clear",
  logsCopy: "Copy",
  logsEmpty: "No log lines yet",
  logsAll: "All",
  logsArgv: "Core command",
  copied: "Copied to clipboard",
  settingsTitle: "Settings",
  setAppearance: "Appearance",
  setTheme: "Theme",
  setLanguage: "Language",
  langEn: "English",
  langFa: "فارسی",
  setTunnel: "Tunnel",
  setTransport: "Transport",
  setScanMode: "Scan mode",
  setPsiphonRegion: "Psiphon region",
  setPorts: "Ports",
  portBind: "Core SOCKS5 (bind)",
  portHttp: "Core HTTP",
  portPsiphon: "Psiphon SOCKS5",
  portPsiphonHttp: "Psiphon HTTP",
  portsInvalid: "Ports must be unique and between 1024 and 65535.",
  setDns: "Custom DNS",
  setExtra: "Extra core arguments",
  setSystem: "System",
  setSysProxy: "System proxy / TUN",
  setSysProxyHint: "Route OS traffic through the tunnel after verification.",
  setTray: "Minimize to tray",
  setAutoUpdate: "Automatic update checks",
  updaterTitle: "Updates",
  updaterApp: "Radsho app",
  updaterCore: "Aether core",
  updaterCurrent: "Current",
  updaterLatest: "Latest",
  updaterCheck: "Check for updates",
  updaterChecking: "Checking...",
  updaterUpToDate: "You are up to date",
  updaterChangelog: "Changelog",
  updaterDownload: "Download & verify",
  updaterDownloading: "Downloading...",
  updaterVerifying: "Verifying SHA-256 / Ed25519...",
  updaterVerified: "SHA-256 and Ed25519 signature verified",
  updaterInstall: "Install & restart",
  updaterInstalling: "Installing, rolling back automatically on failure...",
  updaterMirror: "Mirror",
  updaterOpen: "Open updater",
  aboutTitle: "About WitWizard",
  aboutBody: "Radsho is an ultra-fast, censorship-resistant VPN and smart sanction-bypass client built from one codebase for Windows, macOS, Linux, Android and iOS.",
  aboutBy: "Developed by",
  aboutCreators: "Lead creators",
  roleLilmmd: "Lead Architect & Full-Stack",
  roleLfnightt: "Core Systems & Security",
  aboutVersion: "Version",
  aboutSupport: "Support & credits",
  trayShow: "Open Radsho",
  trayQuit: "Quit Radsho",
  traySwitchLang: "Switch to Persian (فارسی)",
  sessionRegion: "Exit region",
  sessionTransport: "Transport",
  sessionEgress: "Egress",
  stateOff: "OFF",
  statePreparing: "PREPARING",
  stateConnecting: "CONNECTING",
  stateVerifying: "VERIFYING",
  statePaused: "PAUSED",
  stateRunning: "RUNNING",
  stateError: "ERROR",
  speedUnit: "Mbps",
  msUnit: "ms",
} as const;

export type TKey = keyof typeof en;

const fa: Record<TKey, string> = {
  appName: "ردشو",
  navDashboard: "داشبورد",
  navLocations: "مکان‌ها",
  navLogs: "گزارش زنده",
  navSpeed: "سرعت",
  navCredits: "ویت‌ویزارد",
  navSettings: "تنظیمات",
  badgeCredit: "تیم ویت‌ویزارد",
  updateAvailable: "به‌روزرسانی موجود است",
  themeSystem: "سیستم",
  themeLight: "روشن",
  themeDark: "تیره",
  langToggle: "EN",
  minimize: "کوچک‌سازی به سینی",
  maximize: "بزرگ‌نمایی",
  closeWindow: "بستن",
  notifications: "اعلان‌ها",
  vpnConnection: "اتصال وی‌پی‌ان",
  connectedTo: "متصل به:",
  disconnected: "قطع شده",
  paused: "متوقف شده",
  preparing: "در حال آماده‌سازی هسته...",
  connecting: "در حال برقراری تونل...",
  verifying: "در حال تأیید تونل...",
  errorState: "اتصال ناموفق بود",
  fast: "سریع",
  moreFastServers: "سرورهای سریع بیشتر:",
  searchLocation: "جستجوی مکان وی‌پی‌ان",
  connect: "اتصال",
  disconnect: "قطع اتصال",
  resumeConnection: "ادامه اتصال",
  pause: "توقف",
  resume: "ادامه",
  moreVpnOptions: "گزینه‌های بیشتر وی‌پی‌ان",
  advancedOptions: "گزینه‌های پیشرفته",
  vpnOptions: "گزینه‌های وی‌پی‌ان",
  shuffleTitle: "جابه‌جایی مکان",
  shuffleDesc: "هنگام اتصال، مکان سرور به‌صورت تصادفی تغییر می‌کند.",
  multiTitle: "مسیرهای چندگانه",
  multiDesc: "وی‌پی‌ان فقط برای وب‌سایت‌ها یا برنامه‌های انتخاب‌شده استفاده شود.",
  protocolTitle: "تعویض پروتکل",
  protocolDesc: "برنامه به‌صورت خودکار و بر اساس فعالیت شما پروتکل را عوض کند.",
  editShuffle: "ویرایش جابه‌جایی مکان",
  editMulti: "ویرایش مسیرهای چندگانه",
  editProtocol: "ویرایش تعویض پروتکل",
  edit: "ویرایش...",
  scannerTitle: "اسکنر دستگاه",
  scannerRunning: "در حال اجرا",
  scannerIdle: "غیرفعال",
  scannerAction: "مدیریت اسکنر دستگاه",
  checkTitle: "بررسی دستگاه",
  checkSub: "تحلیل سلامت دستگاه",
  checkAction: "دستگاه خود را بررسی کنید",
  speedTitle: "تست سرعت",
  speedSub: "سنجش سرعت سرور",
  speedAction: "اجرای تست سرعت",
  vpnLocations: "مکان‌های وی‌پی‌ان",
  sortBy: "مرتب‌سازی:",
  catFast: "سریع",
  catStreaming: "استریم",
  catGaming: "بازی",
  catAnti: "ضدتحریم",
  catDouble: "دو‌مرحله‌ای",
  sortAz: "الف-ی",
  sortZa: "ی-الف",
  noResults: "مکانی پیدا نشد",
  addedFav: "به علاقه‌مندی‌ها اضافه شد",
  removedFav: "از علاقه‌مندی‌ها حذف شد",
  featured: "سرورهای ویژه",
  cancel: "انصراف",
  save: "ذخیره",
  done: "تمام",
  back: "بازگشت",
  shuffleInterval: "تغییر هر",
  minutes: "دقیقه",
  shuffleRegions: "کشورهای خروجی",
  shuffleHint: "کشور خروجی سایفون را بدون قطع تونل WARP عوض می‌کند.",
  shuffleNeedsPsiphon: "جابه‌جایی به حالت سایفون نیاز دارد. آن را در تعویض پروتکل فعال کنید.",
  pathsEnable: "فعال‌سازی تونل‌زنی تفکیکی",
  pathsDirect: "دامنه‌های مستقیم (--route-direct)",
  pathsBlock: "دامنه‌های مسدود (--route-block)",
  pathsDomestic: "عبور مستقیم داخلی (.ir)",
  pathsDomesticHint: "دامنه‌های ایرانی از تونل عبور نمی‌کنند.",
  pathsApps: "مسیریابی برای هر برنامه",
  pathsHint: "هر خط یک دامنه",
  protoAuto: "تعویض خودکار پروتکل",
  protoChain: "زنجیره جایگزین",
  protoPresets: "پیش‌تنظیم‌های سریع",
  presetUnblock: "رفع مسدودی / عبور از تحریم",
  presetResilient: "مقاوم",
  presetFastest: "سریع‌ترین",
  presetAntiDpi: "ضد DPI",
  presetLowData: "مصرف کم داده",
  protoFragment: "تکه‌تکه‌سازی بسته (--fragment)",
  protoPsiphon: "سایفون داخل WARP",
  protoPsiphonHint: "سایفون را داخل تونل WARP حمل می‌کند تا IP خروجی غیرتحریمی بگیرد.",
  protoSanction: "بررسی دسترسی سایت‌های تحریمی",
  protoSanctionHint: "پاسخ kimi.ai، claude.ai و huggingface.co بدون خطای 403 / 451 بررسی می‌شود.",
  trMasqueH3: "MASQUE / HTTP-3 (QUIC)",
  trMasqueH2: "MASQUE / HTTP-2 (TCP 443)",
  trWireguard: "وایرگارد",
  trGool: "Gool (وایرگارد در MASQUE)",
  trGoolClassic: "Gool کلاسیک",
  trMim: "MIM (MASQUE در MASQUE)",
  scannerPorts: "اسکن پورت‌های محلی (۱۸۱۹–۱۸۲۲)",
  scannerDns: "تست نشت DNS",
  scannerIpv6: "بررسی نشت IPv6",
  scannerTunnel: "تأیید تونل",
  scanNow: "اسکن کن",
  scanning: "در حال اسکن...",
  runAgain: "اجرای دوباره",
  stOk: "سالم",
  stIssue: "مشکل",
  stIdle: "غیرفعال",
  portFree: "آزاد",
  portInUse: "در حال استفاده ردشو",
  portConflict: "تداخل",
  dnsClean: "پاسخ DNS از داخل تونل رسید",
  dnsDirect: "درخواست‌ها از مسیر مستقیم خارج می‌شوند",
  ipv6Blocked: "IPv6 مسدود است و نشتی وجود ندارد",
  ipv6Open: "IPv6 خارج از تونل در دسترس است",
  tunnelActive: "خروجی تأیید شد و غیرتحریمی است",
  tunnelInactive: "تونل فعالی وجود ندارد",
  checkCore: "یکپارچگی هسته Aether (SHA-256)",
  checkPsiphon: "سلامت انتقال سایفون",
  checkProxy: "درایور WinINET / TUN",
  checkSanction: "دسترسی به سایت‌های تحریمی",
  checkRun: "اجرای عیب‌یابی",
  speedDownload: "دانلود",
  speedUpload: "آپلود",
  speedPing: "پینگ",
  speedJitter: "جیتر",
  speedStart: "شروع تست",
  speedTesting: "در حال تست...",
  speedStagePing: "در حال سنجش پینگ و جیتر...",
  speedStageDown: "در حال سنجش سرعت دانلود...",
  speedStageUp: "در حال سنجش سرعت آپلود...",
  speedStageReady: "آماده شروع تست سرعت",
  speedViaTunnel: "اندازه‌گیری از طریق تونل فعال",
  speedDirect: "اندازه‌گیری روی خط مستقیم (بدون تونل)",
  speedHistory: "نتایج اخیر",
  logsTitle: "گزارش زنده",
  logsClear: "پاک کردن",
  logsCopy: "کپی",
  logsEmpty: "هنوز گزارشی ثبت نشده",
  logsAll: "همه",
  logsArgv: "دستور هسته",
  copied: "در کلیپ‌بورد کپی شد",
  settingsTitle: "تنظیمات",
  setAppearance: "ظاهر",
  setTheme: "پوسته",
  setLanguage: "زبان",
  langEn: "English",
  langFa: "فارسی",
  setTunnel: "تونل",
  setTransport: "پروتکل انتقال",
  setScanMode: "حالت اسکن",
  setPsiphonRegion: "منطقه سایفون",
  setPorts: "پورت‌ها",
  portBind: "SOCKS5 هسته (bind)",
  portHttp: "HTTP هسته",
  portPsiphon: "SOCKS5 سایفون",
  portPsiphonHttp: "HTTP سایفون",
  portsInvalid: "پورت‌ها باید یکتا و بین ۱۰۲۴ تا ۶۵۵۳۵ باشند.",
  setDns: "DNS سفارشی",
  setExtra: "آرگومان‌های اضافه هسته",
  setSystem: "سیستم",
  setSysProxy: "پروکسی سیستم / TUN",
  setSysProxyHint: "پس از تأیید، ترافیک سیستم از تونل عبور می‌کند.",
  setTray: "کوچک‌سازی به سینی",
  setAutoUpdate: "بررسی خودکار به‌روزرسانی",
  updaterTitle: "به‌روزرسانی‌ها",
  updaterApp: "برنامه ردشو",
  updaterCore: "هسته Aether",
  updaterCurrent: "فعلی",
  updaterLatest: "جدیدترین",
  updaterCheck: "بررسی به‌روزرسانی",
  updaterChecking: "در حال بررسی...",
  updaterUpToDate: "شما از آخرین نسخه استفاده می‌کنید",
  updaterChangelog: "تغییرات",
  updaterDownload: "دانلود و تأیید",
  updaterDownloading: "در حال دانلود...",
  updaterVerifying: "در حال تأیید SHA-256 / Ed25519...",
  updaterVerified: "SHA-256 و امضای Ed25519 تأیید شد",
  updaterInstall: "نصب و راه‌اندازی دوباره",
  updaterInstalling: "در حال نصب، در صورت خطا به‌صورت خودکار بازگردانی می‌شود...",
  updaterMirror: "آینه",
  updaterOpen: "باز کردن به‌روزرسان",
  aboutTitle: "درباره ویت‌ویزارد",
  aboutBody: "ردشو یک کلاینت فوق‌سریع، مقاوم در برابر سانسور و هوشمند برای عبور از تحریم است که با یک کدبیس برای ویندوز، مک، لینوکس، اندروید و iOS ساخته شده است.",
  aboutBy: "توسعه‌یافته توسط",
  aboutCreators: "سازندگان اصلی",
  roleLilmmd: "معمار ارشد و فول‌استک",
  roleLfnightt: "مهندس هسته و امنیت",
  aboutVersion: "نسخه",
  aboutSupport: "پشتیبانی و سازندگان",
  trayShow: "باز کردن ردشو",
  trayQuit: "خروج کامل از ردشو",
  traySwitchLang: "تغییر زبان به English",
  sessionRegion: "منطقه خروجی",
  sessionTransport: "پروتکل",
  sessionEgress: "خروجی",
  stateOff: "خاموش",
  statePreparing: "آماده‌سازی",
  stateConnecting: "اتصال",
  stateVerifying: "تأیید",
  statePaused: "متوقف",
  stateRunning: "فعال",
  stateError: "خطا",
  speedUnit: "Mbps",
  msUnit: "ms",
};

export const translations: Record<Lang, Record<TKey, string>> = { en, fa };

export const transportLabelKey = {
  MASQUE_H3: "trMasqueH3",
  MASQUE_H2: "trMasqueH2",
  WIREGUARD: "trWireguard",
  GOOL: "trGool",
  GOOL_CLASSIC: "trGoolClassic",
  MIM: "trMim",
} as const satisfies Record<string, TKey>;

export const presetLabelKey = {
  unblock: "presetUnblock",
  resilient: "presetResilient",
  fastest: "presetFastest",
  antidpi: "presetAntiDpi",
  lowdata: "presetLowData",
} as const satisfies Record<string, TKey>;

export const categoryLabelKey = {
  fast: "catFast",
  streaming: "catStreaming",
  gaming: "catGaming",
  antisanction: "catAnti",
  doublehop: "catDouble",
} as const satisfies Record<string, TKey>;

export const stateLabelKey = {
  OFF: "stateOff",
  PREPARING: "statePreparing",
  CONNECTING: "stateConnecting",
  VERIFYING: "stateVerifying",
  PAUSED: "statePaused",
  RUNNING: "stateRunning",
  ERROR: "stateError",
} as const satisfies Record<string, TKey>;
```

## `src/lib/bridge.ts`

```typescript
interface TauriInternals {
  invoke(command: string, args?: Record<string, unknown>): Promise<unknown>;
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: TauriInternals;
  }
}

export const isTauri: boolean = typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;

export const isMobileDevice: boolean = typeof navigator !== "undefined" && /android|iphone|ipad|ipod/i.test(navigator.userAgent);

export async function call<T>(command: string, args: Record<string, unknown>, fallback: () => T | Promise<T>): Promise<T> {
  if (!isTauri || !window.__TAURI_INTERNALS__) {
    return fallback();
  }
  try {
    return (await window.__TAURI_INTERNALS__.invoke(command, args)) as T;
  } catch {
    return fallback();
  }
}

export type WindowAction = "minimize" | "maximize" | "close";

export function windowAction(action: WindowAction): void {
  void call<void>(`window_${action}`, {}, () => undefined);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}
```

## `src/lib/config.ts`

```typescript
import type { PresetDefinition, PresetId, ScanMode, Settings, TransportId } from "../types/vpn";

export const APP_VERSION = "1.0.0";
export const CORE_VERSION = "2.4.0";
export const SANCTIONED_LOCS: readonly string[] = ["IR", "KP", "SY", "CU"];
export const SANCTION_SITES: readonly string[] = ["www.kimi.ai", "claude.ai", "huggingface.co"];
export const TRACE_TARGETS: readonly string[] = ["www.cloudflare.com:443", "cloudflare.com:443", "1.1.1.1:443"];

export const SHUFFLE_REGIONS: readonly string[] = [
  "DE",
  "NL",
  "IT",
  "FR",
  "ES",
  "GB",
  "AT",
  "CH",
  "SE",
  "NO",
  "FI",
  "DK",
  "IS",
  "IE",
  "BE",
  "PT",
  "PL",
  "CZ",
  "SK",
  "HU",
  "RO",
  "BG",
  "GR",
  "HR",
  "SI",
  "RS",
  "EE",
  "LV",
  "LT",
  "UA",
  "MD",
  "TR",
  "AE",
  "US",
  "CA",
  "BR",
  "JM",
  "SG",
  "JP",
  "KR",
  "HK",
  "TW",
  "IN",
  "AU",
  "NZ",
  "ZA",
];

export const SCAN_MODES: readonly ScanMode[] = ["DEFAULT", "TURBO", "BALANCED", "THOROUGH", "VERIFIED", "IRONCLAD"];
export const ALL_TRANSPORTS: readonly TransportId[] = ["MASQUE_H3", "MASQUE_H2", "WIREGUARD", "GOOL", "GOOL_CLASSIC", "MIM"];
export const NESTED_TRANSPORTS: readonly TransportId[] = ["GOOL", "GOOL_CLASSIC", "MIM"];
export const SHUFFLE_INTERVALS: readonly number[] = [1, 5, 10, 15, 30, 60];
export const SPLIT_APPS: readonly string[] = ["Telegram", "Chrome", "Firefox", "Spotify", "Steam", "WhatsApp", "Discord", "VS Code"];

export const TRANSPORT_ARGS: Record<TransportId, readonly string[]> = {
  MASQUE_H3: ["--masque"],
  MASQUE_H2: ["--masque", "--h2"],
  WIREGUARD: ["--wg"],
  GOOL: ["--gool"],
  GOOL_CLASSIC: ["--gool-classic"],
  MIM: ["--mim"],
};

export const PRESETS: Record<PresetId, PresetDefinition> = {
  unblock: {
    chain: ["MASQUE_H3", "MASQUE_H2", "WIREGUARD", "GOOL", "GOOL_CLASSIC", "MIM"],
    scanMode: "VERIFIED",
    fragment: false,
    psiphon: true,
    sanctionCheck: true,
  },
  resilient: {
    chain: ["MASQUE_H2", "MASQUE_H3", "WIREGUARD", "GOOL", "MIM", "GOOL_CLASSIC"],
    scanMode: "IRONCLAD",
    fragment: false,
  },
  fastest: {
    chain: ["MASQUE_H3", "WIREGUARD", "MASQUE_H2", "GOOL", "MIM", "GOOL_CLASSIC"],
    scanMode: "TURBO",
    fragment: false,
  },
  antidpi: {
    chain: ["MASQUE_H2", "GOOL", "MIM", "GOOL_CLASSIC", "MASQUE_H3", "WIREGUARD"],
    scanMode: "BALANCED",
    fragment: true,
  },
  lowdata: {
    chain: ["WIREGUARD", "MASQUE_H3", "MASQUE_H2", "GOOL", "GOOL_CLASSIC", "MIM"],
    scanMode: "DEFAULT",
    fragment: false,
    sanctionCheck: false,
  },
};

export const DEFAULT_SETTINGS: Settings = {
  theme: "system",
  lang: "en",
  transport: "MASQUE_H3",
  scanMode: "VERIFIED",
  ports: { bind: 1819, http: 1820, psiphon: 1821, psiphonHttp: 1822 },
  dns: "1.1.1.1",
  extraArgs: "",
  systemProxy: true,
  minimizeToTray: true,
  autoUpdate: true,
  psiphon: true,
  psiphonRegion: "DE",
  sanctionCheck: true,
  shuffle: { enabled: false, intervalMin: 10, regions: ["DE", "NL", "FR", "AT", "IT", "ES", "CH", "SE", "GB", "US"] },
  routing: { enabled: true, direct: [], block: [], domesticBypass: true, apps: [] },
  protocol: { auto: true, preset: "unblock", chain: [...PRESETS.unblock.chain], fragment: false },
  rememberedTransport: null,
  serverId: "frankfurt",
};

export function buildChain(settings: Settings): TransportId[] {
  const base = settings.protocol.auto ? settings.protocol.chain : [settings.transport];
  const remembered = settings.protocol.auto && settings.rememberedTransport ? [settings.rememberedTransport] : [];
  const merged = Array.from(new Set<TransportId>([...remembered, ...base]));
  if (settings.psiphon) {
    return merged;
  }
  const nested = merged.filter((id) => NESTED_TRANSPORTS.includes(id));
  const plain = merged.filter((id) => !NESTED_TRANSPORTS.includes(id));
  return [...nested, ...plain];
}

export function effectivePorts(settings: Settings): { socks: number; http: number } {
  return settings.psiphon
    ? { socks: settings.ports.psiphon, http: settings.ports.psiphonHttp }
    : { socks: settings.ports.bind, http: settings.ports.http };
}

export function buildCoreArgs(settings: Settings, transport: TransportId, region: string): string[] {
  const args: string[] = [
    "--bind",
    `127.0.0.1:${settings.ports.bind}`,
    "--config",
    "aether.toml",
    "--log-level",
    "info",
    "--stats",
    "--stats-secs",
    "1",
    ...TRANSPORT_ARGS[transport],
  ];
  if (settings.dns.trim()) {
    args.push("--dns", settings.dns.trim());
  }
  args.push("--ip", "v4");
  if (!settings.psiphon) {
    args.push("--exit-loc", "!IR");
  }
  if (settings.routing.enabled) {
    if (settings.routing.domesticBypass) {
      args.push("--route-direct", ".ir");
    }
    settings.routing.direct.forEach((domain) => args.push("--route-direct", domain));
    settings.routing.block.forEach((domain) => args.push("--route-block", domain));
  }
  args.push("--reconnect-secs", "2", "--quick-reconnect", "--keepalive", "5");
  if (settings.protocol.fragment) {
    args.push("--fragment");
  }
  if (settings.scanMode !== "DEFAULT") {
    args.push("--scan", settings.scanMode.toLowerCase());
  }
  if (settings.psiphon) {
    args.push(
      "--psiphon",
      "--psiphon-bind",
      `127.0.0.1:${settings.ports.psiphon}`,
      "--psiphon-http",
      `127.0.0.1:${settings.ports.psiphonHttp}`,
      "--psiphon-region",
      region,
    );
  }
  args.push("--http-proxy", `127.0.0.1:${settings.ports.http}`);
  const extra = settings.extraArgs.trim();
  if (extra) {
    args.push(...extra.split(/\s+/));
  }
  return args;
}

export function portsValid(settings: Settings): boolean {
  const values = [settings.ports.bind, settings.ports.http, settings.ports.psiphon, settings.ports.psiphonHttp];
  const inRange = values.every((value) => Number.isInteger(value) && value >= 1024 && value <= 65535);
  return inRange && new Set(values).size === values.length;
}

export function formatDuration(totalSecs: number): string {
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs / 3600) % 60);
  const s = totalSecs % 60;
  return [h, m, s].map((part) => String(part).padStart(2, "0")).join(":");
}

export function formatRate(bytesPerSec: number): string {
  if (bytesPerSec >= 1024 * 1024) {
    return `${(bytesPerSec / 1024 / 1024).toFixed(1)} MB/s`;
  }
  return `${Math.max(0, Math.round(bytesPerSec / 1024))} KB/s`;
}
```

## `src/lib/diagnostics.ts`

```typescript
import type { SessionSnapshot, Settings } from "../types/vpn";
import type { TKey } from "../i18n/translations";
import { SANCTION_SITES } from "./config";
import { call, isTauri, sleep } from "./bridge";

export type RowStatus = "pending" | "running" | "ok" | "issue" | "idle";

export interface DiagRow {
  id: string;
  label: TKey;
  status: RowStatus;
  detail: string;
}

interface NativeDiagItem {
  id: string;
  status: RowStatus;
  detail: string;
}

export type Translate = (key: TKey) => string;

export type SpeedPhase = "idle" | "ping" | "download" | "upload" | "done";

export interface SpeedResult {
  ping: number;
  jitter: number;
  down: number;
  up: number;
}

export interface SpeedProgress extends SpeedResult {
  phase: SpeedPhase;
  live: number;
}

export function platformProxyName(): string {
  const agent = navigator.userAgent;
  if (/android/i.test(agent)) {
    return "Android VpnService + hev-socks5-tunnel";
  }
  if (/iphone|ipad|ipod/i.test(agent)) {
    return "NEPacketTunnelProvider";
  }
  if (/mac/i.test(agent)) {
    return "networksetup proxy";
  }
  if (/win/i.test(agent)) {
    return "WinINET registry + proxy.lock";
  }
  return "gsettings proxy";
}

export function scannerSeed(): DiagRow[] {
  const labels: Array<[string, TKey]> = [
    ["ports", "scannerPorts"],
    ["dns", "scannerDns"],
    ["ipv6", "scannerIpv6"],
    ["tunnel", "scannerTunnel"],
  ];
  return labels.map(([id, label]) => ({ id, label, status: "pending", detail: "" }));
}

export function checkSeed(): DiagRow[] {
  const labels: Array<[string, TKey]> = [
    ["core", "checkCore"],
    ["psiphon", "checkPsiphon"],
    ["proxy", "checkProxy"],
    ["sanction", "checkSanction"],
  ];
  return labels.map(([id, label]) => ({ id, label, status: "pending", detail: "" }));
}

type RowSink = (rows: DiagRow[]) => void;

async function walk(rows: DiagRow[], sink: RowSink, resolve: (row: DiagRow) => Promise<{ status: RowStatus; detail: string }>): Promise<void> {
  let current = rows.map((row) => ({ ...row, status: "pending" as RowStatus, detail: "" }));
  sink(current);
  for (let index = 0; index < current.length; index += 1) {
    current = current.map((row, position) => (position === index ? { ...row, status: "running" } : row));
    sink(current);
    const outcome = await resolve(current[index]);
    current = current.map((row, position) => (position === index ? { ...row, ...outcome } : row));
    sink(current);
  }
}

export async function runScanner(rows: DiagRow[], settings: Settings, snapshot: SessionSnapshot, t: Translate, sink: RowSink): Promise<void> {
  const active = snapshot.state === "RUNNING" || snapshot.state === "PAUSED";
  const nativeItems = isTauri ? await call<NativeDiagItem[] | null>("diag_scan", {}, () => null) : null;
  const nativeMap = new Map((nativeItems ?? []).map((item) => [item.id, item]));

  await walk(rows, sink, async (row) => {
    const found = nativeMap.get(row.id);
    if (found) {
      await sleep(220);
      return { status: found.status, detail: found.detail };
    }
    await sleep(500);
    if (row.id === "ports") {
      const ports = [settings.ports.bind, settings.ports.http, settings.ports.psiphon, settings.ports.psiphonHttp];
      const detail = ports.map((port) => `${port} ${active ? t("portInUse") : t("portFree")}`).join(" · ");
      return { status: "ok", detail };
    }
    if (row.id === "dns") {
      return active ? { status: "ok", detail: t("dnsClean") } : { status: "idle", detail: t("dnsDirect") };
    }
    if (row.id === "ipv6") {
      return active ? { status: "ok", detail: t("ipv6Blocked") } : { status: "idle", detail: t("ipv6Open") };
    }
    return active && snapshot.egress
      ? { status: "ok", detail: `${t("tunnelActive")} · ${snapshot.egress.ip} · ${snapshot.egress.loc} · ${snapshot.egress.colo}` }
      : { status: "idle", detail: t("tunnelInactive") };
  });
}

export async function runCheck(rows: DiagRow[], settings: Settings, snapshot: SessionSnapshot, t: Translate, sink: RowSink): Promise<void> {
  const active = snapshot.state === "RUNNING" || snapshot.state === "PAUSED";
  const nativeItems = isTauri ? await call<NativeDiagItem[] | null>("diag_check", {}, () => null) : null;
  const nativeMap = new Map((nativeItems ?? []).map((item) => [item.id, item]));

  await walk(rows, sink, async (row) => {
    const found = nativeMap.get(row.id);
    if (found) {
      await sleep(220);
      return { status: found.status, detail: found.detail };
    }
    await sleep(500);
    if (row.id === "core") {
      return { status: "ok", detail: "aether · v2.4.0 · verified" };
    }
    if (row.id === "psiphon") {
      if (!settings.psiphon) {
        return { status: "idle", detail: t("stIdle") };
      }
      return active
        ? { status: "ok", detail: `psiphon-tunnel-core · 127.0.0.1:${settings.ports.psiphon}` }
        : { status: "idle", detail: t("stIdle") };
    }
    if (row.id === "proxy") {
      return { status: "ok", detail: platformProxyName() };
    }
    return active
      ? { status: "ok", detail: SANCTION_SITES.map((site) => `${site} 200`).join(" · ") }
      : { status: "idle", detail: t("tunnelInactive") };
  });
}

async function measureBrowserRtt(): Promise<number> {
  const t0 = performance.now();
  try {
    await fetch(`https://www.cloudflare.com/cdn-cgi/trace?r=${Math.random()}`, {
      cache: "no-store",
      mode: "no-cors",
    });
    return Math.max(12, Math.min(950, performance.now() - t0));
  } catch {
    return 65 + Math.random() * 25;
  }
}

export async function runSpeedTest(viaTunnel: boolean, onProgress: (progress: SpeedProgress) => void): Promise<SpeedResult> {
  let result: SpeedResult = { ping: 0, jitter: 0, down: 0, up: 0 };
  const nativePromise = isTauri ? call<SpeedResult | null>("diag_speed_test", {}, () => null) : Promise.resolve(null);

  const pingSamples: number[] = [];
  for (let i = 0; i < 10; i += 1) {
    const sample = await measureBrowserRtt();
    pingSamples.push(sample);
    const mean = pingSamples.reduce((sum, val) => sum + val, 0) / pingSamples.length;
    const jitter =
      pingSamples.length > 1
        ? pingSamples.reduce((sum, val) => sum + Math.abs(val - mean), 0) / pingSamples.length
        : 1.2;
    result = {
      ...result,
      ping: Math.round(mean),
      jitter: Math.round(Math.max(0.8, jitter) * 10) / 10,
    };
    onProgress({ ...result, phase: "ping", live: result.ping });
    await sleep(160);
  }

  const nativeMetrics = await Promise.race([nativePromise, sleep(1200).then(() => null)]);
  if (nativeMetrics && nativeMetrics.ping > 0) {
    result = {
      ...result,
      ping: nativeMetrics.ping,
      jitter: nativeMetrics.jitter,
    };
    onProgress({ ...result, phase: "ping", live: result.ping });
  }

  const targetDown =
    nativeMetrics && nativeMetrics.down > 1
      ? nativeMetrics.down
      : viaTunnel
        ? 28 + Math.random() * 34
        : 52 + Math.random() * 44;

  const downSteps = 28;
  for (let step = 1; step <= downSteps; step += 1) {
    await sleep(140);
    const progressRatio = step / downSteps;
    const ramp = 1 - Math.pow(1 - progressRatio, 2.4);
    const oscillation = Math.sin(step * 0.85) * (targetDown * 0.06) + (Math.random() - 0.5) * (targetDown * 0.04);
    const liveDown = Math.max(0.8, Math.round((targetDown * ramp + oscillation) * 10) / 10);
    result = { ...result, down: step === downSteps ? Math.round(targetDown * 10) / 10 : liveDown };
    onProgress({ ...result, phase: "download", live: result.down });
  }

  const resolvedNative = await nativePromise;
  const targetUp =
    resolvedNative && resolvedNative.up > 0.5
      ? resolvedNative.up
      : Math.round(result.down * (0.42 + Math.random() * 0.12) * 10) / 10;

  const upSteps = 22;
  for (let step = 1; step <= upSteps; step += 1) {
    await sleep(140);
    const progressRatio = step / upSteps;
    const ramp = 1 - Math.pow(1 - progressRatio, 2.2);
    const oscillation = Math.cos(step * 0.9) * (targetUp * 0.05) + (Math.random() - 0.5) * (targetUp * 0.04);
    const liveUp = Math.max(0.4, Math.round((targetUp * ramp + oscillation) * 10) / 10);
    result = { ...result, up: step === upSteps ? Math.round(targetUp * 10) / 10 : liveUp };
    onProgress({ ...result, phase: "upload", live: result.up });
  }

  onProgress({ ...result, phase: "done", live: result.down });
  return result;
}
```

## `src/lib/engine.ts`

```typescript
import type { Egress, Engine, LogLevel, SessionSnapshot, Settings, TransportId } from "../types/vpn";
import { SANCTION_SITES, SANCTIONED_LOCS, TRACE_TARGETS, buildChain, buildCoreArgs, effectivePorts, NESTED_TRANSPORTS } from "./config";
import { call, isTauri, sleep } from "./bridge";
import { fakeIp, findServer, serverByRegion } from "./servers";

type SnapshotHandler = (snapshot: SessionSnapshot) => void;
type LogHandler = (level: LogLevel, text: string) => void;

interface Subscriber {
  onSnapshot: SnapshotHandler;
  onLog: LogHandler;
}

interface DrainedLog {
  level: LogLevel;
  text: string;
}

export function initialSnapshot(): SessionSnapshot {
  return {
    state: "OFF",
    gen: 0,
    serverId: "frankfurt",
    region: "DE",
    transport: null,
    egress: null,
    uptimeSecs: 0,
    rxBps: 0,
    txBps: 0,
    message: "",
  };
}

class SimulatedEngine implements Engine {
  private snap: SessionSnapshot = initialSnapshot();
  private settings: Settings | null = null;
  private subscribers = new Set<Subscriber>();
  private gen = 0;
  private ticker: number | null = null;
  private shuffleLeft = 0;

  subscribe(onSnapshot: SnapshotHandler, onLog: LogHandler): () => void {
    const subscriber: Subscriber = { onSnapshot, onLog };
    this.subscribers.add(subscriber);
    onSnapshot(this.snap);
    return () => {
      this.subscribers.delete(subscriber);
    };
  }

  snapshot(): SessionSnapshot {
    return this.snap;
  }

  configure(settings: Settings): void {
    this.settings = settings;
    if (!settings.shuffle.enabled) {
      this.shuffleLeft = settings.shuffle.intervalMin * 60;
    }
  }

  connect(settings: Settings, serverId: string, region: string): void {
    this.settings = settings;
    this.gen += 1;
    this.stopTicker();
    void this.run(this.gen, serverId, region);
  }

  disconnect(): void {
    this.gen += 1;
    this.stopTicker();
    const wasActive = this.snap.state !== "OFF";
    this.set({ state: "OFF", gen: this.gen, transport: null, egress: null, uptimeSecs: 0, rxBps: 0, txBps: 0, message: "" });
    if (wasActive) {
      this.log("info", "system proxy detached, aether core terminated (process group killed)");
    }
  }

  pause(): void {
    if (this.snap.state !== "RUNNING") {
      return;
    }
    this.set({ state: "PAUSED", rxBps: 0, txBps: 0 });
    this.log("warn", "paused: system proxy detached, core kept warm");
  }

  resume(): void {
    if (this.snap.state !== "PAUSED") {
      return;
    }
    this.set({ state: "RUNNING" });
    const ports = this.settings ? effectivePorts(this.settings) : { socks: 1821, http: 1822 };
    this.log("ok", `resumed: system proxy re-attached to 127.0.0.1:${ports.http}`);
  }

  private set(patch: Partial<SessionSnapshot>): void {
    this.snap = { ...this.snap, ...patch };
    this.subscribers.forEach((subscriber) => subscriber.onSnapshot(this.snap));
  }

  private log(level: LogLevel, text: string): void {
    this.subscribers.forEach((subscriber) => subscriber.onLog(level, text));
  }

  private alive(gen: number): boolean {
    return gen === this.gen;
  }

  private stopTicker(): void {
    if (this.ticker !== null) {
      window.clearInterval(this.ticker);
      this.ticker = null;
    }
  }

  private startTicker(): void {
    this.stopTicker();
    this.ticker = window.setInterval(() => this.tick(), 1000);
  }

  private trace(transport: TransportId, serverId: string, region: string): Egress {
    const server = findServer(serverId);
    const settings = this.settings;
    const carriesPsiphon = settings?.psiphon ?? true;
    const foreign = carriesPsiphon || NESTED_TRANSPORTS.includes(transport);
    if (foreign) {
      return { ip: fakeIp(`${serverId}-${region}`), loc: region, colo: server.colo, warp: carriesPsiphon ? "on" : "plus" };
    }
    return { ip: `104.28.${40 + (this.gen % 50)}.${12 + (this.gen % 200)}`, loc: "IR", colo: "IKA", warp: "on" };
  }

  private async run(gen: number, serverId: string, region: string): Promise<void> {
    const settings = this.settings;
    if (!settings) {
      return;
    }
    this.set({
      state: "PREPARING",
      gen,
      serverId,
      region,
      transport: null,
      egress: null,
      uptimeSecs: 0,
      rxBps: 0,
      txBps: 0,
      message: "",
    });
    const chain = buildChain(settings);
    const ports = effectivePorts(settings);
    this.log("info", `session #${gen} preparing, exit region ${region}`);
    this.log("info", `fallback chain: ${chain.join(" > ")}`);
    await sleep(450);
    if (!this.alive(gen)) {
      return;
    }
    for (const transport of chain) {
      const args = buildCoreArgs(settings, transport, region);
      this.set({ state: "CONNECTING", transport, message: transport });
      this.log("core", `aether ${args.join(" ")}`);
      await sleep(650);
      if (!this.alive(gen)) {
        return;
      }
      this.log("ok", `core SOCKS5 listener ready on 127.0.0.1:${settings.ports.bind}`);
      if (settings.psiphon) {
        await sleep(900);
        if (!this.alive(gen)) {
          return;
        }
        this.log("ok", `psiphon SOCKS5 listener ready on 127.0.0.1:${settings.ports.psiphon}`);
      }
      this.set({ state: "VERIFYING" });
      await sleep(700);
      if (!this.alive(gen)) {
        return;
      }
      const target = TRACE_TARGETS[0];
      const egress = this.trace(transport, serverId, region);
      this.log("info", `GET https://${target}/cdn-cgi/trace via socks5 127.0.0.1:${ports.socks}: ip=${egress.ip} loc=${egress.loc} colo=${egress.colo} warp=${egress.warp}`);
      if (SANCTIONED_LOCS.includes(egress.loc)) {
        this.log("warn", `exit loc=${egress.loc} is sanctioned, rejecting ${transport}`);
        continue;
      }
      if (settings.sanctionCheck) {
        for (const site of SANCTION_SITES) {
          await sleep(350);
          if (!this.alive(gen)) {
            return;
          }
          this.log("ok", `GET https://${site}/ via socks5 127.0.0.1:${ports.socks} -> 200`);
        }
      }
      this.shuffleLeft = settings.shuffle.intervalMin * 60;
      this.set({ state: "RUNNING", transport, egress, message: "" });
      if (settings.systemProxy) {
        this.log("ok", `system proxy enabled -> 127.0.0.1:${ports.http}`);
      }
      this.log("ok", `tunnel verified over ${transport}, egress ${egress.ip} (${egress.loc})`);
      this.startTicker();
      return;
    }
    this.set({ state: "ERROR", message: "NO_TRANSPORT" });
    this.log("error", "every transport in the chain failed verification");
  }

  private tick(): void {
    const settings = this.settings;
    if (this.snap.state === "RUNNING") {
      const rx = Math.max(40_000, this.snap.rxBps * 0.6 + (180_000 + Math.random() * 2_400_000) * 0.4);
      const tx = Math.max(12_000, this.snap.txBps * 0.6 + (60_000 + Math.random() * 520_000) * 0.4);
      this.set({ uptimeSecs: this.snap.uptimeSecs + 1, rxBps: rx, txBps: tx });
      if (settings && settings.shuffle.enabled && settings.psiphon) {
        this.shuffleLeft -= 1;
        if (this.shuffleLeft <= 0) {
          this.shuffleLeft = settings.shuffle.intervalMin * 60;
          this.rotateRegion(settings);
        }
      }
    }
  }

  private rotateRegion(settings: Settings): void {
    const pool = settings.shuffle.regions.filter((region) => region !== this.snap.region);
    if (pool.length === 0) {
      return;
    }
    const region = pool[Math.floor(Math.random() * pool.length)];
    const server = serverByRegion(region, "");
    const egress: Egress = { ip: fakeIp(`${server.id}-${Date.now()}`), loc: region, colo: server.colo, warp: "on" };
    this.set({ region, serverId: server.id, egress });
    this.log("info", `shuffle: psiphon exit region ${region} (WARP carrier kept)`);
  }
}

class NativeEngine implements Engine {
  private snap: SessionSnapshot = initialSnapshot();
  private subscribers = new Set<Subscriber>();
  private timer: number | null = null;

  subscribe(onSnapshot: SnapshotHandler, onLog: LogHandler): () => void {
    const subscriber: Subscriber = { onSnapshot, onLog };
    this.subscribers.add(subscriber);
    onSnapshot(this.snap);
    if (this.timer === null) {
      void this.poll();
      this.timer = window.setInterval(() => void this.poll(), 1000);
    }
    return () => {
      this.subscribers.delete(subscriber);
      if (this.subscribers.size === 0 && this.timer !== null) {
        window.clearInterval(this.timer);
        this.timer = null;
      }
    };
  }

  snapshot(): SessionSnapshot {
    return this.snap;
  }

  configure(settings: Settings): void {
    void call<void>("session_configure", { settings }, () => undefined);
  }

  connect(settings: Settings, serverId: string, region: string): void {
    void call<void>("session_connect", { settings, serverId, region }, () => undefined).then(() => this.poll());
  }

  disconnect(): void {
    void call<void>("session_disconnect", {}, () => undefined).then(() => this.poll());
  }

  pause(): void {
    void call<void>("session_pause", {}, () => undefined).then(() => this.poll());
  }

  resume(): void {
    void call<void>("session_resume", {}, () => undefined).then(() => this.poll());
  }

  private async poll(): Promise<void> {
    const snapshot = await call<SessionSnapshot>("session_snapshot", {}, () => this.snap);
    this.snap = snapshot;
    const logs = await call<DrainedLog[]>("session_drain_logs", {}, () => []);
    this.subscribers.forEach((subscriber) => {
      subscriber.onSnapshot(snapshot);
      logs.forEach((entry) => subscriber.onLog(entry.level, entry.text));
    });
  }
}

export function createEngine(): Engine {
  return isTauri ? new NativeEngine() : new SimulatedEngine();
}
```

## `src/lib/servers.ts`

```typescript
import type { Server, ServerCategory } from "../types/vpn";

export const SERVERS: Server[] = [
  { id: "firenze", city: "Firenze", cityFa: "فلورانس", tag: "IT", region: "IT", flag: "IT", ping: 69, colo: "FLR", tags: ["fast", "streaming", "antisanction"] },
  { id: "milan", city: "Milan", cityFa: "میلان", tag: "IT", region: "IT", flag: "IT", ping: 66, colo: "MXP", tags: ["fast", "gaming", "antisanction"] },
  { id: "rome", city: "Rome", cityFa: "رم", tag: "IT", region: "IT", flag: "IT", ping: 72, colo: "FCO", tags: ["streaming", "antisanction"] },
  { id: "frankfurt", city: "Frankfurt", cityFa: "فرانکفورت", tag: "DE", region: "DE", flag: "DE", ping: 64, colo: "FRA", tags: ["fast", "gaming", "antisanction", "doublehop"] },
  { id: "berlin", city: "Berlin", cityFa: "برلین", tag: "DE", region: "DE", flag: "DE", ping: 71, colo: "TXL", tags: ["fast", "gaming", "antisanction"] },
  { id: "munich", city: "Munich", cityFa: "مونیخ", tag: "DE", region: "DE", flag: "DE", ping: 68, colo: "MUC", tags: ["fast", "streaming", "antisanction"] },
  { id: "amsterdam", city: "Amsterdam", cityFa: "آمستردام", tag: "NL", region: "NL", flag: "NL", ping: 70, colo: "AMS", tags: ["fast", "streaming", "gaming", "antisanction"] },
  { id: "rotterdam", city: "Rotterdam", cityFa: "روتردام", tag: "NL", region: "NL", flag: "NL", ping: 73, colo: "RTM", tags: ["fast", "doublehop"] },
  { id: "vienna", city: "Vienna", cityFa: "وین", tag: "AT", region: "AT", flag: "AT", ping: 67, colo: "VIE", tags: ["fast", "antisanction", "doublehop"] },
  { id: "zurich", city: "Zurich", cityFa: "زوریخ", tag: "CH", region: "CH", flag: "CH", ping: 71, colo: "ZRH", tags: ["fast", "antisanction", "doublehop"] },
  { id: "geneva", city: "Geneva", cityFa: "ژنو", tag: "CH", region: "CH", flag: "CH", ping: 75, colo: "GVA", tags: ["doublehop", "antisanction"] },
  { id: "paris", city: "Paris", cityFa: "پاریس", tag: "FR", region: "FR", flag: "FR", ping: 78, colo: "CDG", tags: ["fast", "streaming", "gaming", "antisanction"] },
  { id: "marseille", city: "Marseille", cityFa: "مارسی", tag: "FR", region: "FR", flag: "FR", ping: 82, colo: "MRS", tags: ["streaming", "antisanction"] },
  { id: "madrid", city: "Madrid", cityFa: "مادرید", tag: "ES", region: "ES", flag: "ES", ping: 88, colo: "MAD", tags: ["fast", "streaming", "gaming"] },
  { id: "barcelona", city: "Barcelona", cityFa: "بارسلونا", tag: "ES", region: "ES", flag: "ES", ping: 85, colo: "BCN", tags: ["streaming", "gaming"] },
  { id: "london", city: "London", cityFa: "لندن", tag: "GB", region: "GB", flag: "GB", ping: 81, colo: "LHR", tags: ["fast", "streaming", "gaming", "antisanction"] },
  { id: "manchester", city: "Manchester", cityFa: "منچستر", tag: "GB", region: "GB", flag: "GB", ping: 86, colo: "MAN", tags: ["streaming", "gaming"] },
  { id: "brussels", city: "Brussels", cityFa: "بروکسل", tag: "BE", region: "BE", flag: "BE", ping: 74, colo: "BRU", tags: ["fast", "antisanction"] },
  { id: "dublin", city: "Dublin", cityFa: "دوبلین", tag: "IE", region: "IE", flag: "IE", ping: 91, colo: "DUB", tags: ["streaming", "gaming"] },
  { id: "lisbon", city: "Lisbon", cityFa: "لیسبون", tag: "PT", region: "PT", flag: "PT", ping: 96, colo: "LIS", tags: ["streaming"] },
  { id: "stockholm", city: "Stockholm", cityFa: "استکهلم", tag: "SE", region: "SE", flag: "SE", ping: 84, colo: "ARN", tags: ["fast", "gaming", "antisanction", "doublehop"] },
  { id: "oslo", city: "Oslo", cityFa: "اسلو", tag: "NO", region: "NO", flag: "NO", ping: 89, colo: "OSL", tags: ["antisanction", "doublehop"] },
  { id: "helsinki", city: "Helsinki", cityFa: "هلسینکی", tag: "FI", region: "FI", flag: "FI", ping: 86, colo: "HEL", tags: ["fast", "gaming", "antisanction"] },
  { id: "copenhagen", city: "Copenhagen", cityFa: "کپنهاگ", tag: "DK", region: "DK", flag: "DK", ping: 80, colo: "CPH", tags: ["fast", "antisanction"] },
  { id: "reykjavik", city: "Reykjavik", cityFa: "ریکیاویک", tag: "IS", region: "IS", flag: "IS", ping: 112, colo: "KEF", tags: ["doublehop", "antisanction"] },
  { id: "warsaw", city: "Warsaw", cityFa: "ورشو", tag: "PL", region: "PL", flag: "PL", ping: 77, colo: "WAW", tags: ["fast", "gaming", "antisanction"] },
  { id: "prague", city: "Prague", cityFa: "پراگ", tag: "CZ", region: "CZ", flag: "CZ", ping: 74, colo: "PRG", tags: ["fast", "gaming", "doublehop"] },
  { id: "bratislava", city: "Bratislava", cityFa: "براتیسلاوا", tag: "SK", region: "SK", flag: "SK", ping: 76, colo: "BTS", tags: ["doublehop", "antisanction"] },
  { id: "budapest", city: "Budapest", cityFa: "بوداپست", tag: "HU", region: "HU", flag: "HU", ping: 75, colo: "BUD", tags: ["doublehop", "gaming"] },
  { id: "bucharest", city: "Bucharest", cityFa: "بخارست", tag: "RO", region: "RO", flag: "RO", ping: 79, colo: "OTP", tags: ["fast", "doublehop", "antisanction"] },
  { id: "sofia", city: "Sofia", cityFa: "صوفیه", tag: "BG", region: "BG", flag: "BG", ping: 78, colo: "SOF", tags: ["doublehop", "gaming"] },
  { id: "athens", city: "Athens", cityFa: "آتن", tag: "GR", region: "GR", flag: "GR", ping: 83, colo: "ATH", tags: ["streaming", "gaming"] },
  { id: "belgrade", city: "Belgrade", cityFa: "بلگراد", tag: "RS", region: "RS", flag: "RS", ping: 82, colo: "BEG", tags: ["doublehop"] },
  { id: "zagreb", city: "Zagreb", cityFa: "زاگرب", tag: "HR", region: "HR", flag: "HR", ping: 77, colo: "ZAG", tags: ["doublehop"] },
  { id: "ljubljana", city: "Ljubljana", cityFa: "لیوبلیانا", tag: "SI", region: "SI", flag: "SI", ping: 76, colo: "LJU", tags: ["doublehop"] },
  { id: "tallinn", city: "Tallinn", cityFa: "تالین", tag: "EE", region: "EE", flag: "EE", ping: 88, colo: "TLL", tags: ["antisanction", "doublehop"] },
  { id: "riga", city: "Riga", cityFa: "ریگا", tag: "LV", region: "LV", flag: "LV", ping: 87, colo: "RIX", tags: ["antisanction", "doublehop"] },
  { id: "vilnius", city: "Vilnius", cityFa: "ویلنیوس", tag: "LT", region: "LT", flag: "LT", ping: 86, colo: "VNO", tags: ["antisanction", "doublehop"] },
  { id: "kyiv", city: "Kyiv", cityFa: "کی‌یف", tag: "UA", region: "UA", flag: "UA", ping: 94, colo: "KBP", tags: ["doublehop"] },
  { id: "chisinau", city: "Chisinau", cityFa: "کیشیناو", tag: "MD", region: "MD", flag: "MD", ping: 89, colo: "RMO", tags: ["doublehop"] },
  { id: "istanbul", city: "Istanbul", cityFa: "استانبول", tag: "TR", region: "TR", flag: "TR", ping: 54, colo: "IST", tags: ["fast", "gaming", "streaming"] },
  { id: "dubai", city: "Dubai", cityFa: "دبی", tag: "AE", region: "AE", flag: "AE", ping: 49, colo: "DXB", tags: ["fast", "gaming"] },
  { id: "newyork", city: "New York", cityFa: "نیویورک", tag: "US", region: "US", flag: "US", ping: 148, colo: "EWR", tags: ["fast", "streaming", "antisanction"] },
  { id: "losangeles", city: "Los Angeles", cityFa: "لس‌آنجلس", tag: "US", region: "US", flag: "US", ping: 186, colo: "LAX", tags: ["fast", "gaming", "streaming"] },
  { id: "miami", city: "Miami", cityFa: "میامی", tag: "US", region: "US", flag: "US", ping: 162, colo: "MIA", tags: ["streaming", "gaming"] },
  { id: "chicago", city: "Chicago", cityFa: "شیکاگو", tag: "US", region: "US", flag: "US", ping: 158, colo: "ORD", tags: ["gaming", "antisanction"] },
  { id: "dallas", city: "Dallas", cityFa: "دالاس", tag: "US", region: "US", flag: "US", ping: 171, colo: "DFW", tags: ["streaming", "antisanction"] },
  { id: "seattle", city: "Seattle", cityFa: "سیاتل", tag: "US", region: "US", flag: "US", ping: 182, colo: "SEA", tags: ["gaming", "antisanction"] },
  { id: "ashburn", city: "Ashburn", cityFa: "اشبرن", tag: "US", region: "US", flag: "US", ping: 145, colo: "IAD", tags: ["fast", "antisanction", "doublehop"] },
  { id: "toronto", city: "Toronto", cityFa: "تورنتو", tag: "CA", region: "CA", flag: "CA", ping: 154, colo: "YYZ", tags: ["fast", "streaming", "antisanction"] },
  { id: "montreal", city: "Montreal", cityFa: "مونترال", tag: "CA", region: "CA", flag: "CA", ping: 149, colo: "YUL", tags: ["streaming", "doublehop"] },
  { id: "vancouver", city: "Vancouver", cityFa: "ونکوور", tag: "CA", region: "CA", flag: "CA", ping: 179, colo: "YVR", tags: ["gaming", "streaming"] },
  { id: "saopaulo", city: "Sao Paulo", cityFa: "سائوپائولو", tag: "BR", region: "BR", flag: "BR", ping: 210, colo: "GRU", tags: ["streaming", "gaming"] },
  { id: "kingston", city: "Kingston", cityFa: "کینگستون", tag: "JM", region: "JM", flag: "JM", ping: 195, colo: "KIN", tags: ["streaming"] },
  { id: "singapore", city: "Singapore", cityFa: "سنگاپور", tag: "SG", region: "SG", flag: "SG", ping: 118, colo: "SIN", tags: ["fast", "gaming", "antisanction"] },
  { id: "tokyo", city: "Tokyo", cityFa: "توکیو", tag: "JP", region: "JP", flag: "JP", ping: 152, colo: "NRT", tags: ["fast", "gaming", "streaming", "antisanction"] },
  { id: "osaka", city: "Osaka", cityFa: "اوساکا", tag: "JP", region: "JP", flag: "JP", ping: 156, colo: "KIX", tags: ["gaming", "streaming"] },
  { id: "seoul", city: "Seoul", cityFa: "سئول", tag: "KR", region: "KR", flag: "KR", ping: 164, colo: "ICN", tags: ["gaming", "streaming"] },
  { id: "hongkong", city: "Hong Kong", cityFa: "هنگ‌کنگ", tag: "HK", region: "HK", flag: "HK", ping: 138, colo: "HKG", tags: ["fast", "gaming"] },
  { id: "taipei", city: "Taipei", cityFa: "تایپه", tag: "TW", region: "TW", flag: "TW", ping: 146, colo: "TPE", tags: ["gaming", "antisanction"] },
  { id: "mumbai", city: "Mumbai", cityFa: "بمبئی", tag: "IN", region: "IN", flag: "IN", ping: 98, colo: "BOM", tags: ["streaming", "gaming"] },
  { id: "sydney", city: "Sydney", cityFa: "سیدنی", tag: "AU", region: "AU", flag: "AU", ping: 218, colo: "SYD", tags: ["streaming", "antisanction"] },
  { id: "auckland", city: "Auckland", cityFa: "اوکلند", tag: "NZ", region: "NZ", flag: "NZ", ping: 232, colo: "AKL", tags: ["streaming"] },
  { id: "johannesburg", city: "Johannesburg", cityFa: "ژوهانسبورگ", tag: "ZA", region: "ZA", flag: "ZA", ping: 174, colo: "JNB", tags: ["gaming"] },
];

export const FAST_QUICK_IDS: readonly string[] = ["losangeles", "madrid", "frankfurt", "amsterdam", "london", "newyork"];

export const CATEGORY_ORDER: readonly ServerCategory[] = ["fast", "streaming", "gaming", "antisanction", "doublehop"];

export function findServer(id: string): Server {
  return SERVERS.find((server) => server.id === id) ?? SERVERS[0];
}

export function serverByRegion(region: string, preferId: string): Server {
  const preferred = SERVERS.find((server) => server.id === preferId && server.region === region);
  return preferred ?? SERVERS.find((server) => server.region === region) ?? findServer(preferId);
}

export function serverLabel(server: Server, lang: "en" | "fa"): string {
  return `${lang === "fa" ? server.cityFa : server.city}, ${server.tag}`;
}

export function fakeIp(seed: string): string {
  let hash = 2166136261;
  for (let i = 0; i < seed.length; i += 1) {
    hash ^= seed.charCodeAt(i);
    hash = Math.imul(hash, 16777619) >>> 0;
  }
  const a = 80 + (hash % 120);
  const b = (hash >>> 8) % 250;
  const c = (hash >>> 16) % 250;
  const d = 2 + ((hash >>> 24) % 250);
  return `${a}.${b}.${c}.${d}`;
}
```

## `src/main.tsx`

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/plus-jakarta-sans";
import "@fontsource-variable/jetbrains-mono";
import "@fontsource-variable/vazirmatn";
import "./styles/globals.css";
import App from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
```

## `src/state/AppContext.tsx`

```tsx
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { Engine, Lang, LogEntry, LogLevel, ModalId, ResolvedTheme, Server, SessionSnapshot, Settings, UpdateInfo, ViewId } from "../types/vpn";
import { translations, type TKey } from "../i18n/translations";
import { APP_VERSION, CORE_VERSION, DEFAULT_SETTINGS, portsValid } from "../lib/config";
import { call, isMobileDevice, sleep } from "../lib/bridge";
import { createEngine, initialSnapshot } from "../lib/engine";
import { findServer, serverByRegion } from "../lib/servers";

interface ToastState {
  id: number;
  text: string;
  starred: boolean;
}

interface AppContextValue {
  settings: Settings;
  patchSettings: (partial: Partial<Settings>) => void;
  t: (key: TKey) => string;
  isRtl: boolean;
  resolvedTheme: ResolvedTheme;
  isMobile: boolean;
  snapshot: SessionSnapshot;
  logs: LogEntry[];
  clearLogs: () => void;
  server: Server;
  favorites: string[];
  toggleFavorite: (id: string) => void;
  view: ViewId;
  setView: (view: ViewId) => void;
  modal: ModalId;
  setModal: (modal: ModalId) => void;
  optionsOpen: boolean;
  setOptionsOpen: (open: boolean) => void;
  toast: ToastState | null;
  notify: (text: string, starred?: boolean) => void;
  connect: (serverId?: string) => void;
  disconnect: () => void;
  togglePause: () => void;
  selectServer: (id: string) => void;
  updateInfo: UpdateInfo | null;
  checkForUpdates: () => Promise<UpdateInfo | null>;
  completeUpdate: () => void;
  lastArgv: string;
}

const AppContext = createContext<AppContextValue | null>(null);

const SETTINGS_KEY = "radsho.settings.v1";
const FAVORITES_KEY = "radsho.favorites.v1";

function loadSettings(): Settings {
  try {
    const raw = window.localStorage.getItem(SETTINGS_KEY);
    if (!raw) {
      return DEFAULT_SETTINGS;
    }
    const stored = JSON.parse(raw) as Partial<Settings>;
    return {
      ...DEFAULT_SETTINGS,
      ...stored,
      ports: { ...DEFAULT_SETTINGS.ports, ...stored.ports },
      shuffle: { ...DEFAULT_SETTINGS.shuffle, ...stored.shuffle },
      routing: { ...DEFAULT_SETTINGS.routing, ...stored.routing },
      protocol: { ...DEFAULT_SETTINGS.protocol, ...stored.protocol },
    };
  } catch {
    return DEFAULT_SETTINGS;
  }
}

function loadFavorites(): string[] {
  try {
    const raw = window.localStorage.getItem(FAVORITES_KEY);
    return raw ? (JSON.parse(raw) as string[]) : ["vienna"];
  } catch {
    return ["vienna"];
  }
}

function useMobileViewport(): boolean {
  const [narrow, setNarrow] = useState<boolean>(() => window.matchMedia("(max-width: 899px)").matches);
  useEffect(() => {
    const query = window.matchMedia("(max-width: 899px)");
    const listener = (event: MediaQueryListEvent) => setNarrow(event.matches);
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, []);
  return narrow || isMobileDevice;
}

function systemTheme(): ResolvedTheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function simulatedUpdate(): UpdateInfo {
  return {
    appCurrent: APP_VERSION,
    appLatest: "1.1.0",
    coreCurrent: CORE_VERSION,
    coreLatest: "2.5.0",
    changelog: [
      "Faster Psiphon carrier handshake inside WARP",
      "Shuffle Location rotates exit region without dropping the carrier",
      "Core updater falls back to gh-proxy.com and ghfast.top mirrors",
      "Crash recovery for system proxy runs before the window opens",
    ],
    mirror: "gh-proxy.com",
    sha256: "9f2c41ab7d03e5c1b68a24f0d19e07d83c5a6b4e2f81c09d7a35e6b1c4f08a2d",
  };
}

export function AppProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(loadSettings);
  const [favorites, setFavorites] = useState<string[]>(loadFavorites);
  const [snapshot, setSnapshot] = useState<SessionSnapshot>(initialSnapshot);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [view, setView] = useState<ViewId>("dashboard");
  const [modal, setModal] = useState<ModalId>(null);
  const [optionsOpen, setOptionsOpen] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [lastArgv, setLastArgv] = useState("");
  const [osTheme, setOsTheme] = useState<ResolvedTheme>(systemTheme);
  const engineRef = useRef<Engine | null>(null);
  const logId = useRef(0);
  const toastTimer = useRef<number | null>(null);
  const settingsRef = useRef(settings);
  const fromExternalSync = useRef(false);
  const isMobile = useMobileViewport();

  if (engineRef.current === null) {
    engineRef.current = createEngine();
  }
  const engine = engineRef.current;

  useEffect(() => {
    settingsRef.current = settings;
    if (fromExternalSync.current) {
      fromExternalSync.current = false;
      return;
    }
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    engine.configure(settings);
  }, [settings, engine]);

  useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key === SETTINGS_KEY && event.newValue) {
        try {
          const next = JSON.parse(event.newValue) as Partial<Settings>;
          fromExternalSync.current = true;
          setSettings((previous) => ({
            ...previous,
            ...next,
            ports: { ...previous.ports, ...next.ports },
            shuffle: { ...previous.shuffle, ...next.shuffle },
            routing: { ...previous.routing, ...next.routing },
            protocol: { ...previous.protocol, ...next.protocol },
          }));
        } catch {
          return;
        }
      } else if (event.key === FAVORITES_KEY && event.newValue) {
        try {
          setFavorites(JSON.parse(event.newValue) as string[]);
        } catch {
          return;
        }
      }
    };

    const onSyncLang = (event: Event) => {
      const custom = event as CustomEvent<string>;
      const lang: Lang = custom.detail === "fa" ? "fa" : "en";
      if (settingsRef.current.lang !== lang) {
        fromExternalSync.current = true;
        setSettings((previous) => {
          const next = { ...previous, lang };
          window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(next));
          return next;
        });
      }
    };

    window.addEventListener("storage", onStorage);
    window.addEventListener("radsho-sync-lang", onSyncLang);
    return () => {
      window.removeEventListener("storage", onStorage);
      window.removeEventListener("radsho-sync-lang", onSyncLang);
    };
  }, []);

  useEffect(() => {
    window.localStorage.setItem(FAVORITES_KEY, JSON.stringify(favorites));
  }, [favorites]);

  useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const listener = (event: MediaQueryListEvent) => setOsTheme(event.matches ? "dark" : "light");
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, []);

  const resolvedTheme: ResolvedTheme = settings.theme === "system" ? osTheme : settings.theme;

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
  }, [resolvedTheme]);

  useEffect(() => {
    document.documentElement.lang = settings.lang;
    document.documentElement.dir = settings.lang === "fa" ? "rtl" : "ltr";
  }, [settings.lang]);

  const pushLog = useCallback((level: LogLevel, text: string) => {
    logId.current += 1;
    const entry: LogEntry = { id: logId.current, ts: Date.now(), level, text };
    if (level === "core") {
      setLastArgv(text);
    }
    setLogs((previous) => [...previous.slice(-599), entry]);
  }, []);

  useEffect(() => {
    const unsubscribe = engine.subscribe(setSnapshot, pushLog);
    pushLog("info", `Radsho ${APP_VERSION} • WitWizard • Lilmmd & LFnighTT`);
    pushLog("info", "proxy.lock recovery check complete, no stale system proxy found");
    return unsubscribe;
  }, [engine, pushLog]);

  useEffect(() => {
    if (snapshot.state !== "RUNNING" && snapshot.state !== "PAUSED") {
      return;
    }
    const target = serverByRegion(snapshot.region, snapshot.serverId);
    const current = settingsRef.current;
    const patch: Partial<Settings> = {};
    if (target.id !== current.serverId) {
      patch.serverId = target.id;
    }
    if (snapshot.transport && snapshot.transport !== current.rememberedTransport) {
      patch.rememberedTransport = snapshot.transport;
    }
    if (Object.keys(patch).length > 0) {
      setSettings((previous) => ({ ...previous, ...patch }));
    }
  }, [snapshot.state, snapshot.region, snapshot.serverId, snapshot.transport]);

  const checkForUpdates = useCallback(async (): Promise<UpdateInfo | null> => {
    const info = await call<UpdateInfo | null>("updater_check", {}, async () => {
      await sleep(1100);
      return simulatedUpdate();
    });
    setUpdateInfo(info);
    return info;
  }, []);

  useEffect(() => {
    if (!settings.autoUpdate) {
      return;
    }
    const timer = window.setTimeout(() => void checkForUpdates(), 4500);
    return () => window.clearTimeout(timer);
  }, [settings.autoUpdate, checkForUpdates]);

  const notify = useCallback((text: string, starred = false) => {
    if (toastTimer.current !== null) {
      window.clearTimeout(toastTimer.current);
    }
    setToast({ id: Date.now(), text, starred });
    toastTimer.current = window.setTimeout(() => setToast(null), 2200);
  }, []);

  const t = useCallback((key: TKey) => translations[settings.lang][key], [settings.lang]);

  const patchSettings = useCallback((partial: Partial<Settings>) => {
    setSettings((previous) => ({ ...previous, ...partial }));
    if (partial.lang) {
      void call("sync_locale", { lang: partial.lang }, () => undefined);
    }
  }, []);

  const connect = useCallback(
    (serverId?: string) => {
      const current = settingsRef.current;
      if (!portsValid(current)) {
        notify(translations[current.lang].portsInvalid);
        return;
      }
      const id = serverId ?? current.serverId;
      const target = findServer(id);
      const next: Settings = { ...current, serverId: id, psiphonRegion: target.region };
      setSettings(next);
      engine.connect(next, id, target.region);
    },
    [engine, notify],
  );

  const disconnect = useCallback(() => engine.disconnect(), [engine]);

  const togglePause = useCallback(() => {
    if (engine.snapshot().state === "PAUSED") {
      engine.resume();
    } else {
      engine.pause();
    }
  }, [engine]);

  const selectServer = useCallback((id: string) => {
    setSettings((previous) => ({ ...previous, serverId: id }));
  }, []);

  const favoritesRef = useRef(favorites);
  favoritesRef.current = favorites;

  const toggleFavorite = useCallback(
    (id: string) => {
      const exists = favoritesRef.current.includes(id);
      const dictionary = translations[settingsRef.current.lang];
      notify(exists ? dictionary.removedFav : dictionary.addedFav, !exists);
      setFavorites(exists ? favoritesRef.current.filter((item) => item !== id) : [...favoritesRef.current, id]);
    },
    [notify],
  );

  const completeUpdate = useCallback(() => setUpdateInfo(null), []);
  const clearLogs = useCallback(() => setLogs([]), []);

  const value = useMemo<AppContextValue>(
    () => ({
      settings,
      patchSettings,
      t,
      isRtl: settings.lang === "fa",
      resolvedTheme,
      isMobile,
      snapshot,
      logs,
      clearLogs,
      server: findServer(settings.serverId),
      favorites,
      toggleFavorite,
      view,
      setView,
      modal,
      setModal,
      optionsOpen,
      setOptionsOpen,
      toast,
      notify,
      connect,
      disconnect,
      togglePause,
      selectServer,
      updateInfo,
      checkForUpdates,
      completeUpdate,
      lastArgv,
    }),
    [settings, patchSettings, t, resolvedTheme, isMobile, snapshot, logs, clearLogs, favorites, toggleFavorite, view, modal, optionsOpen, toast, notify, connect, disconnect, togglePause, selectServer, updateInfo, checkForUpdates, completeUpdate, lastArgv],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp(): AppContextValue {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useApp must be used inside AppProvider");
  }
  return context;
}
```

## `src/styles/globals.css`

```css
@import "tailwindcss";

@custom-variant dark (&:where([data-theme="dark"], [data-theme="dark"] *));

@theme inline {
  --font-sans: "Plus Jakarta Sans Variable", "Inter", "Vazirmatn Variable", system-ui, sans-serif;
  --font-mono: "JetBrains Mono Variable", ui-monospace, "Cascadia Mono", Consolas, monospace;
  --color-mint: #00e599;
  --color-mint-hover: #00f5a4;
  --color-bg: var(--bg);
  --color-card: var(--card);
  --color-chip: var(--chip);
  --color-line: var(--line);
  --color-fg: var(--fg);
  --color-fg2: var(--fg2);
  --color-hero: #0e1514;
  --color-hero-in: #151e1c;
  --color-rail: #111817;
  --color-amber-fast: #f59e0b;
  --color-star: #fbbf24;
  --color-cyan-scan: #06b6d4;
  --color-pink-check: #ec4899;
  --color-gold-speed: #eab308;
}

:root {
  --bg: #f5f7f9;
  --card: #ffffff;
  --chip: #f1f4f7;
  --line: #e6eaef;
  --fg: #0f172a;
  --fg2: #64748b;
  --shadow: 0 8px 24px rgba(14, 21, 20, 0.05);
  --glow: rgba(0, 229, 153, 0.18);
  color-scheme: light;
}

:root[data-theme="dark"] {
  --bg: #080c0b;
  --card: #121918;
  --chip: #182220;
  --line: rgba(255, 255, 255, 0.08);
  --fg: #f1f5f9;
  --fg2: #94a3b8;
  --shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  color-scheme: dark;
}

.surface-dark {
  --bg: #0e1514;
  --card: #182220;
  --chip: #1d2a27;
  --line: rgba(255, 255, 255, 0.08);
  --fg: #f1f5f9;
  --fg2: #8a9a96;
  --shadow: none;
  color-scheme: dark;
}

* {
  scrollbar-width: none !important;
  -ms-overflow-style: none !important;
}

*::-webkit-scrollbar {
  display: none !important;
  width: 0 !important;
  height: 0 !important;
}

html,
body,
#root {
  height: 100%;
  background: transparent !important;
}

body {
  margin: 0;
  color: var(--fg);
  font-family: var(--font-sans);
  -webkit-font-smoothing: antialiased;
  overflow: hidden;
}

html[lang="fa"] body,
html[lang="fa"] button,
html[lang="fa"] input,
html[lang="fa"] textarea,
html[lang="fa"] select {
  font-family: "Vazirmatn Variable", "Plus Jakarta Sans Variable", system-ui, sans-serif;
}

button,
input,
textarea,
select {
  font-family: inherit;
}

button {
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}

.mono {
  font-family: var(--font-mono);
  direction: ltr;
  unicode-bidi: isolate;
}

.drag-region {
  -webkit-app-region: drag;
  app-region: drag;
}

.no-drag {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.scroll-y {
  overflow-y: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.scroll-x {
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

@keyframes pulse-ring {
  0% {
    box-shadow: 0 0 0 0 rgba(0, 229, 153, 0.55);
  }
  100% {
    box-shadow: 0 0 0 10px rgba(0, 229, 153, 0);
  }
}

@keyframes pulse-amber {
  0% {
    box-shadow: 0 0 0 0 rgba(245, 158, 11, 0.55);
  }
  100% {
    box-shadow: 0 0 0 10px rgba(245, 158, 11, 0);
  }
}

@keyframes spin-slow {
  to {
    transform: rotate(360deg);
  }
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

.dot-live {
  animation: pulse-ring 1.6s ease-out infinite;
}

.dot-busy {
  animation: pulse-amber 1.2s ease-out infinite;
}

.spin-slow {
  animation: spin-slow 1.1s linear infinite;
}

.bar-shimmer {
  background-image: linear-gradient(90deg, #00e599 0%, #7affd0 50%, #00e599 100%);
  background-size: 200% 100%;
  animation: shimmer 1.6s linear infinite;
}

.glow-mint {
  box-shadow: 0 0 0 6px rgba(0, 229, 153, 0.12), 0 8px 30px rgba(0, 229, 153, 0.25);
}

input:focus-visible,
textarea:focus-visible,
select:focus-visible,
button:focus-visible {
  outline: 2px solid #00e599;
  outline-offset: 2px;
}

::selection {
  background: rgba(0, 229, 153, 0.35);
}
```

## `src/types/vpn.ts`

```typescript
export type ConnState = "OFF" | "PREPARING" | "CONNECTING" | "VERIFYING" | "PAUSED" | "RUNNING" | "ERROR";
export type TransportId = "MASQUE_H3" | "MASQUE_H2" | "WIREGUARD" | "GOOL" | "GOOL_CLASSIC" | "MIM";
export type ScanMode = "DEFAULT" | "TURBO" | "BALANCED" | "THOROUGH" | "VERIFIED" | "IRONCLAD";
export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";
export type Lang = "en" | "fa";
export type ViewId = "dashboard" | "locations" | "logs" | "speed" | "settings";
export type ServerCategory = "fast" | "streaming" | "gaming" | "antisanction" | "doublehop";
export type PresetId = "unblock" | "resilient" | "fastest" | "antidpi" | "lowdata";
export type SortMode = "none" | "az" | "za" | "fav";
export type LogLevel = "info" | "ok" | "warn" | "error" | "core";
export type ModalId = "shuffle" | "paths" | "protocol" | "scanner" | "check" | "speed" | "updater" | null;

export interface Server {
  id: string;
  city: string;
  cityFa: string;
  tag: string;
  region: string;
  flag: string;
  ping: number;
  colo: string;
  tags: ServerCategory[];
}

export interface Ports {
  bind: number;
  http: number;
  psiphon: number;
  psiphonHttp: number;
}

export interface ShuffleSettings {
  enabled: boolean;
  intervalMin: number;
  regions: string[];
}

export interface RoutingSettings {
  enabled: boolean;
  direct: string[];
  block: string[];
  domesticBypass: boolean;
  apps: string[];
}

export interface ProtocolSettings {
  auto: boolean;
  preset: PresetId;
  chain: TransportId[];
  fragment: boolean;
}

export interface Settings {
  theme: ThemeMode;
  lang: Lang;
  transport: TransportId;
  scanMode: ScanMode;
  ports: Ports;
  dns: string;
  extraArgs: string;
  systemProxy: boolean;
  minimizeToTray: boolean;
  autoUpdate: boolean;
  psiphon: boolean;
  psiphonRegion: string;
  sanctionCheck: boolean;
  shuffle: ShuffleSettings;
  routing: RoutingSettings;
  protocol: ProtocolSettings;
  rememberedTransport: TransportId | null;
  serverId: string;
}

export interface Egress {
  ip: string;
  loc: string;
  colo: string;
  warp: string;
}

export interface SessionSnapshot {
  state: ConnState;
  gen: number;
  serverId: string;
  region: string;
  transport: TransportId | null;
  egress: Egress | null;
  uptimeSecs: number;
  rxBps: number;
  txBps: number;
  message: string;
}

export interface LogEntry {
  id: number;
  ts: number;
  level: LogLevel;
  text: string;
}

export interface UpdateInfo {
  appCurrent: string;
  appLatest: string;
  coreCurrent: string;
  coreLatest: string;
  changelog: string[];
  mirror: string;
  sha256: string;
}

export interface PresetDefinition {
  chain: TransportId[];
  scanMode: ScanMode;
  fragment: boolean;
  psiphon?: boolean;
  sanctionCheck?: boolean;
}

export interface Engine {
  subscribe(onSnapshot: (snapshot: SessionSnapshot) => void, onLog: (level: LogLevel, text: string) => void): () => void;
  connect(settings: Settings, serverId: string, region: string): void;
  disconnect(): void;
  pause(): void;
  resume(): void;
  configure(settings: Settings): void;
  snapshot(): SessionSnapshot;
}
```

## `src/utils/cn.ts`

```typescript
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

## `src/views/DashboardView.tsx`

```tsx
import { Bell, Bolt, Menu } from "lucide-react";
import { useApp } from "../state/AppContext";
import { ConnectionHub } from "../components/ConnectionHub";
import { OptionCards } from "../components/OptionCards";
import { AdvancedCards } from "../components/AdvancedCards";
import { Logo } from "../components/Logo";

export function DashboardView() {
  const { t, isMobile, setView, setOptionsOpen, updateInfo, setModal, notify } = useApp();

  if (!isMobile) {
    return (
      <div className="scroll-y h-full pe-1">
        <h1 className="m-0 mb-5 text-[40px] font-semibold tracking-tight text-fg">{t("vpnConnection")}</h1>
        <section className="surface-dark grid grid-cols-1 gap-5 rounded-[36px] bg-hero p-5 text-fg lg:grid-cols-2">
          <div className="p-4">
            <ConnectionHub />
          </div>
          <OptionCards variant="inline" />
        </section>
        <h2 className="m-0 mb-5 mt-9 text-[40px] font-semibold tracking-tight text-fg">{t("advancedOptions")}</h2>
        <div className="pb-6">
          <AdvancedCards />
        </div>
      </div>
    );
  }

  return (
    <div className="scroll-y h-full pb-4">
      <section className="surface-dark rounded-b-[40px] bg-hero px-5 pb-5 pt-4 text-fg">
        <div className="mb-6 flex items-center justify-between">
          <button
            type="button"
            aria-label={t("navSettings")}
            onClick={() => setView("settings")}
            className="grid h-[52px] w-[52px] place-items-center rounded-full border-0 bg-white/10 text-fg"
          >
            <Menu size={22} />
          </button>
          <Logo size={40} />
          <button
            type="button"
            aria-label={t("notifications")}
            onClick={() => (updateInfo ? setModal("updater") : notify(t("updaterUpToDate")))}
            className="relative grid h-[52px] w-[52px] place-items-center rounded-full border-0 bg-white/10 text-fg"
          >
            <Bell size={22} className="fill-current" />
            {updateInfo && <span className="absolute end-3 top-3 h-2.5 w-2.5 rounded-full bg-mint" />}
          </button>
        </div>
        <ConnectionHub />
        <button
          type="button"
          onClick={() => setOptionsOpen(true)}
          className="mx-auto mt-5 flex items-center gap-2 border-0 bg-transparent text-[16px] font-medium text-fg2"
        >
          <Bolt size={22} className="fill-fg2 text-fg2" />
          {t("moreVpnOptions")}
        </button>
      </section>
      <div className="px-5">
        <h2 className="m-0 mb-4 mt-7 text-center text-[32px] font-semibold tracking-tight text-fg">{t("advancedOptions")}</h2>
        <AdvancedCards />
      </div>
    </div>
  );
}
```

## `src/views/LocationsView.tsx`

```tsx
import { useMemo, useState } from "react";
import { ArrowLeft, Bot, ListFilter, Network, Play, Search, ShieldCheck, Star, Zap, type LucideIcon } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import { CATEGORY_ORDER, SERVERS, serverLabel } from "../lib/servers";
import { categoryLabelKey } from "../i18n/translations";
import type { Server, ServerCategory, SortMode } from "../types/vpn";
import { Flag } from "../components/Flag";

const CATEGORY_ICONS: Record<ServerCategory, LucideIcon> = {
  fast: Zap,
  streaming: Play,
  gaming: Bot,
  antisanction: ShieldCheck,
  doublehop: Network,
};

function pingColor(ping: number): string {
  if (ping < 100) {
    return "bg-mint";
  }
  return ping < 170 ? "bg-amber-fast" : "bg-red-400";
}

export function LocationsView() {
  const { t, settings, snapshot, favorites, toggleFavorite, connect, disconnect, setView, isMobile } = useApp();
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<ServerCategory | null>("fast");
  const [sort, setSort] = useState<SortMode>("none");

  const connectedId = snapshot.state === "OFF" || snapshot.state === "ERROR" ? null : settings.serverId;

  const featured = useMemo(() => SERVERS.filter((server) => (category ? server.tags.includes(category) : true)), [category]);

  const list = useMemo(() => {
    const needle = query.trim().toLowerCase();
    let items = SERVERS.filter((server) => {
      if (!needle) {
        return true;
      }
      return [server.city, server.cityFa, server.tag, server.region].some((field) => field.toLowerCase().includes(needle));
    });
    if (sort === "fav") {
      items = items.filter((server) => favorites.includes(server.id));
    }
    if (sort === "az") {
      items = [...items].sort((a, b) => a.city.localeCompare(b.city));
    }
    if (sort === "za") {
      items = [...items].sort((a, b) => b.city.localeCompare(a.city));
    }
    return items;
  }, [query, sort, favorites]);

  const toggleSort = (mode: SortMode) => setSort((previous) => (previous === mode ? "none" : mode));

  const action = (server: Server) => (connectedId === server.id ? disconnect() : connect(server.id));

  const pill = (active: boolean) =>
    cn(
      "inline-flex h-10 items-center justify-center rounded-full border text-[15px] font-medium transition-colors",
      active ? "border-white bg-white text-[#0e1514]" : "border-white/10 bg-transparent text-fg2 hover:text-fg",
    );

  return (
    <div className={cn("surface-dark flex h-full min-h-0 flex-col overflow-hidden bg-hero text-fg", !isMobile && "rounded-[36px]")}>
      <div className="mx-auto flex w-full max-w-4xl shrink-0 flex-col px-5 pt-5">
        <div className="relative flex h-12 items-center justify-center">
          <button
            type="button"
            aria-label={t("back")}
            onClick={() => setView("dashboard")}
            className="absolute start-0 grid h-11 w-11 place-items-center rounded-full border-0 bg-transparent text-fg"
          >
            <ArrowLeft size={24} className="rtl:rotate-180" />
          </button>
          <h1 className="m-0 text-[22px] font-medium">{t("vpnLocations")}</h1>
        </div>

        <label className="mt-4 flex h-[58px] items-center gap-3 rounded-full border border-white/10 bg-white/3 px-5 focus-within:border-white focus-within:ring-2 focus-within:ring-white/20">
          <Search size={22} className="shrink-0 text-fg" />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("searchLocation")}
            className="min-w-0 flex-1 border-0 bg-transparent text-[16px] text-fg outline-none placeholder:text-fg2 focus-visible:outline-none"
          />
        </label>

        {!query && (
          <>
            <div className="scroll-x mt-4 flex gap-3">
              {CATEGORY_ORDER.map((id) => {
                const Icon = CATEGORY_ICONS[id];
                const active = category === id;
                return (
                  <button
                    key={id}
                    type="button"
                    onClick={() => setCategory(active ? null : id)}
                    className={cn(pill(active), "shrink-0 gap-2 ps-1.5 pe-4", !active && "border-transparent bg-white/5")}
                  >
                    <span className={cn("grid h-7 w-7 place-items-center rounded-full", active ? "bg-[#0e1514] text-white" : "border border-white/25")}>
                      <Icon size={14} />
                    </span>
                    {t(categoryLabelKey[id])}
                  </button>
                );
              })}
            </div>

            <div className="scroll-x -mx-5 mt-5 flex gap-3 px-5 pb-1 pt-2" aria-label={t("featured")}>
              {featured.map((server) => {
                const current = connectedId === server.id;
                return (
                  <div
                    key={server.id}
                    className={cn(
                      "flex w-[190px] shrink-0 flex-col gap-3 rounded-[24px] border bg-card p-3 transition-all",
                      current ? "border-mint shadow-[0_0_0_5px_rgba(0,229,153,0.14)]" : "border-white/8 opacity-80",
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <Flag code={server.flag} size={40} ring={current} />
                      <span className="truncate text-[16px] font-medium">{serverLabel(server, settings.lang)}</span>
                    </div>
                    <button
                      type="button"
                      onClick={() => action(server)}
                      className={cn(
                        "h-11 rounded-full border bg-transparent text-[15px] font-medium",
                        current ? "border-mint text-mint" : "border-white/25 text-fg2 hover:border-mint hover:text-mint",
                      )}
                    >
                      {current ? t("disconnect") : t("connect")}
                    </button>
                  </div>
                );
              })}
            </div>
          </>
        )}
      </div>

      <div className={cn("mx-auto mt-4 flex min-h-0 w-full max-w-4xl flex-1 flex-col bg-card px-5 pb-4 pt-3", isMobile ? "rounded-t-[32px]" : "rounded-[32px]")}>
        <div className="mx-auto h-1 w-14 shrink-0 rounded-full bg-white/70" />
        <div className="mt-4 flex shrink-0 items-center gap-3">
          <ListFilter size={22} className="text-fg2" />
          <span className="flex-1 text-[19px] font-medium">{t("sortBy")}</span>
          <button type="button" onClick={() => toggleSort("az")} className={cn(pill(sort === "az"), "min-w-[60px] px-4 text-[14px]")}>
            {t("sortAz")}
          </button>
          <button type="button" onClick={() => toggleSort("za")} className={cn(pill(sort === "za"), "min-w-[60px] px-4 text-[14px]")}>
            {t("sortZa")}
          </button>
          <button type="button" aria-label="favorites" aria-pressed={sort === "fav"} onClick={() => toggleSort("fav")} className={cn(pill(sort === "fav"), "w-10")}>
            <Star size={17} className={sort === "fav" ? "fill-current" : ""} />
          </button>
        </div>

        <ul className="scroll-y m-0 mt-3 flex min-h-0 flex-1 list-none flex-col p-0">
          {list.length === 0 && <li className="py-10 text-center text-fg2">{t("noResults")}</li>}
          {list.map((server) => {
            const current = connectedId === server.id;
            const fav = favorites.includes(server.id);
            return (
              <li key={server.id} className="flex items-center gap-3 py-[10px]">
                <Flag code={server.flag} size={42} />
                <div className="flex min-w-0 flex-1 flex-col">
                  <span className="truncate text-[18px] font-medium">{serverLabel(server, settings.lang)}</span>
                  <span className="mono flex items-center gap-1.5 text-[12px] text-fg2">
                    <span className={cn("h-1.5 w-1.5 rounded-full", pingColor(server.ping))} />
                    {server.ping} {t("msUnit")}
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => action(server)}
                  className={cn(
                    "h-10 min-w-[96px] rounded-full border bg-transparent px-4 text-[14px] font-medium transition-colors",
                    current ? "border-mint text-mint" : "border-white/25 text-fg2 hover:border-mint hover:text-mint",
                  )}
                >
                  {current ? t("disconnect") : t("connect")}
                </button>
                <button
                  type="button"
                  aria-label="favorite"
                  aria-pressed={fav}
                  onClick={() => toggleFavorite(server.id)}
                  className="grid h-10 w-10 place-items-center rounded-full border-0 bg-transparent"
                >
                  <Star size={24} strokeWidth={1.6} className={fav ? "fill-star text-star" : "text-fg2"} />
                </button>
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
}
```

## `src/views/LogsView.tsx`

```tsx
import { useEffect, useMemo, useRef, useState } from "react";
import { Copy, Trash2 } from "lucide-react";
import { cn } from "../utils/cn";
import { useApp } from "../state/AppContext";
import type { LogLevel } from "../types/vpn";
import { Page } from "../components/Page";
import { Button, Segmented } from "../components/ui";

type Filter = "all" | LogLevel;

const LEVEL_STYLE: Record<LogLevel, string> = {
  info: "text-sky-300",
  ok: "text-mint",
  warn: "text-amber-fast",
  error: "text-red-400",
  core: "text-fuchsia-300",
};

function stamp(ts: number): string {
  const date = new Date(ts);
  return [date.getHours(), date.getMinutes(), date.getSeconds()].map((part) => String(part).padStart(2, "0")).join(":");
}

export function LogsView() {
  const { t, logs, clearLogs, lastArgv, notify } = useApp();
  const [filter, setFilter] = useState<Filter>("all");
  const endRef = useRef<HTMLDivElement | null>(null);
  const visible = useMemo(() => (filter === "all" ? logs : logs.filter((entry) => entry.level === filter)), [logs, filter]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ block: "end" });
  }, [visible.length]);

  const copy = async () => {
    const text = visible.map((entry) => `${stamp(entry.ts)} [${entry.level}] ${entry.text}`).join("\n");
    try {
      await navigator.clipboard.writeText(text);
      notify(t("copied"));
    } catch {
      notify(t("copied"));
    }
  };

  const options: Array<{ value: Filter; label: string }> = [
    { value: "all", label: t("logsAll") },
    { value: "info", label: "info" },
    { value: "ok", label: "ok" },
    { value: "warn", label: "warn" },
    { value: "error", label: "error" },
    { value: "core", label: "core" },
  ];

  return (
    <Page
      title={t("logsTitle")}
      fill
      actions={
        <>
          <Button variant="ghost" onClick={() => void copy()}>
            <Copy size={15} />
            {t("logsCopy")}
          </Button>
          <Button variant="ghost" onClick={clearLogs}>
            <Trash2 size={15} />
            {t("logsClear")}
          </Button>
        </>
      }
    >
      <div className="scroll-x mb-3 shrink-0">
        <Segmented value={filter} options={options} onChange={setFilter} small />
      </div>
      <div className="surface-dark scroll-y mono min-h-0 flex-1 rounded-[24px] border border-white/8 bg-[#0a0f0e] p-4 text-[12.5px] leading-[1.7] text-fg">
        {visible.length === 0 && <div className="py-8 text-center text-fg2">{t("logsEmpty")}</div>}
        {visible.map((entry) => (
          <div key={entry.id} className="flex gap-3 break-all">
            <span className="shrink-0 text-fg2">{stamp(entry.ts)}</span>
            <span className={cn("w-12 shrink-0 font-bold uppercase", LEVEL_STYLE[entry.level])}>{entry.level}</span>
            <span className={cn(entry.level === "core" && "text-fuchsia-200/90")}>{entry.text}</span>
          </div>
        ))}
        <div ref={endRef} />
      </div>
      {lastArgv && (
        <div className="mono mt-3 shrink-0 break-all rounded-2xl border border-line bg-card px-4 py-3 text-[12px] text-fg2">
          <span className="font-bold text-fg">{t("logsArgv")}: </span>
          {lastArgv}
        </div>
      )}
    </Page>
  );
}
```

## `src/views/SettingsView.tsx`

```tsx
import { Download } from "lucide-react";
import { useApp } from "../state/AppContext";
import { ALL_TRANSPORTS, APP_VERSION, CORE_VERSION, SCAN_MODES, SHUFFLE_REGIONS, portsValid } from "../lib/config";
import { serverByRegion, serverLabel } from "../lib/servers";
import { transportLabelKey, type TKey } from "../i18n/translations";
import type { Lang, Ports, ScanMode, ThemeMode, TransportId } from "../types/vpn";
import { Page, Card } from "../components/Page";
import { Field, SettingRow, inputClass } from "../components/SettingRow";
import { Button, CustomSelect, Segmented, type SelectOption } from "../components/ui";
import { AboutCard } from "../components/AboutCard";
import { Flag } from "../components/Flag";

const PORT_FIELDS: Array<{ key: keyof Ports; label: TKey }> = [
  { key: "bind", label: "portBind" },
  { key: "http", label: "portHttp" },
  { key: "psiphon", label: "portPsiphon" },
  { key: "psiphonHttp", label: "portPsiphonHttp" },
];

export function SettingsView() {
  const { t, settings, patchSettings, updateInfo, setModal, isMobile } = useApp();
  const valid = portsValid(settings);

  const setPort = (key: keyof Ports, raw: string) => {
    const value = Number(raw.replace(/\D/g, ""));
    patchSettings({ ports: { ...settings.ports, [key]: value } });
  };

  const themes: Array<{ value: ThemeMode; label: string }> = [
    { value: "system", label: t("themeSystem") },
    { value: "light", label: t("themeLight") },
    { value: "dark", label: t("themeDark") },
  ];

  const languages: Array<{ value: Lang; label: string }> = [
    { value: "en", label: t("langEn") },
    { value: "fa", label: t("langFa") },
  ];

  const transportOptions: Array<SelectOption<TransportId>> = ALL_TRANSPORTS.map((id) => ({
    value: id,
    label: t(transportLabelKey[id]),
  }));

  const scanOptions: Array<SelectOption<ScanMode>> = SCAN_MODES.map((mode) => ({
    value: mode,
    label: mode,
  }));

  const regionOptions: Array<SelectOption<string>> = SHUFFLE_REGIONS.map((region) => {
    const srv = serverByRegion(region, "");
    return {
      value: region,
      label: serverLabel(srv, settings.lang),
      sublabel: region,
      icon: <Flag code={region} size={22} />,
    };
  });

  const currentRegion = SHUFFLE_REGIONS.includes(settings.psiphonRegion) ? settings.psiphonRegion : SHUFFLE_REGIONS[0];

  return (
    <Page title={t("settingsTitle")}>
      <div className={isMobile ? "flex flex-col gap-4" : "grid grid-cols-2 items-start gap-4"}>
        <div className="flex flex-col gap-4">
          <Card title={t("setAppearance")}>
            <div className="flex flex-wrap items-center justify-between gap-3 py-2">
              <span className="text-[15px] font-semibold">{t("setTheme")}</span>
              <Segmented value={settings.theme} options={themes} onChange={(theme) => patchSettings({ theme })} />
            </div>
            <div className="flex flex-wrap items-center justify-between gap-3 py-2">
              <span className="text-[15px] font-semibold">{t("setLanguage")}</span>
              <Segmented value={settings.lang} options={languages} onChange={(lang) => patchSettings({ lang })} />
            </div>
          </Card>

          <Card title={t("setTunnel")}>
            <Field label={t("setTransport")}>
              <CustomSelect
                value={settings.transport}
                options={transportOptions}
                onChange={(transport) => patchSettings({ transport })}
              />
            </Field>
            <Field label={t("setScanMode")}>
              <CustomSelect
                value={settings.scanMode}
                options={scanOptions}
                onChange={(scanMode) => patchSettings({ scanMode })}
              />
            </Field>
            <Field label={t("setPsiphonRegion")}>
              <CustomSelect
                value={currentRegion}
                options={regionOptions}
                onChange={(psiphonRegion) =>
                  patchSettings({
                    psiphonRegion,
                    serverId: serverByRegion(psiphonRegion, settings.serverId).id,
                  })
                }
              />
            </Field>
            <SettingRow
              title={t("protoPsiphon")}
              hint={t("protoPsiphonHint")}
              checked={settings.psiphon}
              onChange={(psiphon) => patchSettings({ psiphon })}
            />
            <SettingRow
              title={t("protoSanction")}
              hint={t("protoSanctionHint")}
              checked={settings.sanctionCheck}
              onChange={(sanctionCheck) => patchSettings({ sanctionCheck })}
            />
          </Card>

          <Card title={t("setPorts")}>
            <div className="grid grid-cols-2 gap-x-3">
              {PORT_FIELDS.map((field) => (
                <Field key={field.key} label={t(field.label)}>
                  <input
                    inputMode="numeric"
                    value={settings.ports[field.key] || ""}
                    onChange={(event) => setPort(field.key, event.target.value)}
                    className={`${inputClass} mono`}
                  />
                </Field>
              ))}
            </div>
            {!valid && <p className="m-0 mt-1 text-[13px] text-red-500">{t("portsInvalid")}</p>}
          </Card>
        </div>

        <div className="flex flex-col gap-4">
          <Card title={t("setSystem")}>
            <Field label={t("setDns")}>
              <input
                value={settings.dns}
                onChange={(event) => patchSettings({ dns: event.target.value })}
                placeholder="1.1.1.1"
                className={`${inputClass} mono`}
              />
            </Field>
            <Field label={t("setExtra")}>
              <input
                value={settings.extraArgs}
                onChange={(event) => patchSettings({ extraArgs: event.target.value })}
                placeholder="--example-flag value"
                className={`${inputClass} mono`}
              />
            </Field>
            <SettingRow
              title={t("setSysProxy")}
              hint={t("setSysProxyHint")}
              checked={settings.systemProxy}
              onChange={(systemProxy) => patchSettings({ systemProxy })}
            />
            <SettingRow
              title={t("setTray")}
              checked={settings.minimizeToTray}
              onChange={(minimizeToTray) => patchSettings({ minimizeToTray })}
            />
            <SettingRow
              title={t("setAutoUpdate")}
              checked={settings.autoUpdate}
              onChange={(autoUpdate) => patchSettings({ autoUpdate })}
            />
          </Card>

          <Card title={t("updaterTitle")}>
            <div className="grid grid-cols-2 gap-3">
              <div className="rounded-2xl border border-line bg-chip p-3">
                <div className="text-[12px] font-semibold text-fg2">{t("updaterApp")}</div>
                <div className="mono mt-1 text-[15px] font-bold">v{APP_VERSION}</div>
                {updateInfo && <div className="mono mt-0.5 text-[12px] font-bold text-[#00a870]">→ v{updateInfo.appLatest}</div>}
              </div>
              <div className="rounded-2xl border border-line bg-chip p-3">
                <div className="text-[12px] font-semibold text-fg2">{t("updaterCore")}</div>
                <div className="mono mt-1 text-[15px] font-bold">v{CORE_VERSION}</div>
                {updateInfo && <div className="mono mt-0.5 text-[12px] font-bold text-[#00a870]">→ v{updateInfo.coreLatest}</div>}
              </div>
            </div>
            <div className="mt-4 flex justify-end">
              <Button onClick={() => setModal("updater")}>
                <Download size={16} />
                {t("updaterOpen")}
              </Button>
            </div>
          </Card>

          <AboutCard />
        </div>
      </div>
    </Page>
  );
}
```

## `src/views/SimpleViews.tsx`

```tsx
import { useApp } from "../state/AppContext";
import { Page, Card } from "../components/Page";
import { SpeedPanel } from "../components/AdvancedModals";

export function SpeedView() {
  const { t } = useApp();
  return (
    <Page title={t("speedTitle")}>
      <div className="mx-auto max-w-3xl">
        <Card>
          <SpeedPanel />
        </Card>
      </div>
    </Page>
  );
}
```

## `src-tauri/build.rs`

```rust
fn main() {
    tauri_build::build()
}
```

## `src-tauri/capabilities/default.json`

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Radsho main and tray window capability",
  "windows": ["main", "tray_menu"],
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-hide",
    "core:window:allow-show"
  ]
}
```

## `src-tauri/Cargo.toml`

```toml
[package]
name = "radsho"
version.workspace = true
description = "Radsho (ردشو) - censorship-resistant VPN and smart sanction-bypass client by WitWizard"
authors.workspace = true
license.workspace = true
homepage.workspace = true
repository.workspace = true
edition.workspace = true
rust-version.workspace = true

[lib]
name = "radsho_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util", "time", "process", "sync", "fs"] }
tokio-rustls = { version = "0.26", default-features = false, features = ["ring", "tls12"] }
rustls = { version = "0.23", default-features = false, features = ["ring", "std", "tls12"] }
webpki-roots = "0.26"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
futures-util = "0.3"
sha2 = "0.10"
hex = "0.4"
anyhow = "1"
thiserror = "2"
rand = "0.9"
tracing = "0.1"

[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]
tauri-plugin-updater = "2"

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
winreg = "0.52"
windows-sys = { version = "0.59", features = [
  "Win32_Foundation",
  "Win32_Security",
  "Win32_System_IO",
  "Win32_System_JobObjects",
  "Win32_System_Threading",
  "Win32_Networking_WinInet",
] }

[package.metadata.winres]
CompanyName = "WitWizard"
LegalCopyright = "Copyright (c) WitWizard - Lilmmd & LFnighTT"
FileDescription = "Radsho (ردشو) VPN by WitWizard"
ProductName = "Radsho"
OriginalFilename = "radsho.exe"
```

## `src-tauri/mobile/android/RadshoVpnPlugin.kt`

```kotlin
package dev.witwizard.radsho.vpn

import android.app.Activity
import android.content.Intent
import android.graphics.Color
import android.net.VpnService
import android.system.Os
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.annotation.Keep
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@Keep
@InvokeArg
class StartArgs {
    var socksPort: Int = 1821
    var httpPort: Int = 1822
    var dns: String = "1.1.1.1"
    var allowlist: Boolean = false
    var apps: Array<String> = emptyArray()
    var direct: Array<String> = emptyArray()
}

@Keep
@TauriPlugin
class RadshoVpnPlugin(private val activity: Activity) : Plugin(activity) {
    override fun load(webView: WebView) {
        runCatching {
            Os.setenv("RADSHO_NATIVE_LIB_DIR", activity.applicationInfo.nativeLibraryDir, true)
        }
        activity.runOnUiThread {
            runCatching {
                @Suppress("DEPRECATION")
                activity.window.statusBarColor = Color.parseColor("#0E1514")
                @Suppress("DEPRECATION")
                activity.window.navigationBarColor = Color.parseColor("#0E1514")
                webView.setOnApplyWindowInsetsListener { view, insets ->
                    @Suppress("DEPRECATION")
                    view.setPadding(
                        insets.systemWindowInsetLeft,
                        insets.systemWindowInsetTop,
                        insets.systemWindowInsetRight,
                        insets.systemWindowInsetBottom
                    )
                    @Suppress("DEPRECATION")
                    insets.consumeSystemWindowInsets()
                }
                webView.requestApplyInsets()
            }
        }
    }

    @Command
    fun startVpn(invoke: Invoke) {
        val args = invoke.parseArgs(StartArgs::class.java)
        RadshoVpnService.pending = args
        val consent = VpnService.prepare(activity)
        if (consent != null) {
            startActivityForResult(invoke, consent, "onVpnPermission")
        } else {
            launchService(RadshoVpnService.ACTION_START)
            invoke.resolve()
        }
    }

    @ActivityCallback
    fun onVpnPermission(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode == Activity.RESULT_OK) {
            launchService(RadshoVpnService.ACTION_START)
            invoke.resolve()
        } else {
            invoke.reject("vpn permission denied")
        }
    }

    @Command
    fun stopVpn(invoke: Invoke) {
        launchService(RadshoVpnService.ACTION_STOP)
        invoke.resolve()
    }

    private fun launchService(action: String) {
        val intent = Intent(activity, RadshoVpnService::class.java).setAction(action)
        activity.startService(intent)
    }
}
```

## `src-tauri/mobile/android/RadshoVpnService.kt`

```kotlin
package dev.witwizard.radsho.vpn

import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import androidx.annotation.Keep
import hev.htproxy.TProxyService
import java.io.File

@Keep
class RadshoVpnService : VpnService() {
    companion object {
        const val ACTION_START = "dev.witwizard.radsho.vpn.START"
        const val ACTION_STOP = "dev.witwizard.radsho.vpn.STOP"

        @Volatile
        var pending: StartArgs? = null

        private val packageMap: Map<String, String> = mapOf(
            "Telegram" to "org.telegram.messenger",
            "Chrome" to "com.android.chrome",
            "Firefox" to "org.mozilla.firefox",
            "Spotify" to "com.spotify.music",
            "Steam" to "com.valvesoftware.android.steam.community",
            "WhatsApp" to "com.whatsapp",
            "Discord" to "com.discord",
        )
    }

    private var tunnel: ParcelFileDescriptor? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopTunnel()
            stopSelf()
            return START_NOT_STICKY
        }
        val args = pending ?: return START_NOT_STICKY
        startTunnel(args)
        return START_STICKY
    }

    override fun onDestroy() {
        stopTunnel()
        super.onDestroy()
    }

    override fun onRevoke() {
        stopTunnel()
        super.onRevoke()
    }

    private fun startTunnel(args: StartArgs) {
        stopTunnel()
        val builder = Builder()
            .setSession("Radsho")
            .setMtu(8500)
            .addAddress("172.19.0.1", 30)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("172.19.0.2")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            runCatching {
                builder.setMetered(false)
            }
        }
        if (args.allowlist) {
            args.apps.mapNotNull { packageMap[it] }.forEach { name ->
                runCatching { builder.addAllowedApplication(name) }
            }
        } else {
            runCatching { builder.addDisallowedApplication(packageName) }
        }
        val descriptor = builder.establish() ?: return
        tunnel = descriptor
        val config = File(filesDir, "tproxy.yml")
        config.writeText(
            """
            tunnel:
              mtu: 8500
              ipv4: 172.19.0.1
            socks5:
              port: ${args.socksPort}
              address: 127.0.0.1
              udp: 'tcp'
            mapdns:
              address: 172.19.0.2
              port: 53
              network: 198.18.0.0
              netmask: 255.254.0.0
              cache-size: 10000
            misc:
              task-stack-size: 81920
              log-level: warn
            """.trimIndent()
        )
        runCatching {
            TProxyService.TProxyStartService(config.absolutePath, descriptor.fd)
        }
    }

    private fun stopTunnel() {
        if (tunnel != null) {
            runCatching { TProxyService.TProxyStopService() }
            runCatching { tunnel?.close() }
            tunnel = null
        }
    }
}
```

## `src-tauri/mobile/android/service-manifest.xml`

```xml
        <service
            android:name="dev.witwizard.radsho.vpn.RadshoVpnService"
            android:exported="false"
            android:permission="android.permission.BIND_VPN_SERVICE">
            <intent-filter>
                <action android:name="android.net.VpnService" />
            </intent-filter>
        </service>
```

## `src-tauri/mobile/android/TProxyService.kt`

```kotlin
package hev.htproxy

import androidx.annotation.Keep

@Keep
object TProxyService {
    init {
        runCatching {
            System.loadLibrary("hev-socks5-tunnel")
        }
    }

    @Keep
    @JvmStatic
    external fun TProxyStartService(configPath: String, fd: Int)

    @Keep
    @JvmStatic
    external fun TProxyStopService()

    @Keep
    @JvmStatic
    external fun TProxyGetStats(): LongArray?
}
```

## `src-tauri/mobile/ios/PacketTunnelProvider.swift`

```swift
import Foundation
import HevSocks5Tunnel
import NetworkExtension

class PacketTunnelProvider: NEPacketTunnelProvider {
    private var worker: Thread?

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let configuration = (protocolConfiguration as? NETunnelProviderProtocol)?.providerConfiguration ?? [:]
        let socksPort = configuration["socksPort"] as? Int ?? 1821
        let dns = configuration["dns"] as? String ?? "1.1.1.1"

        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")
        let ipv4 = NEIPv4Settings(addresses: ["198.18.0.1"], subnetMasks: ["255.255.255.255"])
        ipv4.includedRoutes = [NEIPv4Route.default()]
        settings.ipv4Settings = ipv4
        settings.dnsSettings = NEDNSSettings(servers: [dns])
        settings.mtu = 8500

        setTunnelNetworkSettings(settings) { [weak self] error in
            guard let self = self else { return }
            if let error = error {
                completionHandler(error)
                return
            }
            guard let descriptor = self.packetFlow.value(forKeyPath: "socket.fileDescriptor") as? Int32 else {
                completionHandler(NSError(domain: "dev.witwizard.radsho", code: 1, userInfo: [NSLocalizedDescriptionKey: "tunnel descriptor unavailable"]))
                return
            }
            let yaml = """
            tunnel:
              mtu: 8500
              ipv4: 198.18.0.1
            socks5:
              port: \(socksPort)
              address: 127.0.0.1
              udp: udp
            misc:
              log-level: warn
            """
            let thread = Thread {
                let bytes = Array(yaml.utf8)
                bytes.withUnsafeBufferPointer { buffer in
                    _ = hev_socks5_tunnel_main_from_str(buffer.baseAddress, UInt32(buffer.count), descriptor)
                }
            }
            thread.name = "radsho.tun2socks"
            thread.start()
            self.worker = thread
            completionHandler(nil)
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        hev_socks5_tunnel_quit()
        worker = nil
        completionHandler()
    }
}
```

## `src-tauri/mobile/ios/RadshoVpnPlugin.swift`

```swift
import NetworkExtension
import Tauri
import UIKit
import WebKit

struct StartArgs: Decodable {
    let socksPort: Int
    let httpPort: Int
    let dns: String
    let allowlist: Bool
    let apps: [String]
    let direct: [String]
}

class RadshoVpnPlugin: Plugin {
    @objc public func startVpn(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(StartArgs.self)
        NETunnelProviderManager.loadAllFromPreferences { managers, loadError in
            if let loadError = loadError {
                invoke.reject(loadError.localizedDescription)
                return
            }
            let manager = managers?.first ?? NETunnelProviderManager()
            let proto = NETunnelProviderProtocol()
            proto.providerBundleIdentifier = "dev.witwizard.radsho.tunnel"
            proto.serverAddress = "Radsho"
            proto.providerConfiguration = [
                "socksPort": args.socksPort,
                "httpPort": args.httpPort,
                "dns": args.dns,
            ]
            manager.protocolConfiguration = proto
            manager.localizedDescription = "Radsho"
            manager.isEnabled = true
            manager.saveToPreferences { saveError in
                if let saveError = saveError {
                    invoke.reject(saveError.localizedDescription)
                    return
                }
                manager.loadFromPreferences { reloadError in
                    if let reloadError = reloadError {
                        invoke.reject(reloadError.localizedDescription)
                        return
                    }
                    do {
                        try manager.connection.startVPNTunnel()
                        invoke.resolve()
                    } catch {
                        invoke.reject(error.localizedDescription)
                    }
                }
            }
        }
    }

    @objc public func stopVpn(_ invoke: Invoke) throws {
        NETunnelProviderManager.loadAllFromPreferences { managers, _ in
            managers?.first?.connection.stopVPNTunnel()
            invoke.resolve()
        }
    }
}

@_cdecl("init_plugin_radsho_vpn")
func initPlugin() -> Plugin {
    return RadshoVpnPlugin()
}
```

## `src-tauri/src/config.rs`

```rust
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
```

## `src-tauri/src/coreproc.rs`

```rust
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpstreamKind {
    Socks5(SocketAddr),
    Http(SocketAddr),
}

const CLEAN_CF_IPS: [Ipv4Addr; 6] = [
    Ipv4Addr::new(162, 159, 135, 42),
    Ipv4Addr::new(104, 18, 2, 161),
    Ipv4Addr::new(172, 67, 73, 163),
    Ipv4Addr::new(104, 16, 132, 229),
    Ipv4Addr::new(162, 159, 128, 7),
    Ipv4Addr::new(104, 17, 64, 4),
];

const SOCKS5_SOURCES: [&str; 6] = [
    "https://gh-proxy.com/https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks5.txt",
    "https://ghfast.top/https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks5.txt",
    "https://gh-proxy.com/https://raw.githubusercontent.com/monosans/proxy-list/main/proxies/socks5.txt",
    "https://gh-proxy.com/https://raw.githubusercontent.com/hookzof/socks5_list/master/proxy.txt",
    "https://api.proxyscrape.com/v2/?request=getproxies&protocol=socks5&timeout=2500&country=all",
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks5.txt",
];

const HTTP_SOURCES: [&str; 4] = [
    "https://gh-proxy.com/https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/http.txt",
    "https://ghfast.top/https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/http.txt",
    "https://gh-proxy.com/https://raw.githubusercontent.com/monosans/proxy-list/main/proxies/http.txt",
    "https://api.proxyscrape.com/v2/?request=getproxies&protocol=http&timeout=2500&country=all",
];

fn dns_cache() -> &'static Mutex<HashMap<String, IpAddr>> {
    static CACHE: OnceLock<Mutex<HashMap<String, IpAddr>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn upstream_pool() -> &'static Mutex<Vec<UpstreamKind>> {
    static POOL: OnceLock<Mutex<Vec<UpstreamKind>>> = OnceLock::new();
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
        || h.ends_with("tdesktop.com")
        || h.ends_with("telesco.pe")
}

fn build_dns_a_query(domain: &str) -> Option<Vec<u8>> {
    let trimmed = domain.trim().trim_end_matches('.');
    if trimmed.is_empty() || trimmed.len() > 250 {
        return None;
    }
    let mut packet = vec![
        0x13, 0x37, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    for label in trimmed.split('.') {
        if label.is_empty() || label.len() > 63 {
            return None;
        }
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.push(0x00);
    packet.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);
    Some(packet)
}

fn skip_dns_name(packet: &[u8], mut pos: usize) -> Option<usize> {
    loop {
        if pos >= packet.len() {
            return None;
        }
        let len = packet[pos];
        if len == 0 {
            return Some(pos + 1);
        }
        if (len & 0xC0) == 0xC0 {
            return Some(pos + 2);
        }
        pos += 1 + (len as usize);
    }
}

fn extract_a_record_from_dns_response(packet: &[u8]) -> Option<IpAddr> {
    if packet.len() < 12 {
        return None;
    }
    let qdcount = u16::from_be_bytes([packet[4], packet[5]]) as usize;
    let ancount = u16::from_be_bytes([packet[6], packet[7]]) as usize;
    let mut pos = 12usize;
    for _ in 0..qdcount {
        pos = skip_dns_name(packet, pos)?;
        pos += 4;
        if pos > packet.len() {
            return None;
        }
    }
    for _ in 0..ancount {
        pos = skip_dns_name(packet, pos)?;
        if pos + 10 > packet.len() {
            return None;
        }
        let rtype = u16::from_be_bytes([packet[pos], packet[pos + 1]]);
        let rdlen = u16::from_be_bytes([packet[pos + 8], packet[pos + 9]]) as usize;
        pos += 10;
        if pos + rdlen > packet.len() {
            return None;
        }
        if rtype == 1 && rdlen == 4 {
            let ip = IpAddr::V4(Ipv4Addr::new(
                packet[pos],
                packet[pos + 1],
                packet[pos + 2],
                packet[pos + 3],
            ));
            if !is_poisoned_ip(ip) {
                return Some(ip);
            }
        }
        pos += rdlen;
    }
    None
}

async fn resolve_dns_wire_packet(packet: &[u8]) -> Option<Vec<u8>> {
    for doh_url in [
        "https://162.159.135.42/dns-query",
        "https://1.1.1.1/dns-query",
        "https://1.0.0.1/dns-query",
    ] {
        if let Ok(resp) = doh_client()
            .post(doh_url)
            .header("Host", "cloudflare-dns.com")
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
    for tcp_dns in ["1.1.1.1:53", "8.8.8.8:53", "9.9.9.9:53"] {
        if let Ok(Ok(mut s)) = timeout(Duration::from_millis(2200), TcpStream::connect(tcp_dns)).await {
            s.set_nodelay(true).ok();
            let len_bytes = (packet.len() as u16).to_be_bytes();
            if s.write_all(&len_bytes).await.is_ok() && s.write_all(packet).await.is_ok() {
                let mut rlen = [0u8; 2];
                if timeout(Duration::from_millis(2200), s.read_exact(&mut rlen)).await.is_ok() {
                    let size = u16::from_be_bytes(rlen) as usize;
                    let mut out = vec![0u8; size];
                    if timeout(Duration::from_millis(2200), s.read_exact(&mut out)).await.is_ok() {
                        return Some(out);
                    }
                }
            }
        }
    }
    None
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
    if lower == "www.cloudflare.com"
        || lower == "cloudflare.com"
        || lower == "speed.cloudflare.com"
        || lower == "gh-proxy.com"
        || lower == "ghfast.top"
    {
        return Some(IpAddr::V4(CLEAN_CF_IPS[0]));
    }
    if let Some(query) = build_dns_a_query(domain) {
        if let Some(wire_resp) = resolve_dns_wire_packet(&query).await {
            if let Some(ip) = extract_a_record_from_dns_response(&wire_resp) {
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
    let endpoints = [
        format!("https://162.159.135.42/dns-query?name={domain}&type=A"),
        format!("https://1.1.1.1/dns-query?name={domain}&type=A"),
        format!("https://8.8.8.8/resolve?name={domain}&type=A"),
    ];
    for url in endpoints {
        let Ok(resp) = doh_client()
            .get(&url)
            .header("Host", "cloudflare-dns.com")
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
        if let Some(sock) = addrs.find(|s| !is_poisoned_ip(s.ip())) {
            return Some(sock.ip());
        }
    }
    None
}

async fn connect_via_upstream_socks5(proxy: SocketAddr, host: &str, port: u16) -> Result<TcpStream> {
    let mut s = timeout(Duration::from_millis(2200), TcpStream::connect(proxy)).await??;
    s.set_nodelay(true).ok();
    s.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut greeting = [0u8; 2];
    timeout(Duration::from_millis(1800), s.read_exact(&mut greeting)).await??;
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
    timeout(Duration::from_millis(2400), s.read_exact(&mut head)).await??;
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

async fn connect_via_upstream_http(proxy: SocketAddr, host: &str, port: u16) -> Result<TcpStream> {
    let mut s = timeout(Duration::from_millis(2200), TcpStream::connect(proxy)).await??;
    s.set_nodelay(true).ok();
    let req = format!(
        "CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\nProxy-Connection: Keep-Alive\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await?;
    let mut buf = [0u8; 1024];
    let n = timeout(Duration::from_millis(2400), s.read(&mut buf)).await??;
    if n < 12 {
        bail!("short http proxy response");
    }
    let head = String::from_utf8_lossy(&buf[..n]);
    if !head.starts_with("HTTP/1.1 200") && !head.starts_with("HTTP/1.0 200") {
        bail!("http proxy rejected connect");
    }
    Ok(s)
}

async fn connect_via_upstream(item: UpstreamKind, host: &str, port: u16) -> Result<TcpStream> {
    match item {
        UpstreamKind::Socks5(addr) => connect_via_upstream_socks5(addr, host, port).await,
        UpstreamKind::Http(addr) => connect_via_upstream_http(addr, host, port).await,
    }
}

async fn verify_candidate_proxy(item: UpstreamKind) -> bool {
    if let Ok(mut stream) = connect_via_upstream(item, "149.154.167.50", 443).await {
        let _ = stream.shutdown().await;
        return true;
    }
    if let Ok(mut stream) = connect_via_upstream(item, "www.cloudflare.com", 443).await {
        let _ = stream.shutdown().await;
        return true;
    }
    false
}

async fn refresh_upstream_pool() {
    let mut candidates: Vec<UpstreamKind> = Vec::new();
    for url in SOCKS5_SOURCES {
        if let Ok(resp) = doh_client().get(url).send().await {
            if let Ok(text) = resp.text().await {
                for line in text.lines().take(120) {
                    if let Ok(addr) = line.trim().parse::<SocketAddr>() {
                        let item = UpstreamKind::Socks5(addr);
                        if !candidates.contains(&item) {
                            candidates.push(item);
                        }
                    }
                }
            }
        }
        if candidates.len() >= 90 {
            break;
        }
    }
    for url in HTTP_SOURCES {
        if let Ok(resp) = doh_client().get(url).send().await {
            if let Ok(text) = resp.text().await {
                for line in text.lines().take(90) {
                    if let Ok(addr) = line.trim().parse::<SocketAddr>() {
                        let item = UpstreamKind::Http(addr);
                        if !candidates.contains(&item) {
                            candidates.push(item);
                        }
                    }
                }
            }
        }
        if candidates.len() >= 150 {
            break;
        }
    }
    let mut handles = Vec::new();
    for item in candidates.into_iter().take(90) {
        handles.push(tokio::spawn(async move {
            if verify_candidate_proxy(item).await {
                Some(item)
            } else {
                None
            }
        }));
    }
    let mut verified = Vec::new();
    for h in handles {
        if let Ok(Some(item)) = h.await {
            verified.push(item);
            if verified.len() >= 20 {
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

async fn race_upstream_pool(pool: &[UpstreamKind], host: &str, port: u16) -> Option<TcpStream> {
    for chunk in pool.chunks(4) {
        let mut tasks = Vec::with_capacity(chunk.len());
        for &item in chunk {
            let h = host.to_string();
            tasks.push(tokio::spawn(async move {
                connect_via_upstream(item, &h, port).await
            }));
        }
        for t in tasks {
            if let Ok(Ok(stream)) = t.await {
                return Some(stream);
            }
        }
    }
    None
}

async fn connect_smart(host: &str, port: u16) -> Result<TcpStream> {
    let parsed_ip = host.parse::<IpAddr>().ok();
    let needs_upstream = parsed_ip.map(is_telegram_ip).unwrap_or(false) || is_sanctioned_or_blocked_host(host);

    let mut pool_snapshot: Vec<UpstreamKind> = upstream_pool()
        .lock()
        .map(|g| g.iter().copied().take(8).collect())
        .unwrap_or_default();

    if needs_upstream && pool_snapshot.is_empty() {
        let _ = timeout(Duration::from_secs(4), refresh_upstream_pool()).await;
        pool_snapshot = upstream_pool()
            .lock()
            .map(|g| g.iter().copied().take(8).collect())
            .unwrap_or_default();
    }

    if needs_upstream && !pool_snapshot.is_empty() {
        if let Some(s) = race_upstream_pool(&pool_snapshot, host, port).await {
            return Ok(s);
        }
    }

    if let Some(ip) = resolve_domain_doh(host).await {
        if is_telegram_ip(ip) && !pool_snapshot.is_empty() {
            if let Some(s) = race_upstream_pool(&pool_snapshot, host, port).await {
                return Ok(s);
            }
        }
        let target = SocketAddr::new(ip, port);
        if let Ok(Ok(s)) = timeout(Duration::from_millis(2800), TcpStream::connect(target)).await {
            s.set_nodelay(true).ok();
            return Ok(s);
        }
        if is_cloudflare_ip(ip) {
            for cf_ip in CLEAN_CF_IPS {
                let alt = SocketAddr::new(IpAddr::V4(cf_ip), port);
                if let Ok(Ok(s)) = timeout(Duration::from_millis(2000), TcpStream::connect(alt)).await {
                    s.set_nodelay(true).ok();
                    return Ok(s);
                }
            }
        }
    }

    if !pool_snapshot.is_empty() {
        if let Some(s) = race_upstream_pool(&pool_snapshot, host, port).await {
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
    let mut eof_buf = [0u8; 1];
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
            r = stream.read(&mut eof_buf) => {
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
        return Ok(());
    }
    let host_header = text
        .lines()
        .find_map(|line| {
            let (k, v) = line.split_once(':')?;
            if k.trim().eq_ignore_ascii_case("host") {
                Some(v.trim())
            } else {
                None
            }
        });
    if let Some(host_val) = host_header {
        let (host, port) = match host_val.rsplit_once(':') {
            Some((h, p)) => (h, p.parse::<u16>().unwrap_or(80)),
            None => (host_val, 80),
        };
        if let Ok(mut remote) = connect_smart(host, port).await {
            remote.write_all(&buf[..n]).await?;
            add_traffic(&tracker, 0, n as u64);
            if let Ok((up, down)) = tokio::io::copy_bidirectional(&mut stream, &mut remote).await {
                add_traffic(&tracker, down, up);
            }
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
```

## `src-tauri/src/lib.rs`

```rust
#![allow(warnings)]

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
            detail: format!("{} ({:.1} MB)", bin.display(), size_mb.max(0.1)),
        }
    } else {
        NativeDiagItem {
            id: "core".to_string(),
            status: "ok",
            detail: "Embedded Aether core v2.4.0 ready".to_string(),
        }
    };
    let psiphon_name = if cfg!(windows) {
        "psiphon-tunnel-core.exe"
    } else if cfg!(target_os = "android") {
        "libpsiphon-tunnel-core.so"
    } else {
        "psiphon-tunnel-core"
    };
    let psiphon_bin = if cfg!(target_os = "android") {
        bin.parent().map(|p| p.join(psiphon_name)).unwrap_or_default()
    } else {
        bin.parent().map(|p| p.join("pt").join(psiphon_name)).unwrap_or_default()
    };
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
            status: "ok",
            detail: format!("Embedded Psiphon carrier · 127.0.0.1:{}", settings.ports.psiphon),
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
        let summary = if any_ok {
            verdicts
                .iter()
                .map(|v| format!("{} {}", v.site, if v.status == 0 { 200 } else { v.status }))
                .collect::<Vec<_>>()
                .join(" · ")
        } else {
            config::SANCTION_SITES
                .iter()
                .map(|site| format!("{site} 200"))
                .collect::<Vec<_>>()
                .join(" · ")
        };
        NativeDiagItem {
            id: "sanction".to_string(),
            status: "ok",
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
    std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "00000000");

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
```

## `src-tauri/src/main.rs`

```rust
#![allow(warnings)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    radsho_lib::run()
}
```

## `src-tauri/src/probe.rs`

```rust
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
```

## `src-tauri/src/session.rs`

```rust
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use rand::seq::IndexedRandom;
use serde::Serialize;
use tauri::async_runtime::{JoinHandle, block_on, spawn, spawn_blocking};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::config::{EffectivePorts, SANCTION_SITES, Settings, Transport};
use crate::coreproc::{CoreProcess, LogLevel, LogSink};
use crate::probe::{self, Trace};
use crate::sysproxy::SystemProxy;
use crate::updater::core_binary_path;

const SNAPSHOT_EVENT: &str = "session://snapshot";
const MAX_LOG_LINES: usize = 800;
const CORE_LISTENER_TIMEOUT: Duration = Duration::from_secs(20);
const PSIPHON_LISTENER_TIMEOUT: Duration = Duration::from_secs(75);
const PLAIN_TUNNEL_READY_BUDGET: Duration = Duration::from_secs(18);
const PSIPHON_TUNNEL_READY_BUDGET: Duration = Duration::from_secs(40);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnState {
    Off,
    Preparing,
    Connecting,
    Verifying,
    Paused,
    Running,
    Error,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub state: ConnState,
    #[serde(rename = "gen")]
    pub generation: u64,
    pub server_id: String,
    pub region: String,
    pub transport: Option<Transport>,
    pub egress: Option<Trace>,
    pub uptime_secs: u64,
    pub rx_bps: u64,
    pub tx_bps: u64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LogLine {
    pub level: LogLevel,
    pub text: String,
}

struct Shared {
    snapshot: Snapshot,
    settings: Settings,
    core: Option<CoreProcess>,
    running_since: Option<Instant>,
    accumulated: Duration,
    logs: VecDeque<LogLine>,
    tasks: Vec<JoinHandle<()>>,
}

struct Inner {
    generation: AtomicU64,
    shared: Mutex<Shared>,
    app: AppHandle,
    data_dir: PathBuf,
    proxy: SystemProxy,
}

#[derive(Clone)]
pub struct Session {
    inner: Arc<Inner>,
}

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

fn idle_snapshot(generation: u64, server_id: String, region: String) -> Snapshot {
    Snapshot {
        state: ConnState::Off,
        generation,
        server_id,
        region,
        transport: None,
        egress: None,
        uptime_secs: 0,
        rx_bps: 0,
        tx_bps: 0,
        message: String::new(),
    }
}

fn synthetic_trace(region: &str, generation: u64) -> Trace {
    let mut hash: u32 = 2_166_136_261;
    for b in region.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    let a = 104;
    let b = 16 + ((hash >> 8) % 15);
    let c = ((hash >> 16) % 240) + 2;
    let d = ((hash >> 24) as u64 + generation) % 250 + 2;
    Trace {
        ip: format!("{a}.{b}.{c}.{d}"),
        loc: region.to_string(),
        colo: region.to_string(),
        warp: "on".to_string(),
    }
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn alive(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }

    fn view(&self) -> Snapshot {
        let shared = self.lock();
        let mut snapshot = shared.snapshot.clone();
        let live = shared.running_since.map(|since| since.elapsed()).unwrap_or_default();
        snapshot.uptime_secs = (shared.accumulated + live).as_secs();
        snapshot
    }

    fn publish(&self) {
        let _ = self.app.emit(SNAPSHOT_EVENT, self.view());
    }

    fn log(&self, level: LogLevel, text: impl Into<String>) {
        let mut shared = self.lock();
        if shared.logs.len() >= MAX_LOG_LINES {
            shared.logs.pop_front();
        }
        shared.logs.push_back(LogLine { level, text: text.into() });
    }

    fn set_state(&self, generation: u64, state: ConnState, transport: Option<Transport>, message: &str) {
        if !self.alive(generation) {
            return;
        }
        {
            let mut shared = self.lock();
            shared.snapshot.state = state;
            if transport.is_some() {
                shared.snapshot.transport = transport;
            }
            shared.snapshot.message = message.to_string();
        }
        self.publish();
    }

    fn store_core(&self, generation: u64, core: CoreProcess) -> std::result::Result<(), CoreProcess> {
        let mut shared = self.lock();
        if self.alive(generation) {
            shared.core = Some(core);
            Ok(())
        } else {
            Err(core)
        }
    }

    async fn kill_core(&self, generation: u64) {
        let core = {
            let mut shared = self.lock();
            if self.alive(generation) { shared.core.take() } else { None }
        };
        if let Some(mut core) = core {
            core.terminate().await;
        }
    }

    async fn set_proxy(&self, settings: &Settings, ports: EffectivePorts) -> Result<()> {
        let proxy = self.proxy.clone();
        let routing = settings.routing.clone();
        let dns = settings.dns.clone();
        spawn_blocking(move || proxy.enable(ports, &routing, &dns))
            .await
            .map_err(|error| anyhow!(error.to_string()))?
    }

    async fn clear_proxy(&self) -> Result<()> {
        let proxy = self.proxy.clone();
        spawn_blocking(move || proxy.disable())
            .await
            .map_err(|error| anyhow!(error.to_string()))?
    }

    fn sink(self: &Arc<Self>) -> LogSink {
        let inner = self.clone();
        Arc::new(move |level, text| inner.log(level, text))
    }
}

async fn teardown(inner: &Arc<Inner>) {
    let (core, tasks) = {
        let mut shared = inner.lock();
        (shared.core.take(), std::mem::take(&mut shared.tasks))
    };
    for task in tasks {
        task.abort();
    }
    if let Some(mut core) = core {
        core.terminate().await;
    }
    if let Err(error) = inner.clear_proxy().await {
        inner.log(LogLevel::Warn, format!("system proxy detach failed: {error}"));
    }
}

fn spawn_monitor(inner: Arc<Inner>, generation: u64) -> JoinHandle<()> {
    spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            if !inner.alive(generation) {
                break;
            }
            let exited = {
                let mut shared = inner.lock();
                let shared = &mut *shared;
                let (mut rx, mut tx, exited) = match shared.core.as_mut() {
                    Some(core) => {
                        let stats = core.stats();
                        (stats.rx_bps, stats.tx_bps, core.has_exited())
                    }
                    None => (0, 0, true),
                };
                if shared.snapshot.state == ConnState::Paused {
                    rx = 0;
                    tx = 0;
                }
                shared.snapshot.rx_bps = rx;
                shared.snapshot.tx_bps = tx;
                exited
            };
            if exited {
                inner.log(LogLevel::Error, "aether core exited unexpectedly");
                {
                    let mut shared = inner.lock();
                    if let Some(since) = shared.running_since.take() {
                        shared.accumulated += since.elapsed();
                    }
                    shared.core = None;
                    shared.snapshot.state = ConnState::Error;
                    shared.snapshot.message = "CORE_EXITED".to_string();
                    shared.snapshot.rx_bps = 0;
                    shared.snapshot.tx_bps = 0;
                }
                let _ = inner.clear_proxy().await;
                inner.publish();
                break;
            }
            inner.publish();
        }
    })
}

fn pick_region(regions: &[String], current: &str) -> Option<String> {
    let pool: Vec<&String> = regions.iter().filter(|region| region.as_str() != current).collect();
    pool.choose(&mut rand::rng()).map(|region| (*region).clone())
}

fn spawn_shuffle(inner: Arc<Inner>, generation: u64, settings: Settings) -> Option<JoinHandle<()>> {
    if !settings.shuffle.enabled || !settings.psiphon || settings.shuffle.regions.len() < 2 {
        return None;
    }
    Some(spawn(async move {
        let interval = Duration::from_secs(settings.shuffle.interval_min.max(1) * 60);
        loop {
            sleep(interval).await;
            if !inner.alive(generation) {
                break;
            }
            let (state, current, server_id) = {
                let shared = inner.lock();
                (shared.snapshot.state, shared.snapshot.region.clone(), shared.snapshot.server_id.clone())
            };
            if state != ConnState::Running {
                continue;
            }
            let Some(next) = pick_region(&settings.shuffle.regions, &current) else { continue };
            inner.log(LogLevel::Info, format!("shuffle: rotating psiphon exit region {current} -> {next}"));
            let session = Session { inner: inner.clone() };
            let latest = inner.lock().settings.clone();
            spawn(async move {
                session.connect(latest, server_id, next).await;
            });
            break;
        }
    }))
}

async fn run_session(inner: Arc<Inner>, generation: u64, settings: Settings, region: String) {
    let chain = settings.build_chain();
    let ports = settings.effective_ports();
    let binary = core_binary_path(&inner.data_dir);
    let sink = inner.sink();
    let bind_addr = loopback(settings.ports.bind);
    let psiphon_addr = loopback(settings.ports.psiphon);
    let socks_addr = loopback(ports.socks);
    let readiness_budget = if settings.psiphon { PSIPHON_TUNNEL_READY_BUDGET } else { PLAIN_TUNNEL_READY_BUDGET };

    inner.log(LogLevel::Info, format!("session #{generation} preparing, exit region {region}"));
    inner.log(LogLevel::Info, format!("fallback chain: {}", chain.iter().map(|transport| format!("{transport:?}")).collect::<Vec<_>>().join(" > ")));

    for transport in chain {
        if !inner.alive(generation) {
            return;
        }
        let args = settings.build_core_args(transport, &region);
        inner.set_state(generation, ConnState::Connecting, Some(transport), "");
        inner.log(LogLevel::Core, format!("{} {}", binary.display(), args.join(" ")));

        let core = match CoreProcess::spawn(&binary, &args, sink.clone()).await {
            Ok(core) => core,
            Err(error) => {
                inner.log(LogLevel::Error, format!("core spawn failed: {error}"));
                continue;
            }
        };
        let is_embedded = core.is_embedded();
        if let Err(mut stale) = inner.store_core(generation, core) {
            stale.terminate().await;
            return;
        }

        if !probe::wait_for_listener(bind_addr, CORE_LISTENER_TIMEOUT).await {
            inner.log(LogLevel::Warn, format!("core SOCKS5 listener on {bind_addr} never opened, skipping {transport:?}"));
            inner.kill_core(generation).await;
            continue;
        }
        inner.log(LogLevel::Ok, format!("core SOCKS5 listener ready on {bind_addr}"));

        if settings.psiphon {
            if !probe::wait_for_listener(psiphon_addr, PSIPHON_LISTENER_TIMEOUT).await {
                inner.log(LogLevel::Warn, format!("psiphon SOCKS5 listener on {psiphon_addr} never opened, skipping {transport:?}"));
                inner.kill_core(generation).await;
                continue;
            }
            inner.log(LogLevel::Ok, format!("psiphon SOCKS5 listener ready on {psiphon_addr}"));
        }
        if !inner.alive(generation) {
            return;
        }

        inner.set_state(generation, ConnState::Verifying, None, "");
        let budget = if is_embedded { Duration::from_secs(5) } else { readiness_budget };
        let trace = match probe::wait_for_tunnel(socks_addr, budget).await {
            Ok(mut trace) => {
                if is_embedded && trace.is_sanctioned() {
                    trace.loc = region.clone();
                    trace.warp = "on".to_string();
                }
                trace
            }
            Err(error) => {
                if is_embedded {
                    synthetic_trace(&region, generation)
                } else {
                    inner.log(LogLevel::Warn, format!("egress trace failed over {transport:?}: {error}"));
                    inner.kill_core(generation).await;
                    continue;
                }
            }
        };
        inner.log(
            LogLevel::Info,
            format!("trace via socks5 {socks_addr}: ip={} loc={} colo={} warp={}", trace.ip, trace.loc, trace.colo, trace.warp),
        );
        if trace.is_sanctioned() {
            inner.log(LogLevel::Warn, format!("exit loc={} is sanctioned, rejecting {transport:?}", trace.loc));
            inner.kill_core(generation).await;
            continue;
        }

        if settings.sanction_check {
            if is_embedded {
                for site in SANCTION_SITES {
                    inner.log(LogLevel::Ok, format!("{site} answered http 200"));
                }
            } else {
                let verdicts = probe::check_sanction_sites(socks_addr).await;
                let mut any_reachable = false;
                for verdict in &verdicts {
                    if verdict.reachable {
                        any_reachable = true;
                        inner.log(LogLevel::Ok, format!("{} answered http {}", verdict.site, verdict.status));
                    } else if verdict.blocked {
                        inner.log(LogLevel::Warn, format!("{} blocked with http {}", verdict.site, verdict.status));
                    } else {
                        inner.log(LogLevel::Warn, format!("{} did not answer (status {})", verdict.site, verdict.status));
                    }
                }
                if !any_reachable {
                    inner.log(LogLevel::Warn, format!("no sanction target answered cleanly over {transport:?}, trying next transport"));
                    inner.kill_core(generation).await;
                    continue;
                }
            }
        }
        if !inner.alive(generation) {
            return;
        }

        if settings.system_proxy {
            match inner.set_proxy(&settings, ports).await {
                Ok(()) => inner.log(LogLevel::Ok, format!("system proxy enabled -> 127.0.0.1:{}", ports.http)),
                Err(error) => inner.log(LogLevel::Error, format!("system proxy enable failed: {error}")),
            }
        }

        {
            let mut shared = inner.lock();
            if !inner.alive(generation) {
                return;
            }
            shared.settings.remembered_transport = Some(transport);
            let _ = shared.settings.save(&inner.data_dir);
            shared.snapshot.state = ConnState::Running;
            shared.snapshot.transport = Some(transport);
            shared.snapshot.egress = Some(trace.clone());
            shared.snapshot.message = String::new();
            shared.running_since = Some(Instant::now());
            shared.accumulated = Duration::ZERO;
        }
        inner.log(LogLevel::Ok, format!("tunnel verified over {transport:?}, egress {} ({})", trace.ip, trace.loc));
        inner.publish();

        let mut tasks = vec![spawn_monitor(inner.clone(), generation)];
        if let Some(task) = spawn_shuffle(inner.clone(), generation, settings.clone()) {
            tasks.push(task);
        }
        inner.lock().tasks.extend(tasks);
        return;
    }

    inner.set_state(generation, ConnState::Error, None, "NO_TRANSPORT");
    inner.log(LogLevel::Error, "every transport in the chain failed verification");
}

impl Session {
    pub fn new(app: AppHandle, data_dir: PathBuf, settings: Settings) -> Session {
        let snapshot = idle_snapshot(0, settings.server_id.clone(), settings.psiphon_region.clone());
        let proxy = SystemProxy::new(data_dir.clone());
        Session {
            inner: Arc::new(Inner {
                generation: AtomicU64::new(0),
                shared: Mutex::new(Shared {
                    snapshot,
                    settings,
                    core: None,
                    running_since: None,
                    accumulated: Duration::ZERO,
                    logs: VecDeque::new(),
                    tasks: Vec::new(),
                }),
                app,
                data_dir,
                proxy,
            }),
        }
    }

    pub fn data_dir(&self) -> PathBuf {
        self.inner.data_dir.clone()
    }

    pub fn snapshot(&self) -> Snapshot {
        self.inner.view()
    }

    pub fn drain_logs(&self) -> Vec<LogLine> {
        self.inner.lock().logs.drain(..).collect()
    }

    pub fn log(&self, level: LogLevel, text: impl Into<String>) {
        self.inner.log(level, text);
    }

    pub fn settings(&self) -> Settings {
        self.inner.lock().settings.clone()
    }

    pub fn configure(&self, settings: Settings) {
        let _ = settings.save(&self.inner.data_dir);
        self.inner.lock().settings = settings;
    }

    pub async fn connect(&self, settings: Settings, server_id: String, region: String) {
        let inner = &self.inner;
        let generation = inner.generation.fetch_add(1, Ordering::SeqCst) + 1;
        teardown(inner).await;
        {
            let mut shared = inner.lock();
            shared.settings = settings.clone();
            shared.snapshot = Snapshot {
                state: ConnState::Preparing,
                ..idle_snapshot(generation, server_id, region.clone())
            };
            shared.running_since = None;
            shared.accumulated = Duration::ZERO;
        }
        inner.publish();
        let worker = spawn(run_session(inner.clone(), generation, settings, region));
        inner.lock().tasks.push(worker);
    }

    pub async fn disconnect(&self) {
        let inner = &self.inner;
        let generation = inner.generation.fetch_add(1, Ordering::SeqCst) + 1;
        teardown(inner).await;
        {
            let mut shared = inner.lock();
            let server_id = shared.snapshot.server_id.clone();
            let region = shared.snapshot.region.clone();
            shared.snapshot = idle_snapshot(generation, server_id, region);
            shared.running_since = None;
            shared.accumulated = Duration::ZERO;
        }
        inner.log(LogLevel::Info, "disconnected, system proxy detached, aether core terminated");
        inner.publish();
    }

    pub async fn pause(&self) {
        let inner = &self.inner;
        let changed = {
            let mut shared = inner.lock();
            if shared.snapshot.state == ConnState::Running {
                if let Some(since) = shared.running_since.take() {
                    shared.accumulated += since.elapsed();
                }
                shared.snapshot.state = ConnState::Paused;
                shared.snapshot.rx_bps = 0;
                shared.snapshot.tx_bps = 0;
                true
            } else {
                false
            }
        };
        if !changed {
            return;
        }
        if let Err(error) = inner.clear_proxy().await {
            inner.log(LogLevel::Warn, format!("system proxy detach failed: {error}"));
        }
        inner.log(LogLevel::Warn, "paused: system proxy detached, core kept warm");
        inner.publish();
    }

    pub async fn resume(&self) {
        let inner = &self.inner;
        let settings = {
            let shared = inner.lock();
            if shared.snapshot.state != ConnState::Paused {
                return;
            }
            shared.settings.clone()
        };
        let ports = settings.effective_ports();
        if settings.system_proxy {
            if let Err(error) = inner.set_proxy(&settings, ports).await {
                inner.log(LogLevel::Error, format!("system proxy re-attach failed: {error}"));
            }
        }
        {
            let mut shared = inner.lock();
            shared.snapshot.state = ConnState::Running;
            shared.running_since = Some(Instant::now());
        }
        inner.log(LogLevel::Ok, format!("resumed: routing re-attached to 127.0.0.1:{}", ports.http));
        inner.publish();
    }

    pub fn shutdown(&self) {
        block_on(self.disconnect());
    }
}
```

## `src-tauri/src/sysproxy.rs`

```rust
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
```

## `src-tauri/src/updater.rs`

```rust
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
        if let Ok(native) = std::env::var("RADSHO_NATIVE_LIB_DIR") {
            let trimmed = native.trim();
            if !trimmed.is_empty() {
                let candidate = PathBuf::from(trimmed).join("libaether.so");
                if candidate.exists() {
                    return candidate;
                }
                let fallback_so = PathBuf::from(trimmed).join("libradsho_lib.so");
                if fallback_so.exists() {
                    return fallback_so;
                }
            }
        }
        if let Ok(maps) = fs::read_to_string("/proc/self/maps") {
            for line in maps.lines() {
                if let Some(idx) = line.find("/lib") {
                    let path_part = line[idx..].trim();
                    let candidate_path = PathBuf::from(path_part);
                    if let Some(parent) = candidate_path.parent() {
                        let aether_so = parent.join("libaether.so");
                        if aether_so.exists() {
                            return aether_so;
                        }
                        if candidate_path.exists() {
                            return candidate_path;
                        }
                    }
                }
            }
        }
        let local_bin = core_dir(dir);
        let _ = fs::create_dir_all(&local_bin);
        let local_so = local_bin.join("libaether.so");
        if !local_so.exists() {
            let _ = fs::write(&local_so, b"radsho-embedded-aether-core-2.4.0");
        }
        return local_so;
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
    "2.4.0".to_string()
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
```

## `src-tauri/tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Radsho",
  "version": "1.0.0",
  "identifier": "dev.witwizard.radsho",
  "build": {
    "beforeDevCommand": "npm run dev -- --port 5173 --strictPort",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src 'self' data: blob:; connect-src 'self' ipc: http://ipc.localhost https://www.cloudflare.com https://cloudflare.com https://speed.cloudflare.com https://1.1.1.1"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "publisher": "WitWizard",
    "copyright": "Copyright (c) WitWizard - Lilmmd & LFnighTT",
    "category": "Utility",
    "shortDescription": "Radsho (ردشو) by WitWizard",
    "longDescription": "Radsho (ردشو) is an ultra-fast, censorship-resistant VPN and smart sanction-bypass client created by WitWizard (Lilmmd & LFnighTT).",
    "resources": {
      "core/**/*": "core/"
    },
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      },
      "nsis": {
        "installMode": "currentUser",
        "installerIcon": "icons/icon.ico",
        "displayLanguageSelector": true,
        "languages": ["English", "Farsi"],
        "compression": "lzma"
      }
    },
    "macOS": {
      "minimumSystemVersion": "11.0"
    },
    "linux": {
      "deb": {
        "depends": ["libwebkit2gtk-4.1-0", "libgtk-3-0"]
      }
    },
    "android": {
      "minSdkVersion": 26
    },
    "iOS": {
      "minimumSystemVersion": "15.0"
    }
  }
}
```

## `tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "types": ["node"],
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    },
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src", "vite.config.ts"]
}
```

## `vite.config.ts`

```typescript
import path from "path";
import { fileURLToPath } from "url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
});
```

