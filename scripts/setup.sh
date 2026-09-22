#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# One-time project setup: install dependencies and generate app icons.
# ---------------------------------------------------------------------------
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Installing npm dependencies"
npm install

echo "==> Clearing old generated icons"
rm -rf src-tauri/icons

echo "==> Generating application icons from assets/icon.svg"
npm run tauri icon assets/icon.svg

echo "==> Checking for tesseract (OCR)"
if ! command -v tesseract >/dev/null 2>&1; then
  echo "WARNING: 'tesseract' not found on PATH."
  echo "  macOS:  brew install tesseract"
  echo "  Ubuntu: sudo apt-get install tesseract-ocr"
fi

echo "==> Setup complete. Run 'npm run tauri:dev' to start."
