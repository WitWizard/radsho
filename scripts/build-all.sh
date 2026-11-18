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

run_clean() {
  python3 - "$@" <<'PY'
import re
import subprocess
import sys

ansi = re.compile(r"\x1b\[[0-9;]*[a-zA-Z]")
noise = re.compile(
    r"^\s*(Info|Warn|info:|warning:|WARNING:|Unable to strip|This Android Gradle plugin|You are strongly encouraged|If you are already using|To suppress this warning|android\.suppressUnsupportedCompileSdk)"
)

proc = subprocess.Popen(
    sys.argv[1:],
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    text=True,
    bufsize=1,
)
assert proc.stdout is not None
for line in proc.stdout:
    clean = ansi.sub("", line)
    if not noise.match(clean):
        sys.stdout.write(line)
        sys.stdout.flush()
sys.exit(proc.wait())
PY
}

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
    npm ci --no-audit --no-fund --silent >/dev/null 2>&1
  else
    npm install --no-audit --no-fund --silent >/dev/null 2>&1
  fi
  npm run --silent tauri -- icon public/radsho-icon.png >/dev/null 2>&1
  mkdir -p src-tauri/core/pt
  rm -f "$ROOT/src-tauri/core/"*.exe "$ROOT/src-tauri/core/pt/"*.exe 2>/dev/null || true
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
      npm run --silent tauri -- signer generate --ci --force -w "$KEY_PATH" >/dev/null 2>&1
    fi
    PUBLIC_KEY="$(tr -d '\r\n' < "$KEY_PATH.pub")"
    export TAURI_SIGNING_PRIVATE_KEY="$(tr -d '\r\n' < "$KEY_PATH")"
    export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
  fi
  printf "%s\n" "$PUBLIC_KEY" > "$OUT_DIR/radsho-public-key.asc"
  local create_updater="false"
  if [ "$(uname -s)" = "Darwin" ]; then
    create_updater="true"
  fi
  python3 - "$OVERLAY" "$PUBLIC_KEY" "$create_updater" <<'PY'
import json
import sys

target, key, enable_artifacts = sys.argv[1], sys.argv[2], sys.argv[3] == "true"
payload = {
    "bundle": {
        "createUpdaterArtifacts": enable_artifacts
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
      run_clean npm run --silent tauri -- build --target universal-apple-darwin --bundles app --config "$OVERLAY"
      local app_dir
      app_dir="$(find "$ROOT/target/universal-apple-darwin/release/bundle/macos" -name "*.app" -type d | head -n 1 || true)"
      if [ -n "$app_dir" ]; then
        local dmg_stage="$ROOT/target/dmg-stage"
        rm -rf "$dmg_stage"
        mkdir -p "$dmg_stage"
        cp -R "$app_dir" "$dmg_stage/"
        ln -s /Applications "$dmg_stage/Applications" 2>/dev/null || true
        hdiutil create -volname "Radsho" -srcfolder "$dmg_stage" -ov -format UDZO "$OUT_DIR/Radsho-macos-universal.dmg" >/dev/null 2>&1
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
      run_clean npm run --silent tauri -- build --bundles deb,rpm --config "$OVERLAY"

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
    npm run --silent tauri -- android init --ci >/dev/null 2>&1 || npm run tauri -- android init >/dev/null 2>&1
  fi

  npm run --silent tauri -- icon public/radsho-icon.png >/dev/null 2>&1

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
import os
import re
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
        gp_text += "\norg.gradle.warning.mode=none\norg.gradle.logging.level=quiet\nandroid.suppressUnsupportedCompileSdk=34,35,36\n"
        gradle_props.write_text(gp_text, encoding="utf-8")

ndk_home = os.environ.get("NDK_HOME", "")
ndk_ver = Path(ndk_home).name if ndk_home else "27.0.12077973"

gradle_kts = Path("src-tauri/gen/android/app/build.gradle.kts")
if gradle_kts.exists():
    gtext = gradle_kts.read_text(encoding="utf-8")
    gtext = re.sub(r"compileSdk\s*=\s*\d+", "compileSdk = 34", gtext)
    gtext = re.sub(r"targetSdk\s*=\s*\d+", "targetSdk = 34", gtext)
    if 'signingConfigs.getByName("debug")' not in gtext:
        gtext = gtext.replace(
            'getByName("release") {',
            'getByName("release") {\n            signingConfig = signingConfigs.getByName("debug")'
        )
    if "useLegacyPackaging = true" not in gtext:
        gtext = gtext.replace(
            "android {",
            f'android {{\n    ndkVersion = "{ndk_ver}"\n    packaging {{\n        jniLibs {{\n            useLegacyPackaging = true\n            keepDebugSymbols.add("**/*.so")\n        }}\n    }}'
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

  run_clean npm run --silent tauri -- android build --apk --aab --split-per-abi

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
    npm run --silent tauri -- ios init --ci >/dev/null 2>&1 || npm run tauri -- ios init >/dev/null 2>&1
  fi
  npm run --silent tauri -- icon public/radsho-icon.png >/dev/null 2>&1
  mkdir -p src-tauri/gen/apple/Sources/radsho
  cp src-tauri/mobile/ios/RadshoVpnPlugin.swift src-tauri/gen/apple/Sources/radsho/
  mkdir -p src-tauri/gen/apple/PacketTunnel
  cp src-tauri/mobile/ios/PacketTunnelProvider.swift src-tauri/gen/apple/PacketTunnel/
  run_clean npm run --silent tauri -- ios build
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