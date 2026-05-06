# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

All commands run from the project root (`compress[pdf]/`) unless noted.

| Task | Command |
|------|---------|
| Dev server (Tauri + frontend) | `npm run tauri dev` |
| Production build | `npm run tauri build` |
| Frontend tests (one-shot) | `npm test` |
| Frontend tests (watch) | `npm run test:watch` |
| Single frontend test file | `npx vitest run src/test/ActionBar.test.ts` |
| Type check | `npm run check` |
| Rust tests | `cargo test` (from `src-tauri/`) |
| Single Rust test | `cargo test test_name` (from `src-tauri/`) |
| Integration test (needs system GS) | `cargo test compress_integration -- --ignored` (from `src-tauri/`) |

## Architecture

**Two-layer app**: SvelteKit frontend + Rust/Tauri backend communicating via `invoke()` calls and `listen()` events.

### Frontend (`src/`)

Single-page layout (`src/routes/+page.svelte`): three-panel UI — `Sidebar` (file queue list) | `DetailPanel` (per-file settings) — with `ActionBar` at the bottom and `Toast` overlay.

Four Svelte stores coordinate all state:
- `queueStore` — file list, per-file status/preset/sizes; deduplicates by path
- `settingsStore` — output mode + naming settings, persisted via `invoke("get_settings")` / `invoke("save_settings")`
- `selectionStore` — which file is selected in the sidebar
- `toastStore` — ephemeral notification messages

### Backend (`src-tauri/src/`)

Tauri commands registered in `lib.rs::run()`:
- `compress_files` — iterates jobs, shells out to GS sidecar, emits `compress:progress` events (`processing` → `done`/`error`) per file
- `get_settings` / `save_settings` — JSON persistence to Tauri app-data dir
- `reveal_in_finder` — `open -R` wrapper
- `get_file_meta` / `validate_pdf` / `check_path_writable_cmd` — utility commands

Module responsibilities:
- `compress.rs` — `build_gs_args()` (preset→GS flags), `compress_files` command, async `run_gs()` sidecar runner
- `path_resolver.rs` — `resolve_output_path()` implements the 2×2 output matrix (same-as-source/custom-folder × suffix/overwrite)
- `settings.rs` — `Settings` struct, file-based persistence helpers
- `finder.rs` — Reveal in Finder

### Ghostscript sidecar

GS binary is bundled at `src-tauri/binaries/gs-{target-triple}` and declared in `tauri.conf.json` under `bundle.externalBin`. The binary must be executable and named for the exact Rust target triple (`rustc -vV | grep host`). A Cargo test (`sidecar_tests::gs_binary_exists_for_current_arch`) verifies this at test time.

Compression pipeline: GS writes to a `.pdf.tmp` file first, then `fs::rename` atomically replaces the final output path.

### Testing conventions

- Rust: tests are inline in each module (`#[cfg(test)]`). All pure functions are covered; `#[tauri::command]` functions that need `AppHandle` are excluded from unit tests.
- Frontend: Vitest tests in `src/test/`. Store tests (`queueStore`, `settingsStore`) use the `node` environment; component tests use `happy-dom`. `@tauri-apps/plugin-dialog` is mocked via a vitest alias in `vitest.config.ts`.
- Always write the failing test first before implementing any feature or fix.
