#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PLATFORM="${1:-desktop}"
KEY_DIR="$ROOT/.keys"
KEY_PATH="$KEY_DIR/radsho-updater.key"
OVERLAY="$ROOT/src-tauri/tauri.updater.conf.json"
OUT_DIR="$ROOT/dist-installers"

prepare_toolchain() {
  npm ci
  if [ ! -f src-tauri/icons/icon.png ]; then
    npm run tauri -- icon public/radsho-icon.png
  fi
}

prepare_signing() {
  mkdir -p "$KEY_DIR"
  if [ -n "${RADSHO_UPDATER_PUBKEY:-}" ] && [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
    PUBLIC_KEY="$RADSHO_UPDATER_PUBKEY"
  else
    if [ ! -f "$KEY_PATH" ]; then
      npm run tauri -- signer generate --ci -w "$KEY_PATH" -p ""
    fi
    PUBLIC_KEY="$(tr -d '\n' < "$KEY_PATH.pub")"
    export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY_PATH")"
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
  fi
  python3 - "$OVERLAY" "$PUBLIC_KEY" <<'PY'
import json
import sys

target, key = sys.argv[1], sys.argv[2]
with open(target, "w", encoding="utf-8") as handle:
    json.dump({"plugins": {"updater": {"pubkey": key}}}, handle)
PY
}

collect() {
  mkdir -p "$OUT_DIR"
  find "$ROOT/target" -path "*/release/bundle/*" -type f \
    \( -name "*.AppImage" -o -name "*.deb" -o -name "*.dmg" -o -name "*.exe" -o -name "*.sig" \) \
    -exec cp -f {} "$OUT_DIR/" \; 2>/dev/null || true
  find "$ROOT/src-tauri/gen/android" -type f \( -name "*.apk" -o -name "*.aab" \) -path "*release*" \
    -exec cp -f {} "$OUT_DIR/" \; 2>/dev/null || true
  find "$ROOT/src-tauri/gen/apple/build" -type f -name "*.ipa" -exec cp -f {} "$OUT_DIR/" \; 2>/dev/null || true
}

build_desktop() {
  case "$(uname -s)" in
    Darwin)
      rustup target add aarch64-apple-darwin x86_64-apple-darwin
      npm run tauri -- build --target universal-apple-darwin --bundles dmg,app --config "$OVERLAY"
      ;;
    Linux)
      npm run tauri -- build --bundles appimage,deb --config "$OVERLAY"
      ;;
    *)
      echo "Use scripts/build-windows.ps1 on Windows" >&2
      exit 1
      ;;
  esac
}

build_android() {
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
  if [ ! -d src-tauri/gen/android ]; then
    npm run tauri -- android init
  fi
  local java_dir="src-tauri/gen/android/app/src/main/java"
  mkdir -p "$java_dir/dev/witwizard/radsho/vpn" "$java_dir/hev/htproxy"
  cp src-tauri/mobile/android/RadshoVpnPlugin.kt "$java_dir/dev/witwizard/radsho/vpn/"
  cp src-tauri/mobile/android/RadshoVpnService.kt "$java_dir/dev/witwizard/radsho/vpn/"
  cp src-tauri/mobile/android/TProxyService.kt "$java_dir/hev/htproxy/"
  python3 - <<'PY'
from pathlib import Path

manifest = Path("src-tauri/gen/android/app/src/main/AndroidManifest.xml")
snippet = Path("src-tauri/mobile/android/service-manifest.xml").read_text(encoding="utf-8")
text = manifest.read_text(encoding="utf-8")
if "RadshoVpnService" not in text:
    manifest.write_text(text.replace("</application>", snippet + "    </application>"), encoding="utf-8")
PY
  npm run tauri -- android build --apk --aab --config "$OVERLAY"
}

build_ios() {
  rustup target add aarch64-apple-ios aarch64-apple-ios-sim
  if [ ! -d src-tauri/gen/apple ]; then
    npm run tauri -- ios init
  fi
  mkdir -p src-tauri/gen/apple/Sources/radsho
  cp src-tauri/mobile/ios/RadshoVpnPlugin.swift src-tauri/gen/apple/Sources/radsho/
  mkdir -p src-tauri/gen/apple/PacketTunnel
  cp src-tauri/mobile/ios/PacketTunnelProvider.swift src-tauri/gen/apple/PacketTunnel/
  npm run tauri -- ios build --config "$OVERLAY"
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

collect
echo "Radsho by WitWizard (Lilmmd & LFnighTT): artifacts collected in $OUT_DIR"
