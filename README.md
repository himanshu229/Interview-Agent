# AI Desktop Assistant

A native, cross-platform desktop AI assistant built with **Tauri 2 + React + TypeScript + Rust**. It brings ChatGPT to your desktop with screenshot OCR, live speech transcription, global shortcuts, a system tray, and secure local storage.

![status](https://img.shields.io/badge/status-production--ready-brightgreen) ![tauri](https://img.shields.io/badge/Tauri-2-blue) ![license](https://img.shields.io/badge/license-MIT-green)

---

## 1. Prerequisites

Install these before you start:

- **Node.js** ≥ 18 (includes npm)
- **Rust** ≥ 1.77 — install via [rustup](https://rustup.rs)
- **Tesseract OCR** (required for OCR) — must be on your `PATH`

Platform toolchains:

- **Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/), [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on Win 11), and Tesseract from <https://github.com/UB-Mannheim/tesseract/wiki>
- **macOS:** `xcode-select --install`, then `brew install tesseract`
- **Linux (Debian/Ubuntu):**
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    libwebkit2gtk-4.1-dev build-essential curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
    tesseract-ocr libasound2-dev
  ```

> For **system-audio** transcription, install a loopback device: BlackHole (macOS), VB-CABLE (Windows), or a PulseAudio/PipeWire monitor source (Linux).

---

## 2. Install

Run the one-time setup (installs dependencies and generates app icons):

```bash
# macOS / Linux
./scripts/setup.sh
```

```powershell
# Windows (PowerShell)
./scripts/setup.ps1
```

Or do it manually:

```bash
npm install
npm run tauri icon assets/icon.svg
```

---

## 3. Run (development)

```bash
npm run tauri:dev
```

On first launch, open **Settings → OpenAI API key**, paste your key, and click **Test**. The key is stored securely in your OS keychain.

---

## 4. Build (production)

| Platform | Command | Output |
| --- | --- | --- |
| Windows | `npm run build:windows` | `.exe` (NSIS) + `.msi` (WiX) |
| macOS | `npm run build:macos` | `.dmg` + `.app` |
| Linux | `npm run build:linux` | `.deb` + `.AppImage` |

Or build for the current OS with `npm run tauri:build`. You can also use the scripts:

```bash
# macOS / Linux
./scripts/build.sh
```

```powershell
# Windows (PowerShell)
./scripts/build.ps1
```

---

## 5. Create the executable / installer

After building, the installers and executables are written to:

```
src-tauri/target/release/bundle/
```

You will find:

```
bundle/
├── nsis/AI Desktop Assistant_1.0.0_x64-setup.exe   # Windows installer (.exe)
├── msi/AI Desktop Assistant_1.0.0_x64_en-US.msi    # Windows installer (.msi)
├── dmg/AI Desktop Assistant_1.0.0_aarch64.dmg      # macOS disk image
├── macos/AI Desktop Assistant.app                  # macOS app bundle
├── deb/ai-desktop-assistant_1.0.0_amd64.deb        # Linux .deb
└── appimage/ai-desktop-assistant_1.0.0_amd64.AppImage
```

- **Windows `.exe`:** run `npm run build:windows`, then use `bundle/nsis/…-setup.exe`.
- **Windows `.msi`:** use `bundle/msi/….msi`.
- **macOS `.dmg`:** run `npm run build:macos`, then open `bundle/dmg/….dmg` and drag the app to Applications.

For prerequisites, development, code signing, notarization, and auto-update configuration, see [docs/INSTALLATION.md](docs/INSTALLATION.md).

---

## 6. Useful commands

| Command | Description |
| --- | --- |
| `npm run tauri:dev` | Run the app with hot reload |
| `npm run build` | Type-check + build the frontend only |
| `npm run tauri:build` | Build all bundles for the current OS |
| `npm run build:windows` | Build Windows `.exe` + `.msi` |
| `npm run build:macos` | Build macOS `.dmg` + `.app` |
| `npm run build:linux` | Build Linux `.deb` + `.AppImage` |
| `npm run lint` | Lint TypeScript sources |
| `npm run format` | Format sources with Prettier |

---

## License

MIT © 2026 AI Desktop Assistant
