# AI Desktop Assistant — Project Context & Flow

> **Purpose of this file:** A single source of truth for understanding this
> codebase. If you start a fresh chat/session, read this file first to
> understand the architecture, data flow, and where everything lives. Keep it
> updated when the design changes.

---

## 1. What this app is

A **native cross-platform desktop AI assistant** built with **Tauri 2 + React +
TypeScript + Rust**. It provides:

- ChatGPT chat (OpenAI Chat Completions API)
- A top "ask" search box on every tab
- Screenshot capture (full screen + mouse-drag region selection)
- OCR text extraction (Tesseract) with optional auto-send to ChatGPT
- Microphone speech-to-text and system-audio transcription (OpenAI Whisper)
- Live transcript panel
- Conversation history in SQLite
- Global keyboard shortcut, system tray, always-on-top, light/dark themes
- Secure API-key storage in the OS keychain, logging, and auto-updates

---

## 2. Tech stack

| Layer | Choice |
| --- | --- |
| Frontend | React 18, TypeScript, Vite |
| Backend | Rust, Tauri 2 |
| Database | SQLite (`rusqlite`, bundled — no external install) |
| OCR | Tesseract via `rusty-tesseract` (needs `tesseract` on PATH) |
| Speech-to-text | OpenAI Whisper API |
| AI chat | OpenAI Chat Completions API |
| Audio capture | `cpal` + `hound` (WAV encode) |
| Screen capture | `xcap` |
| Secrets | OS keychain via `keyring` |

---

## 3. Directory map (what lives where)

```
ai-desktop-assistant/
├── index.html                  # Main window HTML entry
├── overlay.html                # Region-selection overlay HTML entry
├── vite.config.ts              # TWO entry points: main + overlay; @ alias -> src
├── package.json                # scripts: tauri:dev, build:windows/macos/linux
├── .env.example                # dev-only defaults (real key goes in keychain)
├── assets/icon.svg             # source icon -> `npm run tauri icon` generates all
├── docs/                       # <-- YOU ARE HERE
├── scripts/                    # setup.sh/ps1 (install+icons), build.sh/ps1
│
├── src/                        # FRONTEND (React)
│   ├── main.tsx                # mounts App inside Settings+Theme providers
│   ├── App.tsx                 # shell: TitleBar + Sidebar + SearchBox + tabs
│   ├── components/             # TitleBar, Sidebar, SearchBox
│   ├── tabs/                   # ChatTab, TranscriptTab, ScreenshotTab,
│   │                           #   HistoryTab, SettingsTab
│   ├── context/                # SettingsContext, ThemeContext
│   ├── hooks/useChat.ts        # central chat state + backend event listener
│   ├── lib/api.ts              # ALL invoke() wrappers (typed) live here
│   ├── overlay/main.tsx        # transparent drag-to-select region UI
│   ├── styles/                 # global.css (theme vars), app.css (layout)
│   └── types/index.ts          # TS types MIRRORING Rust structs
│
└── src-tauri/                  # BACKEND (Rust)
    ├── Cargo.toml              # deps + release profile
    ├── tauri.conf.json         # windows, CSP, bundles, updater config
    ├── capabilities/default.json  # permissions for main + overlay windows
    ├── entitlements.plist      # macOS mic/screen permissions
    └── src/
        ├── main.rs             # thin entry -> lib::run()
        ├── lib.rs              # app builder: plugins, setup, invoke_handler,
        │                       #   window-close-to-tray, tray + shortcut init
        ├── error.rs            # AppError (serializable) + AppResult
        ├── models.rs           # serde structs (camelCase) shared with frontend
        ├── db.rs               # SQLite schema + CRUD (conversations, messages)
        ├── state.rs            # AppState: db, settings, http, audio sessions
        ├── tray.rs             # system tray icon + menu + click handlers
        ├── shortcuts.rs        # global-shortcut plugin + register()
        ├── services/           # pure logic, no Tauri command attributes
        │   ├── openai.rs       # chat() + transcribe() + validate()
        │   ├── ocr.rs          # extract_text() via Tesseract
        │   ├── screenshot.rs   # capture_full_screen / capture_region
        │   ├── audio.rs        # AudioSession: cpal capture -> WAV chunks
        │   └── secrets.rs      # SecretStore: keychain get/set/clear
        └── commands/           # #[tauri::command] handlers (thin wrappers)
            ├── chat.rs         # send_chat
            ├── history.rs      # list/get/create/rename/delete conversations
            ├── screenshot.rs   # capture, run_ocr, start_region_selection, …
            ├── audio.rs        # start/stop mic + system transcription
            ├── settings.rs     # get/save settings, api-key mgmt, test key
            └── window.rs       # set_always_on_top, toggle_floating_window
```

