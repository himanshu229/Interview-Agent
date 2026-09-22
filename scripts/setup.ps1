#Requires -Version 5.1
# ---------------------------------------------------------------------------
# One-time project setup for Windows: install deps and generate app icons.
# ---------------------------------------------------------------------------
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "==> Installing npm dependencies"
npm install

Write-Host "==> Clearing old generated icons"
Remove-Item -Recurse -Force "src-tauri\icons" -ErrorAction SilentlyContinue

Write-Host "==> Generating application icons from assets/icon.svg"
npm run tauri icon assets/icon.svg

Write-Host "==> Checking for tesseract (OCR)"
if (-not (Get-Command tesseract -ErrorAction SilentlyContinue)) {
    Write-Warning "'tesseract' not found on PATH."
    Write-Host "  Install from https://github.com/UB-Mannheim/tesseract/wiki"
}

Write-Host "==> Setup complete. Run 'npm run tauri:dev' to start."
