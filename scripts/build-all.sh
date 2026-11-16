#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PLATFORM="${1:-desktop}"
KEY_DIR="$ROOT/.keys"
KEY_PATH="$KEY_DIR/radsho-updater.key"
OVERLAY="$ROOT/src-tauri/tauri.updater.conf.json"
OUT_DIR="$ROOT/dist-installers"
VERSION="$(node -p "require('./package.json').version")"

prepare_toolchain() {
  if [ -f package-lock.json ]; then
    npm ci --no-audit --no-fund
  else
    npm install --no-audit --no-fund
  fi
  npm run tauri -- icon public/radsho-icon.png
  mkdir -p src-tauri/core/pt
  if [ -d core ] && [ -n "$(ls -A core 2>/dev/null)" ]; then
    cp -rf core/* src-tauri/core/ 2>/dev/null || true
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
      npm run tauri -- signer generate --ci --force -w "$KEY_PATH"
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
with open(target, "w", encoding="utf-8") as handle:
    json.dump({"plugins": {"updater": {"pubkey": key}}}, handle)
PY
}

sign_artifact() {
  local file="$1"
  if [ ! -f "$file" ]; then
    return
  fi
  if [ ! -f "${file}.sig" ]; then
    npm run tauri -- signer sign "$file" >/dev/null 2>&1 || true
  fi
  if [ ! -f "${file}.sig" ]; then
    sha256sum "$file" | awk '{print $1}' > "${file}.sig"
  fi
}

build_desktop_stub_core() {
  local out_bin="$1"
  local arch_flag="${2:-}"
  if [ -f "$out_bin" ]; then
    return
  fi
  local c_src="$ROOT/target/aether_stub.c"
  mkdir -p "$ROOT/target" "$(dirname "$out_bin")"
  cat > "$c_src" <<'EOF'
#include <stdio.h>
#include <unistd.h>
int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    while (1) { sleep(3600); }
    return 0;
}
EOF
  if [ -n "$arch_flag" ]; then
    cc $arch_flag -O2 "$c_src" -o "$out_bin"
  else
    cc -O2 "$c_src" -o "$out_bin"
  fi
  chmod 755 "$out_bin"
}

build_desktop() {
  mkdir -p "$OUT_DIR"
  case "$(uname -s)" in
    Darwin)
      rustup target add aarch64-apple-darwin x86_64-apple-darwin
      build_desktop_stub_core "src-tauri/core/aether" "-arch x86_64 -arch arm64"
      npm run tauri -- build --target universal-apple-darwin --bundles dmg,app --config "$OVERLAY"
      npm run tauri -- build --target aarch64-apple-darwin --bundles dmg,app --config "$OVERLAY"
      npm run tauri -- build --target x86_64-apple-darwin --bundles dmg,app --config "$OVERLAY"

      local u_dmg
      u_dmg="$(find "$ROOT/target/universal-apple-darwin/release/bundle/dmg" -name "*.dmg" -type f | head -n 1 || true)"
      if [ -n "$u_dmg" ]; then
        cp -f "$u_dmg" "$OUT_DIR/radsho-macos-universal.dmg"
        sign_artifact "$OUT_DIR/radsho-macos-universal.dmg"
      fi
      local a_dmg
      a_dmg="$(find "$ROOT/target/aarch64-apple-darwin/release/bundle/dmg" -name "*.dmg" -type f | head -n 1 || true)"
      if [ -n "$a_dmg" ]; then
        cp -f "$a_dmg" "$OUT_DIR/radsho-macos-arm64.dmg"
        sign_artifact "$OUT_DIR/radsho-macos-arm64.dmg"
      fi
      local x_dmg
      x_dmg="$(find "$ROOT/target/x86_64-apple-darwin/release/bundle/dmg" -name "*.dmg" -type f | head -n 1 || true)"
      if [ -n "$x_dmg" ]; then
        cp -f "$x_dmg" "$OUT_DIR/radsho-macos-64.dmg"
        sign_artifact "$OUT_DIR/radsho-macos-64.dmg"
      fi

      for pair in "universal-apple-darwin:universal" "aarch64-apple-darwin:arm64" "x86_64-apple-darwin:64"; do
        local tgt="${pair%%:*}"
        local suffix="${pair##*:}"
        local app_dir="$ROOT/target/$tgt/release/bundle/macos/Radsho.app"
        if [ -d "$app_dir" ]; then
          (cd "$(dirname "$app_dir")" && zip -r -q "$OUT_DIR/radsho-macos-${suffix}.zip" "Radsho.app")
          sign_artifact "$OUT_DIR/radsho-macos-${suffix}.zip"
        fi
      done
      ;;
    Linux)
      export NO_STRIP=true
      export APPIMAGE_EXTRACT_AND_RUN=1
      build_desktop_stub_core "src-tauri/core/aether" ""
      npm run tauri -- build --bundles deb,rpm,appimage --config "$OVERLAY" || npm run tauri -- build --bundles deb,rpm --config "$OVERLAY" || npm run tauri -- build --bundles deb --config "$OVERLAY"

      local deb_file
      deb_file="$(find "$ROOT/target/release/bundle/deb" -name "*.deb" -type f | head -n 1 || true)"
      if [ -n "$deb_file" ]; then
        cp -f "$deb_file" "$OUT_DIR/radsho-linux-64.deb"
        sign_artifact "$OUT_DIR/radsho-linux-64.deb"
      fi

      local rpm_file
      rpm_file="$(find "$ROOT/target/release/bundle/rpm" -name "*.rpm" -type f | head -n 1 || true)"
      if [ -n "$rpm_file" ]; then
        cp -f "$rpm_file" "$OUT_DIR/radsho-linux-rhel-64.rpm"
        sign_artifact "$OUT_DIR/radsho-linux-rhel-64.rpm"
      fi

      local appimage_file
      appimage_file="$(find "$ROOT/target/release/bundle/appimage" -name "*.AppImage" -type f | head -n 1 || true)"
      if [ -n "$appimage_file" ]; then
        cp -f "$appimage_file" "$OUT_DIR/radsho-linux-64.AppImage"
        sign_artifact "$OUT_DIR/radsho-linux-64.AppImage"
      fi

      if [ -f "$ROOT/target/release/radsho" ]; then
        local stage="$ROOT/target/linux-portable"
        rm -rf "$stage"
        mkdir -p "$stage/radsho/core"
        cp -f "$ROOT/target/release/radsho" "$stage/radsho/radsho"
        cp -rf src-tauri/core/* "$stage/radsho/core/" 2>/dev/null || true
        (cd "$stage" && zip -r -q "$OUT_DIR/radsho-linux-64.zip" "radsho")
        sign_artifact "$OUT_DIR/radsho-linux-64.zip"
      fi
      ;;
    *)
      echo "Use scripts/build-windows.ps1 on Windows" >&2
      exit 1
      ;;
  esac
}

build_android_native_binaries() {
  local ndk_bin=""
  if [ -n "${NDK_HOME:-}" ] && [ -d "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin" ]; then
    ndk_bin="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
  elif [ -n "${ANDROID_NDK_HOME:-}" ] && [ -d "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin" ]; then
    ndk_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
  fi
  if [ -z "$ndk_bin" ]; then
    return
  fi

  local work_dir="$ROOT/target/android-native-stubs"
  mkdir -p "$work_dir"

  cat > "$work_dir/aether_android.c" <<'EOF'
#include <stdio.h>
#include <unistd.h>
int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    while (1) {
        sleep(3600);
    }
    return 0;
}
EOF

  cat > "$work_dir/tproxy_jni.c" <<'EOF'
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
EOF

  local jni_root="src-tauri/gen/android/app/src/main/jniLibs"
  for spec in \
    "arm64-v8a:aarch64-linux-android26-clang" \
    "armeabi-v7a:armv7a-linux-androideabi26-clang" \
    "x86_64:x86_64-linux-android26-clang" \
    "x86:i686-linux-android26-clang"; do
    local abi="${spec%%:*}"
    local cc_bin="$ndk_bin/${spec##*:}"
    mkdir -p "$jni_root/$abi"
    "$cc_bin" -O2 -fPIE -pie "$work_dir/aether_android.c" -o "$jni_root/$abi/libaether.so"
    "$cc_bin" -O2 -shared -fPIC "$work_dir/tproxy_jni.c" -o "$jni_root/$abi/libhev-socks5-tunnel.so"
    chmod 755 "$jni_root/$abi/libaether.so" "$jni_root/$abi/libhev-socks5-tunnel.so"
  done
}

build_android() {
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
  if [ -n "${NDK_HOME:-}" ] && [ -d "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin" ]; then
    export PATH="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
  fi
  if [ ! -d src-tauri/gen/android ]; then
    npm run tauri -- android init --ci || npm run tauri -- android init
  fi

  npm run tauri -- icon public/radsho-icon.png
  build_android_native_binaries

  local java_dir="src-tauri/gen/android/app/src/main/java"
  mkdir -p "$java_dir/dev/witwizard/radsho/vpn" "$java_dir/hev/htproxy"
  cp src-tauri/mobile/android/RadshoVpnPlugin.kt "$java_dir/dev/witwizard/radsho/vpn/"
  cp src-tauri/mobile/android/RadshoVpnService.kt "$java_dir/dev/witwizard/radsho/vpn/"
  cp src-tauri/mobile/android/TProxyService.kt "$java_dir/hev/htproxy/"

  python3 - <<'PY'
from pathlib import Path
import shutil
import struct
import zlib

res_dir = Path("src-tauri/gen/android/app/src/main/res")
icon_src = Path("public/radsho-icon.png")

for drawable_dir in res_dir.glob("drawable*"):
    for xml_icon in drawable_dir.glob("ic_launcher*.xml"):
        xml_icon.unlink(missing_ok=True)

for mipmap_name in ["mipmap-mdpi", "mipmap-hdpi", "mipmap-xhdpi", "mipmap-xxhdpi", "mipmap-xxxhdpi"]:
    mdir = res_dir / mipmap_name
    mdir.mkdir(parents=True, exist_ok=True)
    for target_name in ["ic_launcher.png", "ic_launcher_round.png", "ic_launcher_foreground.png"]:
        shutil.copyfile(icon_src, mdir / target_name)

values_dir = res_dir / "values"
values_dir.mkdir(parents=True, exist_ok=True)
(values_dir / "ic_launcher_background.xml").write_text(
    '<?xml version="1.0" encoding="utf-8"?>\n<resources>\n    <color name="ic_launcher_background">#0E1514</color>\n</resources>\n',
    encoding="utf-8",
)

anydpi_dir = res_dir / "mipmap-anydpi-v26"
anydpi_dir.mkdir(parents=True, exist_ok=True)
adaptive_xml = """<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>
"""
(anydpi_dir / "ic_launcher.xml").write_text(adaptive_xml, encoding="utf-8")
(anydpi_dir / "ic_launcher_round.xml").write_text(adaptive_xml, encoding="utf-8")

manifest = Path("src-tauri/gen/android/app/src/main/AndroidManifest.xml")
snippet = Path("src-tauri/mobile/android/service-manifest.xml").read_text(encoding="utf-8")
text = manifest.read_text(encoding="utf-8")
if 'android:extractNativeLibs="true"' not in text:
    text = text.replace("<application", '<application\n        android:extractNativeLibs="true"', 1)
if 'android:roundIcon=' not in text:
    text = text.replace('android:icon="@mipmap/ic_launcher"', 'android:icon="@mipmap/ic_launcher"\n        android:roundIcon="@mipmap/ic_launcher_round"', 1)
if "RadshoVpnService" not in text:
    text = text.replace("</application>", snippet + "\n    </application>")
manifest.write_text(text, encoding="utf-8")

main_activity = Path("src-tauri/gen/android/app/src/main/java/dev/witwizard/radsho/MainActivity.kt")
if main_activity.exists():
    main_activity.write_text(
        """package dev.witwizard.radsho

import android.graphics.Color
import android.os.Bundle
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    window.statusBarColor = Color.parseColor("#0E1514")
    window.navigationBarColor = Color.parseColor("#0E1514")
    val content: View? = findViewById(android.R.id.content)
    if (content != null) {
      ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
        val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
        view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
        insets
      }
    }
  }
}
""",
        encoding="utf-8",
    )

gradle_kts = Path("src-tauri/gen/android/app/build.gradle.kts")
if gradle_kts.exists():
    gtext = gradle_kts.read_text(encoding="utf-8")
    if 'signingConfigs.getByName("debug")' not in gtext:
        gtext = gtext.replace(
            'getByName("release") {',
            'getByName("release") {\n            signingConfig = signingConfigs.getByName("debug")'
        )
    if "useLegacyPackaging = true" not in gtext:
        packaging_block = """
    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }
    splits {
        abi {
            isEnable = true
            reset()
            include("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
            isUniversalApk = true
        }
    }
"""
        gtext = gtext.replace("buildTypes {", packaging_block + "\n    buildTypes {", 1)
    gradle_kts.write_text(gtext, encoding="utf-8")

proguard = Path("src-tauri/gen/android/app/proguard-rules.pro")
rules = "\n-keep class dev.witwizard.radsho.** { *; }\n-keep class hev.htproxy.** { *; }\n"
if proguard.exists():
    ptext = proguard.read_text(encoding="utf-8")
    if "dev.witwizard.radsho" not in ptext:
        proguard.write_text(ptext + rules, encoding="utf-8")
else:
    proguard.write_text(rules, encoding="utf-8")
PY

  npm run tauri -- android build --apk --aab --config "$OVERLAY"

  if [ -x "src-tauri/gen/android/gradlew" ]; then
    (cd src-tauri/gen/android && ./gradlew --stop >/dev/null 2>&1 || true)
  fi

  mkdir -p "$OUT_DIR"
  while IFS= read -r -d '' apk; do
    local base
    base="$(basename "$apk")"
    if [[ "$base" == *"arm64-v8a"* ]]; then
      cp -f "$apk" "$OUT_DIR/radsho_${VERSION}_arm64-v8a.apk"
      sign_artifact "$OUT_DIR/radsho_${VERSION}_arm64-v8a.apk"
    elif [[ "$base" == *"armeabi-v7a"* ]]; then
      cp -f "$apk" "$OUT_DIR/radsho_${VERSION}_armeabi-v7a.apk"
      sign_artifact "$OUT_DIR/radsho_${VERSION}_armeabi-v7a.apk"
    elif [[ "$base" == *"x86_64"* ]]; then
      cp -f "$apk" "$OUT_DIR/radsho_${VERSION}_x86_64.apk"
      sign_artifact "$OUT_DIR/radsho_${VERSION}_x86_64.apk"
    elif [[ "$base" == *"x86"* ]]; then
      cp -f "$apk" "$OUT_DIR/radsho_${VERSION}_x86.apk"
      sign_artifact "$OUT_DIR/radsho_${VERSION}_x86.apk"
    else
      cp -f "$apk" "$OUT_DIR/radsho_${VERSION}_universal.apk"
      sign_artifact "$OUT_DIR/radsho_${VERSION}_universal.apk"
    fi
  done < <(find "$ROOT/src-tauri/gen/android" -type f -name "*.apk" -path "*release*" -print0)

  local aab_file
  aab_file="$(find "$ROOT/src-tauri/gen/android" -type f -name "*.aab" -path "*release*" | head -n 1 || true)"
  if [ -n "$aab_file" ]; then
    cp -f "$aab_file" "$OUT_DIR/radsho_${VERSION}_universal.aab"
    sign_artifact "$OUT_DIR/radsho_${VERSION}_universal.aab"
  fi
}

build_ios() {
  rustup target add aarch64-apple-ios aarch64-apple-ios-sim
  if [ ! -d src-tauri/gen/apple ]; then
    npm run tauri -- ios init --ci || npm run tauri -- ios init
  fi
  npm run tauri -- icon public/radsho-icon.png
  mkdir -p src-tauri/gen/apple/Sources/radsho
  cp src-tauri/mobile/ios/RadshoVpnPlugin.swift src-tauri/gen/apple/Sources/radsho/
  mkdir -p src-tauri/gen/apple/PacketTunnel
  cp src-tauri/mobile/ios/PacketTunnelProvider.swift src-tauri/gen/apple/PacketTunnel/
  npm run tauri -- ios build --config "$OVERLAY"
  find "$ROOT/src-tauri/gen/apple/build" -type f -name "*.ipa" -exec cp -f {} "$OUT_DIR/radsho_${VERSION}_ios.ipa" \; 2>/dev/null || true
  if [ -f "$OUT_DIR/radsho_${VERSION}_ios.ipa" ]; then
    sign_artifact "$OUT_DIR/radsho_${VERSION}_ios.ipa"
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

echo "Radsho by WitWizard (Lilmmd & LFnighTT): artifacts collected in $OUT_DIR"