**Golden rule:** frontend never calls `invoke` directly — everything goes
through `src/lib/api.ts`. Backend commands are thin; real logic lives in
`src-tauri/src/services/`.

---

## 4. Frontend ⇄ Backend contract

### 4.1 Commands (frontend → backend, via `invoke`)

Registered in `src-tauri/src/lib.rs` `invoke_handler!`. Wrapped in
`src/lib/api.ts`.

| Command | File | Purpose |
| --- | --- | --- |
| `send_chat` | commands/chat.rs | Persist user msg, call OpenAI, persist reply |
| `list_conversations` / `get_messages` / `create_conversation` / `rename_conversation` / `delete_conversation` | commands/history.rs | History CRUD |
| `capture_full_screen` / `capture_region` | commands/screenshot.rs | Screenshots |
| `start_region_selection` | commands/screenshot.rs | Opens overlay window |
| `run_ocr` / `ocr_and_maybe_send` | commands/screenshot.rs | OCR (+ auto-send) |
| `start_mic_transcription` / `stop_mic_transcription` | commands/audio.rs | Mic STT |
| `start_system_audio_transcription` / `stop_system_audio_transcription` | commands/audio.rs | System audio STT |
| `get_settings` / `save_settings` | commands/settings.rs | Settings persistence |
| `set_api_key` / `clear_api_key` / `test_api_key` | commands/settings.rs | Keychain + validation |
| `set_always_on_top` / `toggle_floating_window` | commands/window.rs | Window control |

### 4.2 Events (backend → frontend, via `emit` / `listen`)

| Event | Emitted by | Consumed by | Payload |
| --- | --- | --- | --- |
| `chat-message` | ScreenshotTab / backend OCR | `useChat` | `{ prompt, source }` → triggers `send()` |
| `ocr-sent` | ScreenshotTab / backend | App | switches to Chat tab |
| `navigate-tab` | tray.rs / shortcuts.rs | App | tab key string |
| `transcript-segment` | commands/audio.rs | TranscriptTab | `TranscriptSegment` |
| `region-captured` | overlay/main.tsx | ScreenshotTab | `ScreenshotResult` |
| `region-error` | overlay/main.tsx | ScreenshotTab | error string |

**Type sync:** `src/types/index.ts` mirrors `src-tauri/src/models.rs`. Rust
structs use `#[serde(rename_all = "camelCase")]` so field names match TS.

---

## 5. Key end-to-end flows

### 5.1 Ask a question (chat)
1. User types in `SearchBox` or `ChatTab` → `useChat.send()`.
2. `api.sendChat()` → `send_chat` command.
3. `chat.rs`: create/find conversation → insert user message (db.rs) → build
   history (system prompt + prior messages) → drop DB lock.
4. `openai.rs chat()` → OpenAI `/chat/completions`.
5. Insert assistant reply (db.rs) → return `ChatResponse`.
6. `useChat` appends reply to `messages`.

> **Concurrency note:** parking_lot mutex guards are NOT held across `.await`.
> In `send_chat`, DB work is scoped in blocks and the lock is dropped before the
> network call. Follow this pattern for any command mixing DB + network.

### 5.2 Screenshot region → OCR → ChatGPT
1. ScreenshotTab "Select area" → `start_region_selection` opens transparent
   fullscreen `overlay` window (`overlay.html` / `src/overlay/main.tsx`).
2. User drags a rectangle → overlay calls `capture_region` → emits
   `region-captured` with the PNG (path + base64) → overlay closes.
3. ScreenshotTab shows preview. "OCR → Ask ChatGPT" runs `run_ocr`
   (Tesseract) then `emit('chat-message', { prompt, source:'ocr' })`.
4. `useChat` listener sends it to ChatGPT; App switches to Chat tab on
   `ocr-sent`.

