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

function Invoke-Silent {
    param([string]$CommandLine)
    cmd.exe /c "$CommandLine >nul 2>&1"
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code $LASTEXITCODE : $CommandLine"
    }
}

function Invoke-TauriBuild {
    param(
        [string]$Target,
        [string]$BundleList,
        [string]$ConfigPath
    )
    $runnerJs = @'
const { spawn } = require("child_process");
const args = ["tauri", "build", "--target", process.argv[2], "--bundles", process.argv[3], "--config", process.argv[4]];
const child = spawn("npx.cmd", args, { stdio: ["ignore", "pipe", "pipe"], shell: true });
const ansi = /\x1b\[[0-9;]*[a-zA-Z]/g;
const allowed = /^\s*(vite\s+v|✓|dist\/|Compiling\s+|Finished\s+|error(\[|:)|Error(\[|:|\s))/;
const tail = [];
function handleChunk(chunk) {
  const lines = chunk.toString("utf8").split(/\r?\n/);
  for (const line of lines) {
    if (!line) continue;
    tail.push(line);
    if (tail.length > 80) tail.shift();
    const clean = line.replace(ansi, "");
    if (allowed.test(clean)) {
      process.stdout.write(line + "\n");
    }
  }
}
child.stdout.on("data", handleChunk);
child.stderr.on("data", handleChunk);
child.on("close", (code) => {
  if (code !== 0) {
    for (const l of tail) process.stderr.write(l + "\n");
    process.exit(code || 1);
  }
  process.exit(0);
});
'@
    $runnerPath = Join-Path $root "target\tauri-runner.cjs"
    New-Item -ItemType Directory -Force -Path (Split-Path $runnerPath) | Out-Null
    [System.IO.File]::WriteAllText($runnerPath, $runnerJs, (New-Object System.Text.UTF8Encoding($false)))
    try {
        & node $runnerPath $Target $BundleList $ConfigPath
        if ($LASTEXITCODE -ne 0) {
            throw "Tauri build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Remove-Item $runnerPath -Force -ErrorAction SilentlyContinue
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
    Invoke-Silent "npx tauri icon public/radsho-icon.png"
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
    Invoke-Silent "npm install --no-audit --no-fund --silent"
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
        Invoke-Silent "npx tauri signer generate --ci --force -w `"$keyPath`""
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

foreach ($target in $Targets) {
    Invoke-Silent "rustup target add $target"
    Invoke-TauriBuild -Target $target -BundleList $Bundles -ConfigPath $overlay

    $archLabel = if ($target -match "aarch64") { "arm64" } elseif ($target -match "i686") { "32" } else { "64" }
    $relDir = Join-Path $root "target\$target\release"

    $nsisDir = Join-Path $relDir "bundle\nsis"
    if (Test-Path $nsisDir) {
        $exe = Get-ChildItem -Path $nsisDir -Filter "*.exe" | Select-Object -First 1
        if ($exe) {
            $destExe = Join-Path $artifacts "Radsho-windows-$archLabel-setup.exe"
            Copy-Item $exe.FullName -Destination $destExe -Force
            $sig = "$($exe.FullName).sig"
            if (Test-Path $sig) {
                Copy-Item $sig -Destination "$destExe.sig" -Force
            } else {
                Write-SigFile $destExe
            }
        }
    }

    $msiDir = Join-Path $relDir "bundle\msi"
    if (Test-Path $msiDir) {
        $msi = Get-ChildItem -Path $msiDir -Filter "*.msi" | Select-Object -First 1
        if ($msi) {
            $destMsi = Join-Path $artifacts "Radsho-windows-$archLabel.msi"
            Copy-Item $msi.FullName -Destination $destMsi -Force
            $sig = "$($msi.FullName).sig"
            if (Test-Path $sig) {
                Copy-Item $sig -Destination "$destMsi.sig" -Force
            } else {
                Write-SigFile $destMsi
            }
        }
    }

    $mainExe = Join-Path $relDir "radsho.exe"
    if (Test-Path $mainExe) {
        $portableStage = Join-Path $root "target\portable-stage-$archLabel"
        if (Test-Path $portableStage) {
            Remove-Item $portableStage -Recurse -Force
        }
        New-Item -ItemType Directory -Force -Path (Join-Path $portableStage "core") | Out-Null
        Copy-Item $mainExe -Destination (Join-Path $portableStage "radsho.exe") -Force
        Copy-Item (Join-Path $root "src-tauri\core\*") -Destination (Join-Path $portableStage "core") -Recurse -Force

        $zipPortable = Join-Path $artifacts "Radsho-windows-$archLabel.zip"
        $zipDesktop = Join-Path $artifacts "Radsho-windows-$archLabel-desktop.zip"
        Compress-Archive -Path "$portableStage\*" -DestinationPath $zipPortable -Force
        Copy-Item $zipPortable -Destination $zipDesktop -Force
        Write-SigFile $zipPortable
        Write-SigFile $zipDesktop
        Remove-Item $portableStage -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "Radsho v$version artifacts by WitWizard (Lilmmd & LFnighTT) are ready in $artifacts"