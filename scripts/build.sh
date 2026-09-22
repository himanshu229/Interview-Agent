#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Production build for macOS (.dmg + .app) and Linux (.deb + .AppImage).
# ---------------------------------------------------------------------------
set -euo pipefail

cd "$(dirname "$0")/.."

OS="$(uname -s)"

echo "==> Type-checking and building the frontend"
npm run build

if [ "$OS" = "Darwin" ]; then
  echo "==> Building macOS bundles (.app + .dmg)"
  npm run tauri build -- --bundles app,dmg
  echo "==> Artifacts in src-tauri/target/release/bundle/"
elif [ "$OS" = "Linux" ]; then
  echo "==> Building Linux bundles (.deb + .AppImage)"
  npm run tauri build -- --bundles deb,appimage
  echo "==> Artifacts in src-tauri/target/release/bundle/"
else
  echo "Unsupported OS for this script: $OS"
  exit 1
fi
