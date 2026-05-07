# Distribution Design

**Date:** 2026-05-07  
**Status:** Approved

## Context

compress[pdf] v1.2.0 is a macOS-only Tauri app. Currently the only way to obtain it is to clone the repo and run `npm run tauri build` locally. The README already links to a GitHub Releases page that has no published releases. This spec covers making that Releases page the real distribution point.

**Constraints:**
- No paid Apple Developer account — notarization and Developer ID signing are off the table for now
- Target: both Apple Silicon (aarch64) and Intel (x86_64) Macs
- Distribution channel: GitHub Releases (public repo, free GitHub Actions)

## What We're Building

Three things:

1. **Universal binary build** — a single DMG containing a fat binary (aarch64 + x86_64) built via Tauri's `universal-apple-darwin` target
2. **GitHub Actions release workflow** — automated build and draft release on every `v*` tag push
3. **README updates** — Gatekeeper bypass instructions for users opening an unsigned app

## Intel GS Binary

The repo currently has `src-tauri/binaries/gs-aarch64-apple-darwin`. A matching `gs-x86_64-apple-darwin` must be added and committed.

**How to source it:** Use Homebrew's bottle download on the existing arm64 Mac:

```bash
brew fetch --bottle-tag=ventura ghostscript
```

This downloads a `.tar.gz` bottle. Extract it, locate the `gs` binary inside, and copy it to `src-tauri/binaries/gs-x86_64-apple-darwin`. Make it executable (`chmod +x`), then commit it.

The binary is ~15–20 MB — committing it directly is appropriate and consistent with the existing arm64 binary already in the repo.

## GitHub Actions Workflow

**File:** `.github/workflows/release.yml`

**Trigger:** Push of any tag matching `v*` (e.g. `v1.2.0`)

**Runner:** `macos-latest` (Apple Silicon, macOS 15)

**Steps:**
1. Checkout repo
2. Install stable Rust toolchain, add targets `aarch64-apple-darwin` and `x86_64-apple-darwin`
3. Install Node.js 22 with npm cache
4. `npm ci`
5. `npm run tauri build -- --target universal-apple-darwin`
   - Tauri compiles both Rust targets and uses `lipo` to produce a fat binary
   - Output: `src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg`
6. Create a draft GitHub Release named after the tag, upload the DMG

**Release is created as a draft** so release notes can be written before publishing.

**Build time:** ~15–25 minutes (compiling Rust twice).

**No signing steps** — the app ships unsigned.

## Gatekeeper Bypass

Unsigned apps are quarantined by macOS on download. Users need a one-time bypass on first launch.

**Steps for users (GUI):**
1. Download the DMG, drag the app to Applications
2. Double-click the app → macOS shows "Apple cannot verify this app"
3. Open **System Settings → Privacy & Security**, scroll down, click **Open Anyway**
4. Confirm → app is trusted from then on

**For technical users (terminal):**
```bash
xattr -dr com.apple.quarantine /Applications/compress\[pdf\].app
```

## README Changes

- Add a **First Launch** subsection under the existing Download section
- Include GUI steps and terminal one-liner above
- Add a one-sentence explanation: the app is unsigned open-source software and does not send files anywhere

## Release Process (for the developer)

1. Update `version` in `src-tauri/tauri.conf.json` and `package.json`
2. `git commit -m "chore: bump version to X.Y.Z"`
3. `git tag vX.Y.Z`
4. `git push && git push --tags`
5. CI builds and creates a draft release — add release notes and publish

## Files Created / Modified

| File | Change |
|------|--------|
| `src-tauri/binaries/gs-x86_64-apple-darwin` | New — Intel GS binary |
| `.github/workflows/release.yml` | New — release CI workflow |
| `README.md` | Add First Launch / Gatekeeper section |
