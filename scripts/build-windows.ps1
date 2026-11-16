param(
    [string[]]$Targets = @("x86_64-pc-windows-msvc"),
    [string]$Bundles = "nsis",
    [switch]$IncludeAndroid
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Invoke-Checked {
    param([scriptblock]$Command)
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code $LASTEXITCODE"
    }
}

function Ensure-Icon {
    $iconPng = Join-Path $root "public\radsho-icon.png"
    $iconIco = Join-Path $root "src-tauri\icons\icon.ico"
    if (-not (Test-Path $iconPng)) {
        New-Item -ItemType Directory -Force -Path (Split-Path $iconPng) | Out-Null
        Add-Type -AssemblyName System.Drawing
        $bmp = New-Object System.Drawing.Bitmap 512, 512
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
        $g.Clear([System.Drawing.Color]::FromArgb(14, 21, 20))
        $mint = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(0, 229, 153))
        $dark = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(14, 21, 20))
        $g.FillRectangle($mint, 152, 48, 208, 96)
        $g.FillRectangle($mint, 64, 168, 384, 76)
        $g.FillRectangle($mint, 64, 268, 384, 76)
        $g.FillRectangle($dark, 176, 188, 160, 36)
        $pts = [System.Drawing.Point[]]@(
            (New-Object System.Drawing.Point(152, 368)),
            (New-Object System.Drawing.Point(360, 368)),
            (New-Object System.Drawing.Point(360, 412)),
            (New-Object System.Drawing.Point(256, 476)),
            (New-Object System.Drawing.Point(152, 412))
        )
        $g.FillPolygon($mint, $pts)
        $g.Dispose()
        $bmp.Save($iconPng, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
    }
    if (-not (Test-Path $iconIco)) {
        Invoke-Checked { npx tauri icon public/radsho-icon.png }
    }
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
    Invoke-Checked { npm install --no-audit --no-fund }
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
$overlayJson = @{ plugins = @{ updater = @{ pubkey = $publicKey } } } | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($overlay, $overlayJson, (New-Object System.Text.UTF8Encoding($false)))

foreach ($target in $Targets) {
    Invoke-Checked { rustup target add $target }
    Invoke-Checked { npx tauri build --target $target --bundles $Bundles --config $overlay }
}

$artifacts = Join-Path $root "dist-installers"
New-Item -ItemType Directory -Force -Path $artifacts | Out-Null
foreach ($target in $Targets) {
    $nsis = Join-Path $root "target\$target\release\bundle\nsis"
    if (Test-Path $nsis) {
        Copy-Item -Path (Join-Path $nsis "*") -Destination $artifacts -Recurse -Force
    }
}

if ($IncludeAndroid -and $env:ANDROID_HOME) {
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
    if (-not (Test-Path "src-tauri\gen\android")) {
        Invoke-Checked { npx tauri android init --ci }
    }
    $javaDir = "src-tauri\gen\android\app\src\main\java"
    New-Item -ItemType Directory -Force -Path "$javaDir\dev\witwizard\radsho\vpn", "$javaDir\hev\htproxy" | Out-Null
    Copy-Item "src-tauri\mobile\android\RadshoVpnPlugin.kt" "$javaDir\dev\witwizard\radsho\vpn\" -Force
    Copy-Item "src-tauri\mobile\android\RadshoVpnService.kt" "$javaDir\dev\witwizard\radsho\vpn\" -Force
    Copy-Item "src-tauri\mobile\android\TProxyService.kt" "$javaDir\hev\htproxy\" -Force
    $manifestPath = "src-tauri\gen\android\app\src\main\AndroidManifest.xml"
    $snippet = Get-Content "src-tauri\mobile\android\service-manifest.xml" -Raw
    $manifestText = Get-Content $manifestPath -Raw
    if ($manifestText -notmatch "RadshoVpnService") {
        $manifestText = $manifestText.Replace("</application>", "$snippet`n    </application>")
        [System.IO.File]::WriteAllText($manifestPath, $manifestText, (New-Object System.Text.UTF8Encoding($false)))
    }
    Invoke-Checked { npx tauri android build --apk --aab --config $overlay }
    Get-ChildItem -Path "src-tauri\gen\android" -Recurse -Include "*.apk", "*.aab" | Where-Object { $_.FullName -match "release" } | ForEach-Object {
        Copy-Item $_.FullName -Destination$artifacts -Force
    }
}

Write-Host "Radsho installers by WitWizard (Lilmmd & LFnighTT) are ready in $artifacts"