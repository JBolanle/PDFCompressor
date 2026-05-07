# compress[pdf]

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)

A local macOS desktop app for compressing PDF files using Ghostscript. No cloud, no upload — files stay on your machine.

## Stack

- **Frontend**: SvelteKit + Svelte 5 + TypeScript
- **Backend**: Rust via Tauri 2
- **Compression engine**: Ghostscript (bundled sidecar binary)

## Requirements

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- Tauri CLI: `npm install` (included as dev dependency)

The Ghostscript binary is bundled in `src-tauri/binaries/` — no system GS install required at runtime.

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Testing

```bash
# Frontend (Vitest)
npm test

# Rust
cd src-tauri && cargo test
```

## Compression presets

| Preset | GS setting | Default DPI |
|--------|-----------|-------------|
| Max | `/screen` | 72 |
| Balanced | `/ebook` | 150 |
| Minimal | `/printer` | 300 |

DPI can be overridden per file. Output can go to the same folder as the source or a custom folder, with either a `_compressed` suffix or in-place overwrite.

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE).

This software bundles [Ghostscript](https://www.ghostscript.com/), Copyright © Artifex Software, Inc., licensed under AGPL v3. Source code is available at https://github.com/JBolanle/PDFCompressor/
