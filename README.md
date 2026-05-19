# compress[pdf]

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)

<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" alt="compress[pdf] icon" />
</p>

**Compress PDFs locally. No uploads, no limits, no nonsense.**

<p align="center">
  <img src="docs/screenshots/main-window.png" width="720" alt="compress[pdf] main window" />
</p>

## What it does

**compress[pdf]** compresses PDF files entirely on your Mac — no account, no internet connection, no file size limits. Everything stays on your machine; nothing is uploaded anywhere.

## Download

Download the latest release from the [Releases page](https://github.com/JBolanle/PDFCompressor/releases).

### First launch

compress[pdf] is an unsigned open-source app — macOS will block it the first time you open it. To allow it:

1. Open the DMG and drag the app to Applications
2. Double-click the app — macOS shows "Apple cannot verify this app", click **Done**
3. Open **System Settings → Privacy & Security**, scroll down, click **Open Anyway**
4. Click **Open** in the confirmation dialog — the app opens and is trusted from then on

Or if you prefer the terminal:

```bash
xattr -dr com.apple.quarantine "/Applications/compress[pdf].app"
```

## Using the app

1. Add files via drag-and-drop or the File menu
2. Pick a compression preset
3. Optionally override the DPI per file
4. Hit Compress — size savings are shown per file when done

**Presets:**

| Preset   | Quality        | Default DPI |
|----------|----------------|-------------|
| Max      | Smallest file  | 72          |
| Balanced | Good quality   | 150         |
| Minimal  | Near-lossless  | 300         |

Output goes to the same folder with a `_compressed` suffix, or to a custom folder of your choosing.

## Finder integration

You can compress PDFs without opening the app — pick whichever route fits your workflow:

- **Right-click → Open With → compress[pdf]** — works out of the box once the app is in `/Applications`. macOS lists compress[pdf] alongside Preview and any other PDF viewers you have installed.
- **Right-click → Compress with compress[pdf]** (Quick Action) — open the app, expand **Advanced** in the side panel, and click **Install**. The entry appears at the top of Finder's right-click menu, no submenu nesting.
- **Drop PDFs onto the Dock icon** — drag one or more PDFs onto the app icon in the Dock to start compression.

All three routes run in the background — no window appears — and use your saved Output, Naming, and Default preset settings. A native notification reports total bytes saved when the run finishes.

## For developers

**Stack:**
- Frontend: SvelteKit + Svelte 5 + TypeScript
- Backend: Rust via Tauri 2
- Compression engine: Ghostscript (bundled sidecar binary — no system install required)

**Requirements:**
- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)

```bash
# Install and run dev server
npm install
npm run tauri dev

# Build for production
npm run tauri build

# Test
npm test                        # Frontend (Vitest)
cd src-tauri && cargo test      # Rust
```

## License

Licensed under the [GNU Affero General Public License v3.0](LICENSE).

This software bundles [Ghostscript](https://www.ghostscript.com/), Copyright © Artifex Software, Inc., licensed under AGPL v3. Source code is available at https://github.com/ArtifexSoftware/ghostpdl
