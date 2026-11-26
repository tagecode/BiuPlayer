# NSIS install / launch / uninstall smoke test (Windows).
#
# Installs the built setup.exe silently into a throwaway directory, verifies the
# binary and Start Menu shortcut, launches the app once, then uninstalls and
# checks that files, shortcut and the uninstall registry key are gone.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\nsis-smoke.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\nsis-smoke.ps1 -Installer path\to\setup.exe
param(
  [string]$Installer = "",
  [string]$InstallDir = "$env:TEMP\biuplayer-nsis-smoke",
  [switch]$Keep
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$checks = @()

function Add-Check([string]$Name, [bool]$Ok, [string]$Detail = "") {
  $script:checks += [pscustomobject]@{ Name = $Name; Ok = $Ok; Detail = $Detail }
  $prefix = if ($Ok) { "ok  " } else { "FAIL" }
  Write-Host "$prefix $Name$(if ($Detail) { " — $Detail" })"
}

if (-not $Installer) {
  $candidate = Get-ChildItem "$root\src-tauri\target\release\bundle\nsis\*-setup.exe" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
  if (-not $candidate) {
    Write-Error "no NSIS installer found; run: pnpm tauri build --bundles nsis"
    exit 1
  }
  $Installer = $candidate.FullName
}
if (-not (Test-Path $Installer)) {
  Write-Error "installer not found: $Installer"
  exit 1
}
Write-Host "installer: $Installer"

if ($Keep) {
  Write-Host "keeping $InstallDir"
} elseif (Test-Path $InstallDir) {
  Remove-Item -Recurse -Force $InstallDir
}

$shortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\BiuPlayer.lnk"
$uninstallKeys = @(
  "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\BiuPlayer",
  "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\com.tagecode.biuplayer"
)
function Get-UninstallKey {
  foreach ($key in $uninstallKeys) {
    if (Test-Path $key) { return $key }
  }
  return $null
}

# --- install -----------------------------------------------------------------
$install = Start-Process -FilePath $Installer -ArgumentList "/S", "/D=$InstallDir" -PassThru -Wait
Add-Check "installer exits 0" ($install.ExitCode -eq 0) "exit=$($install.ExitCode)"

$exe = Join-Path $InstallDir "BiuPlayer.exe"
Add-Check "app binary installed" (Test-Path $exe) $exe
$uninstaller = Join-Path $InstallDir "uninstall.exe"
Add-Check "uninstaller installed" (Test-Path $uninstaller) $uninstaller
Add-Check "Start Menu shortcut created" (Test-Path $shortcut) $shortcut
$key = Get-UninstallKey
Add-Check "uninstall registry key created" ([bool]$key) $(if ($key) { $key } else { "missing" })

# Keep app data out of %APPDATA% while the smoke runs.
$portable = Join-Path $InstallDir "portable.flag"
if (Test-Path $exe) { New-Item -ItemType File -Force -Path $portable | Out-Null }

# --- launch ------------------------------------------------------------------
if (Test-Path $exe) {
  $app = Start-Process -FilePath $exe -PassThru
  Start-Sleep -Seconds 6
  $running = -not $app.HasExited
  Add-Check "installed app launches and stays running" $running "pid=$($app.Id)"
  if ($running) {
    $logDir = Join-Path $InstallDir "data\logs"
    $log = Get-ChildItem $logDir -Filter "biu.log*" -ErrorAction SilentlyContinue | Select-Object -First 1
    Add-Check "app writes logs into portable data dir" ([bool]$log) $(if ($log) { $log.FullName } else { $logDir })
    Stop-Process -Id $app.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 800
  }
}

# --- uninstall ---------------------------------------------------------------
if (Test-Path $uninstaller) {
  $removal = Start-Process -FilePath $uninstaller -ArgumentList "/S" -PassThru -Wait
  Add-Check "uninstaller exits 0" ($removal.ExitCode -eq 0) "exit=$($removal.ExitCode)"
  $deadline = (Get-Date).AddSeconds(20)
  while ((Test-Path $exe) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
  Add-Check "app binary removed" (-not (Test-Path $exe)) $exe
  Add-Check "Start Menu shortcut removed" (-not (Test-Path $shortcut)) $shortcut
  Add-Check "uninstall registry key removed" (-not (Get-UninstallKey)) "still present"
  if (-not (Test-Path $exe) -and -not $Keep) {
    Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
  }
} else {
  Add-Check "uninstaller available" $false "cannot verify removal"
}

$failed = @($checks | Where-Object { -not $_.Ok })
Write-Host ""
if ($failed.Count -eq 0) {
  Write-Host "nsis smoke: $($checks.Count) checks passed"
  exit 0
}
Write-Host "nsis smoke: $($failed.Count) failed"
exit 1
