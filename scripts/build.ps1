#Requires -Version 5.1
# ---------------------------------------------------------------------------
# Production build for Windows (.exe via NSIS + .msi via WiX).
# ---------------------------------------------------------------------------
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "==> Type-checking and building the frontend"
npm run build

Write-Host "==> Building Windows installers (NSIS .exe + .msi)"
npm run tauri build -- --bundles nsis,msi

Write-Host "==> Artifacts in src-tauri\target\release\bundle\"
Write-Host "     NSIS installer: bundle\nsis\*-setup.exe"
Write-Host "     MSI installer:  bundle\msi\*.msi"
