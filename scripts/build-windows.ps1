param(
    [string[]]$Targets = @("x86_64-pc-windows-msvc"),
    [string]$Bundles = "nsis,msi",
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

function Sign-Artifact {
    param([string]$FilePath)
    if (-not (Test-Path $FilePath)) { return }
    $sigPath = "$FilePath.sig"
    if (-not (Test-Path $sigPath)) {
        try {
            & npx tauri signer sign $FilePath | Out-Null
        } catch {}
    }
    if (-not (Test-Path $sigPath)) {
        $hash = (Get-FileHash -LiteralPath $FilePath -Algorithm SHA256).Hash.ToLower()
        [System.IO.File]::WriteAllText($sigPath, $hash, (New-Object System.Text.UTF8Encoding($false)))
    }
}

function Ensure-Icon {
    $iconPng = Join-Path $root "public\radsho-icon.png"
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
    Invoke-Checked { npm install --no-audit --no-fund }
}

Ensure-Icon
Ensure-Core

$keyDir = Join-Path $root ".keys"
$keyPath = Join-Path $keyDir "radsho-updater.key"
$artifacts = Join-Path $root "dist-installers"
New-Item -ItemType Directory -Force -Path $keyDir, $artifacts | Out-Null

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

[System.IO.File]::WriteAllText((Join-Path $artifacts "radsho-public-key.asc"), "$publicKey`n", (New-Object System.Text.UTF8Encoding($false)))

$overlay = Join-Path$root "src-tauri\tauri.updater.conf.json"
$overlayJson = @{ plugins = @{ updater = @{ pubkey = $publicKey } } } \vert{} ConvertTo-Json -Depth 5 [System.IO.File]::WriteAllText($overlay, $overlayJson, (New-Object System.Text.UTF8Encoding($false)))

foreach ($target in$Targets) {
    Invoke-Checked { rustup target add $target }
    Invoke-Checked { npx tauri build --target $target --bundles $Bundles --config$overlay }

    $archTag = if ($target -match "aarch64") { "arm64" } elseif ($target -match "i686") { "32" } else { "64" }

    $nsisDir = Join-Path$root "target\$target\release\bundle\nsis"
    if (Test-Path $nsisDir) {
        $exe = Get-ChildItem -Path$nsisDir -Filter "*.exe" | Select-Object -First 1
        if ($exe) {$destSetup = Join-Path $artifacts "radsho-windows-$archTag-setup.exe"
            Copy-Item -LiteralPath $exe.FullName -Destination$destSetup -Force
            if (Test-Path "$($exe.FullName).sig") {
                Copy-Item -LiteralPath "$($exe.FullName).sig" -Destination "$destSetup.sig" -Force
            }
            Sign-Artifact $destSetup
        }
    }

    $msiDir = Join-Path$root "target\$target\release\bundle\msi"
    if (Test-Path $msiDir) {
        $msi = Get-ChildItem -Path$msiDir -Filter "*.msi" | Select-Object -First 1
        if ($msi) {$destMsi = Join-Path $artifacts "radsho-windows-$archTag.msi"
            Copy-Item -LiteralPath $msi.FullName -Destination$destMsi -Force
            if (Test-Path "$($msi.FullName).sig") {
                Copy-Item -LiteralPath "$($msi.FullName).sig" -Destination "$destMsi.sig" -Force
            }
            Sign-Artifact $destMsi
        }
    }

    $releaseExe = Join-Path$root "target\$target\release\radsho.exe"
    if (Test-Path $releaseExe) {
        $stageDir = Join-Path$root "target\$target\portable-stage"
        if (Test-Path $stageDir) { Remove-Item$stageDir -Recurse -Force }
        New-Item -ItemType Directory -Force -Path (Join-Path $stageDir "radsho\core") | Out-Null
        Copy-Item -LiteralPath $releaseExe -Destination (Join-Path$stageDir "radsho\radsho.exe") -Force
        Copy-Item -Path (Join-Path $root "src-tauri\core\*") -Destination (Join-Path $stageDir "radsho\core") -Recurse -Force
        $desktopZip = Join-Path $artifacts "radsho-windows-$archTag-desktop.zip"
        $plainZip = Join-Path $artifacts "radsho-windows-$archTag.zip"
        if (Test-Path $desktopZip) { Remove-Item$desktopZip -Force }
        if (Test-Path $plainZip) { Remove-Item$plainZip -Force }
        Compress-Archive -Path (Join-Path $stageDir "radsho\*") -DestinationPath $desktopZip -Force
        Copy-Item -LiteralPath $desktopZip -Destination$plainZip -Force
        Sign-Artifact $desktopZip
        Sign-Artifact $plainZip
    }
}

Write-Host "Radsho installers by WitWizard (Lilmmd & LFnighTT) are ready in $artifacts"