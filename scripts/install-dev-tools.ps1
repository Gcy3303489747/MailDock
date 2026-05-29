param(
    [switch]$SkipVisualStudioBuildTools
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$InstallDir = "C:\tmp\MailDockInstall"
$UserToolsDir = Join-Path $env:USERPROFILE "DevTools"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $UserToolsDir | Out-Null

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Download-File {
    param(
        [string]$Uri,
        [string]$OutFile
    )

    Write-Host "Downloading $Uri"
    Invoke-WebRequest -Uri $Uri -OutFile $OutFile -UseBasicParsing
}

function Run-Installer {
    param(
        [string]$FilePath,
        [string[]]$ArgumentList
    )

    Write-Host "Running $FilePath $($ArgumentList -join ' ')"
    $process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -Wait -PassThru
    if ($process.ExitCode -ne 0 -and $process.ExitCode -ne 3010) {
        throw "Installer failed with exit code $($process.ExitCode): $FilePath"
    }
}

function Add-UserPath {
    param([string]$PathToAdd)

    if (-not (Test-Path $PathToAdd)) {
        throw "Cannot add missing path to PATH: $PathToAdd"
    }

    $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @()
    if ($currentUserPath) {
        $parts = $currentUserPath -split ";" | Where-Object { $_ }
    }

    if ($parts -notcontains $PathToAdd) {
        $nextPath = ($parts + $PathToAdd) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $nextPath, "User")
    }

    if (($env:Path -split ";") -notcontains $PathToAdd) {
        $env:Path = "$env:Path;$PathToAdd"
    }
}

Write-Step "Installing Git for Windows"
if (Test-Path "C:\Program Files\Git\cmd\git.exe") {
    Write-Host "Git is already installed."
}
else {
    $gitRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/git-for-windows/git/releases/latest"
    $gitAsset = $gitRelease.assets |
        Where-Object { $_.name -like "*64-bit.exe" -and $_.name -notlike "*MinGit*" -and $_.name -notlike "*Portable*" } |
        Select-Object -First 1
    if (-not $gitAsset) {
        throw "Could not find Git for Windows 64-bit installer asset."
    }
    $gitInstaller = Join-Path $InstallDir $gitAsset.name
    Download-File -Uri $gitAsset.browser_download_url -OutFile $gitInstaller
    Run-Installer -FilePath $gitInstaller -ArgumentList @("/VERYSILENT", "/NORESTART", "/NOCANCEL", "/SP-")
}
Add-UserPath -PathToAdd "C:\Program Files\Git\cmd"

Write-Step "Installing Node.js LTS"
$nodeFromPath = Get-Command node -ErrorAction SilentlyContinue
if ($nodeFromPath -and $nodeFromPath.Source -notlike "*WindowsApps*") {
    Write-Host "Node.js is already available at $($nodeFromPath.Source)."
}
else {
    $nodeIndex = Invoke-RestMethod -Uri "https://nodejs.org/dist/index.json"
    $nodeLts = $nodeIndex | Where-Object { $_.lts -ne $false } | Select-Object -First 1
    $nodeVersion = $nodeLts.version
    $nodeZipName = "node-$nodeVersion-win-x64.zip"
    $nodeZipUrl = "https://nodejs.org/dist/$nodeVersion/$nodeZipName"
    $nodeZip = Join-Path $InstallDir $nodeZipName
    $nodeExtractDir = Join-Path $UserToolsDir "node-$nodeVersion-win-x64"
    Download-File -Uri $nodeZipUrl -OutFile $nodeZip
    if (-not (Test-Path $nodeExtractDir)) {
        Expand-Archive -Path $nodeZip -DestinationPath $UserToolsDir -Force
    }
    Add-UserPath -PathToAdd $nodeExtractDir
}

Write-Step "Installing Rust stable MSVC toolchain"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path (Join-Path $cargoBin "rustc.exe")) {
    Write-Host "Rust is already installed."
}
else {
    $rustupInstaller = Join-Path $InstallDir "rustup-init.exe"
    Download-File -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInstaller
    Run-Installer -FilePath $rustupInstaller -ArgumentList @("-y", "--default-toolchain", "stable-x86_64-pc-windows-msvc")
}
Add-UserPath -PathToAdd $cargoBin

Write-Step "Installing WebView2 Runtime"
$webViewInstaller = Join-Path $InstallDir "MicrosoftEdgeWebView2RuntimeInstallerX64.exe"
Download-File -Uri "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile $webViewInstaller
Run-Installer -FilePath $webViewInstaller -ArgumentList @("/silent", "/install")

if (-not $SkipVisualStudioBuildTools) {
    Write-Step "Installing Visual Studio 2022 Build Tools with C++ workload"
    $vsInstaller = Join-Path $InstallDir "vs_BuildTools.exe"
    Download-File -Uri "https://aka.ms/vs/17/release/vs_BuildTools.exe" -OutFile $vsInstaller
    Run-Installer -FilePath $vsInstaller -ArgumentList @(
        "--quiet",
        "--wait",
        "--norestart",
        "--nocache",
        "--add",
        "Microsoft.VisualStudio.Workload.VCTools",
        "--includeRecommended"
    )
}

Write-Step "Refreshing PATH for this PowerShell session"
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$env:Path = "$machinePath;$userPath"

Write-Step "Installed tool versions"
git --version
node -v
npm -v
rustc -V
cargo -V

Write-Host ""
Write-Host "MailDock dev tools are installed. Restart Codex or open a new PowerShell before running npm install." -ForegroundColor Green
