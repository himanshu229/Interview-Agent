# Installation, Development & Build Guide

Everything you need to **install prerequisites**, **run in development**, and
**generate production builds** for **AI Desktop Assistant**.

---

## 1. Prerequisites

### Common

| Tool | Version | Notes |
| --- | --- | --- |
| Node.js | ≥ 18 | includes npm |
| Rust | ≥ 1.77 | install via [rustup](https://rustup.rs) |
| Tesseract OCR | ≥ 5 | required for OCR, must be on `PATH` |

### Platform toolchains

**Windows**
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++)
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on Windows 11)
- LLVM/Clang (for the screen-capture dependency) — <https://releases.llvm.org/>
- Tesseract: <https://github.com/UB-Mannheim/tesseract/wiki>
- For **system-audio** capture: install a loopback device such as [VB-CABLE](https://vb-audio.com/Cable/).

**macOS**
- Xcode Command Line Tools: `xcode-select --install`
- Tesseract: `brew install tesseract`
- For **system-audio** capture: install [BlackHole](https://existential.audio/blackhole/) and select it as an input.
- Grant **Screen Recording** and **Microphone** permissions in *System Settings → Privacy & Security*.

**Linux (Debian/Ubuntu)**
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  libclang-dev libxcb1-dev libxrandr-dev libdbus-1-dev \
  libpipewire-0.3-dev libwayland-dev libegl-dev \
  tesseract-ocr libasound2-dev
```
For system audio, use the PulseAudio/PipeWire `monitor` source.

---

## 2. Install (one-time setup)

Installs JS dependencies and generates the app icons from `assets/icon.svg`.

```bash
# macOS / Linux
bash scripts/setup.sh
```
```powershell
# Windows (PowerShell)
./scripts/setup.ps1
```

Or manually:
```bash
npm install
npm run tauri icon assets/icon.svg
```

> If `cargo` is not found afterwards, load it into your shell with
> `. "$HOME/.cargo/env"` (rustup also adds it to your shell profile).

---

## 3. Development (hot reload)

```bash
# optional: copy env defaults
cp .env.example .env

# start the app with hot reload (frontend + Rust backend)
npm run tauri:dev

"$HOME/.cargo/env" && npm run tauri:dev
```

- Vite serves the UI on `http://localhost:1420`; the Rust backend recompiles on
  change.
- On first launch, open the **⋮ menu → Advanced settings**, paste your OpenAI
  API key, and click **Test**. The key is stored in the OS keychain.
- Frontend-only type check: `npm run build`.
- If port 1420 is busy: `lsof -ti:1420 | xargs kill -9` (macOS/Linux).

---

## 4. Production build (generate the installers/executables)

Run the build for your platform. Each command **generates the specific
installer/executable files** listed below.

| Platform | Command | Generated files |
| --- | --- | --- |
| Windows | `npm run build:windows` | `.exe` (NSIS) + `.msi` (WiX) |
| macOS | `npm run build:macos` | `.dmg` + `.app` |
| Linux | `npm run build:linux` | `.deb` + `.AppImage` |

Or build for the current OS with `npm run tauri:build`. Scripts are also
provided: `bash scripts/build.sh` (macOS/Linux) and `scripts/build.ps1`
(Windows).

### Where the generated files are written

```
src-tauri/target/release/bundle/
├── nsis/AI Desktop Assistant_1.0.0_x64-setup.exe   # Windows installer (.exe)
├── msi/AI Desktop Assistant_1.0.0_x64_en-US.msi    # Windows installer (.msi)
├── dmg/AI Desktop Assistant_1.0.0_aarch64.dmg      # macOS disk image
├── macos/AI Desktop Assistant.app                  # macOS app bundle
├── deb/ai-desktop-assistant_1.0.0_amd64.deb        # Linux .deb
└── appimage/ai-desktop-assistant_1.0.0_amd64.AppImage
```

- **Windows `.exe`:** `bundle/nsis/…-setup.exe`
- **Windows `.msi`:** `bundle/msi/….msi`
- **macOS `.dmg`:** open `bundle/dmg/….dmg` and drag the app to Applications

---

## 5. Icons

App icons are generated from `assets/icon.svg`:

```bash
npm run tauri icon assets/icon.svg
```

This writes `32x32.png`, `128x128.png`, `icon.icns`, `icon.ico`, etc. into
`src-tauri/icons/`. The setup script runs this automatically.

---

## 6. Code signing & notarization

### Windows (Authenticode)
```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "<base64 pfx or path>"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<password>"
```
Or configure `bundle.windows.certificateThumbprint` in `tauri.conf.json`.

### macOS (Developer ID + notarization)
```bash
export APPLE_CERTIFICATE="<base64 .p12>"
export APPLE_CERTIFICATE_PASSWORD="<password>"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="<app-specific-password>"
export APPLE_TEAM_ID="TEAMID"
npm run build:macos
```

---

## 7. Auto-updates

1. Generate an updater key pair:
   ```bash
   npm run tauri signer generate -- -w ~/.tauri/ai-desktop-assistant.key
   ```
2. Put the **public key** into `plugins.updater.pubkey` in
   `src-tauri/tauri.conf.json`.
3. Set the update server URL in `plugins.updater.endpoints`.
4. Sign releases by exporting the private key before building:
   ```bash
   export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/ai-desktop-assistant.key)"
   export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="<password>"
   ```
5. Upload the generated `latest.json` and installers to your endpoint.

---

## 8. Troubleshooting

| Symptom | Fix |
| --- | --- |
| `cargo: command not found` | `. "$HOME/.cargo/env"` or reinstall via [rustup](https://rustup.rs) |
| `Port 1420 is already in use` | `lsof -ti:1420 \| xargs kill -9` then re-run |
| `tesseract failed … Is tesseract installed` | Install Tesseract and ensure it is on `PATH` |
| OCR returns wrong language | Install the language pack and set it in the transcript language selector |
| No system-audio transcript | Install/select a loopback device (BlackHole / VB-CABLE / monitor source) |
| Microphone empty on macOS | Grant Microphone permission in System Settings |
| Global shortcut not working | Ensure no other app owns it; restart after changing it |
| `keyring` errors on Linux | Install `gnome-keyring` or another Secret Service provider |

---

## 9. Useful commands

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