### 5.3 Live transcription (mic / system audio)
1. TranscriptTab toggle → `start_mic_transcription` /
   `start_system_audio_transcription`.
2. `audio.rs` `start_capture()` spawns an **OS thread** owning the cpal stream
   (cpal streams are `!Send`); samples are downmixed to mono into a shared
   buffer.
3. A Tauri async task drains ~4s chunks, resamples to 16 kHz, WAV-encodes,
   calls Whisper (`openai.rs transcribe()`), and emits `transcript-segment`.
4. Stop → session flag flips, thread drops the stream, trailing audio flushed.

> System audio needs a loopback device (BlackHole/VB-CABLE/PulseAudio monitor).
> `select_device()` looks for loopback/monitor/stereo-mix/blackhole names and
> falls back to default input with a helpful error otherwise.

### 5.4 Settings & API key
- Settings persisted as JSON in the app data dir (`state.rs`).
- API key stored ONLY in the OS keychain (`secrets.rs`), never on disk/db.
- `state.openai_client()` reads the key from the keychain + base URL from
  settings to build a per-call client.

### 5.5 Tray, shortcut, window lifecycle
- Tray built in `tray.rs` (Show / Screenshot / Transcript / Settings / Quit).
- Global shortcut (default `Ctrl/Cmd+Shift+Space`) toggles the main window
  (`shortcuts.rs`).
- Closing the main window **hides to tray** instead of quitting (see
  `on_window_event` in `lib.rs`).

---

## 6. State & data model

`AppState` (src-tauri/src/state.rs), managed via `app.manage(state)`:
- `db: Mutex<Database>` — SQLite connection
- `settings: RwLock<AppSettings>`
- `http: reqwest::Client` — shared, reused
- `mic_session` / `system_session: Mutex<Option<Arc<AudioSession>>>`

SQLite schema (`db.rs`):
- `conversations(id, title, created_at, updated_at)`
- `messages(id, conversation_id → conversations.id CASCADE, role, content,
  source, created_at)`

---

## 7. Build, run, and gotchas

### Run in dev
```bash
./scripts/setup.sh        # npm install + `tauri icon` (REQUIRED before first build)
npm run tauri:dev
```

### Production bundles
```bash
npm run build:windows     # .exe (NSIS) + .msi (WiX)
npm run build:macos       # .dmg + .app
npm run build:linux       # .deb + .AppImage
```
Artifacts: `src-tauri/target/release/bundle/`.

### Prerequisites / common gotchas
- **Icons must be generated** (`npm run tauri icon assets/icon.svg`) or the
  build fails — the setup script does this.
- **Tesseract** must be installed and on PATH for OCR.
- **System audio** requires a loopback driver.
- **Updater** `pubkey` + `endpoints` in `tauri.conf.json` are PLACEHOLDERS —
  replace before shipping signed releases.
- Signing/notarization steps are in `docs/INSTALLATION.md`.

---

## 8. How to extend (playbook)

- **Add a backend command:**
  1. Write logic in a `services/*.rs` module.
  2. Add a thin `#[tauri::command]` in the right `commands/*.rs`.
  3. Register it in `lib.rs` `invoke_handler!`.
  4. Add a typed wrapper in `src/lib/api.ts`.
  5. If it returns data to the UI, add/adjust a type in `src/types/index.ts`
     and mirror the Rust struct with `#[serde(rename_all = "camelCase")]`.

- **Add a new tab:** create `src/tabs/XTab.tsx`, add a `TabKey` in
  `types/index.ts`, register it in `Sidebar.tsx` and `App.tsx`.

- **Add a backend→frontend event:** `app.emit("name", payload)` in Rust,
  `listen("name", …)` in a React `useEffect` (remember cleanup).

- **Add a permission:** update `src-tauri/capabilities/default.json`.

---

## 9. Security posture (don't regress)

- API key: keychain only, never logged or persisted in plaintext.
- Strict CSP in `index.html` + `tauri.conf.json`; network limited to OpenAI.
- Tauri capabilities scope which commands/windows are exposed.
- All errors typed via `AppError` and surfaced/logged (`tauri-plugin-log`).

---

_Last updated: 2026-09-18. Update this file whenever flows, commands, events,
or structure change._
