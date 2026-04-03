# Aether Programming Language Installer for Windows
# Usage: irm https://aether-lang.org/install.ps1 | iex
#    or: irm https://raw.githubusercontent.com/aether-lang/aether/main/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$Repo = "aether-lang/aether"
$InstallDir = "$env:LOCALAPPDATA\aether"
$BinDir = "$InstallDir\bin"

function Write-Info($msg)  { Write-Host "  info: " -ForegroundColor Cyan -NoNewline; Write-Host $msg }
function Write-Ok($msg)    { Write-Host "    ok: " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warn($msg)  { Write-Host "  warn: " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Err($msg)   { Write-Host " error: " -ForegroundColor Red -NoNewline; Write-Host $msg }

Write-Host ""
Write-Host "  Aether Installer" -ForegroundColor Cyan
Write-Host ""

# ── Detect architecture ──────────────────────────────────────────

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
if ($Arch -eq "X64") {
    $Artifact = "aether-windows-x86_64"
} elseif ($Arch -eq "Arm64") {
    Write-Err "ARM64 Windows builds are not yet available."
    Write-Info "Please build from source: cargo build --release"
    exit 1
} else {
    Write-Err "Unsupported architecture: $Arch"
    exit 1
}

Write-Info "Platform: windows-$($Arch.ToString().ToLower())"

# ── Get version ──────────────────────────────────────────────────

if ($env:AETHER_VERSION) {
    $Version = $env:AETHER_VERSION
    Write-Info "Installing version: $Version"
} else {
    Write-Info "Fetching latest version..."
    try {
        $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -ErrorAction Stop
        $Version = $Release.tag_name
        Write-Info "Latest version: $Version"
    } catch {
        Write-Warn "Could not determine latest version."
        Write-Warn "Please build from source: cargo build --release"
        exit 1
    }
}

# ── Download ─────────────────────────────────────────────────────

$Url = "https://github.com/$Repo/releases/download/$Version/$Artifact.zip"
$ShaUrl = "$Url.sha256"
$TmpDir = Join-Path $env:TEMP "aether-install-$(Get-Random)"
$ZipFile = Join-Path $TmpDir "$Artifact.zip"

New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

Write-Info "Downloading $Artifact..."
try {
    Invoke-WebRequest -Uri $Url -OutFile $ZipFile -ErrorAction Stop
} catch {
    Write-Err "Download failed. Binary may not be available for this version/platform."
    Write-Info "Build from source instead:"
    Write-Info "  git clone https://github.com/$Repo.git"
    Write-Info "  cd aether && cargo build --release"
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
    exit 1
}

# Verify checksum
try {
    $ExpectedSha = (Invoke-WebRequest -Uri $ShaUrl -ErrorAction Stop).Content.Trim().Split()[0]
    $ActualSha = (Get-FileHash $ZipFile -Algorithm SHA256).Hash.ToLower()
    if ($ExpectedSha -and $ActualSha -ne $ExpectedSha) {
        Write-Err "SHA256 checksum mismatch!"
        Write-Err "  Expected: $ExpectedSha"
        Write-Err "  Actual:   $ActualSha"
        Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
        exit 1
    }
    Write-Ok "Checksum verified"
} catch {
    Write-Warn "Could not verify checksum (non-fatal)"
}

# ── Extract and install ──────────────────────────────────────────

Write-Info "Installing..."
Expand-Archive -Path $ZipFile -DestinationPath $TmpDir -Force

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item (Join-Path $TmpDir "aether.exe") (Join-Path $BinDir "aether.exe") -Force

Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue

Write-Ok "Installed to $BinDir\aether.exe"

# ── Add to PATH ──────────────────────────────────────────────────

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$BinDir;$UserPath", "User")
    Write-Info "Added $BinDir to user PATH"
    $env:Path = "$BinDir;$env:Path"
}

# ── Verify ────────────────────────────────────────────────────────

try {
    $InstalledVersion = & "$BinDir\aether.exe" --version 2>&1
    Write-Ok "$InstalledVersion installed successfully!"
} catch {
    Write-Ok "Binary installed"
}

Write-Host ""
Write-Host "  Aether has been installed!" -ForegroundColor Green
Write-Host ""
Write-Host "  To get started, run:" -ForegroundColor White
Write-Host "    aether --version" -ForegroundColor Cyan
Write-Host "    aether repl" -ForegroundColor Cyan
Write-Host "    aether run hello.ae" -ForegroundColor Cyan
Write-Host ""
Write-Host "  You may need to restart your terminal for PATH changes." -ForegroundColor Yellow
Write-Host ""
