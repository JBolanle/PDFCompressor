# Distribution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable automated unsigned macOS universal binary releases via GitHub Actions, publishing a single DMG (aarch64 + x86_64) to GitHub Releases on every version tag push.

**Architecture:** Both GS sidecar binaries are committed to the repo (force-added past the gitignore). A GitHub Actions workflow triggers on `v*` tags, compiles both Rust arches via `universal-apple-darwin`, stitches them with `lipo`, and creates a draft GitHub Release with the DMG attached. Users get a one-time Gatekeeper bypass instruction in the README.

**Tech Stack:** GitHub Actions, Tauri 2 CLI (`universal-apple-darwin` target), Rust cross-compilation, `softprops/action-gh-release@v2`, `Swatinem/rust-cache@v2`, Homebrew bottle extraction.

---

### Task 1: Source and commit both GS sidecar binaries

**Context:** The GS binaries in `src-tauri/binaries/` are excluded by `.gitignore` (`src-tauri/binaries/gs-*`). Both need to be force-added so CI can access them. The arm64 binary already exists locally; the x86_64 binary must be sourced from a Homebrew bottle.

**Files:**
- Force-add: `src-tauri/binaries/gs-aarch64-apple-darwin` (exists, not yet tracked)
- Create: `src-tauri/binaries/gs-x86_64-apple-darwin` (source from Homebrew bottle)

- [ ] **Step 1: Download the x86_64 Ghostscript bottle**

Run from the project root:

```bash
brew fetch --bottle-tag=sequoia ghostscript
```

Expected: Homebrew downloads a `.tar.gz` bottle to the cache. If `sequoia` fails (tag not found), retry with `sonoma` or `ventura` — replace the tag in all subsequent commands.

- [ ] **Step 2: Extract the binary from the bottle**

```bash
BOTTLE=$(brew --cache --bottle-tag=sequoia ghostscript)
echo "Bottle: $BOTTLE"

rm -rf /tmp/gs-x86-extract
mkdir -p /tmp/gs-x86-extract
tar -xzf "$BOTTLE" -C /tmp/gs-x86-extract

GS_BIN=$(find /tmp/gs-x86-extract -name 'gs' -type f | head -1)
echo "Found binary: $GS_BIN"
```

Expected: `GS_BIN` resolves to a path like `/tmp/gs-x86-extract/ghostscript/10.x.x/bin/gs`.

- [ ] **Step 3: Verify the binary is x86_64**

```bash
file "$GS_BIN"
```

Expected output must include: `Mach-O 64-bit executable x86_64`

If it says `arm64`, the wrong bottle tag was used — go back to Step 1 with a different tag (non-`arm64_*` tags are x86_64).

- [ ] **Step 4: Copy binary into the project**

```bash
cp "$GS_BIN" src-tauri/binaries/gs-x86_64-apple-darwin
chmod +x src-tauri/binaries/gs-x86_64-apple-darwin
file src-tauri/binaries/gs-x86_64-apple-darwin
```

Expected: `src-tauri/binaries/gs-x86_64-apple-darwin: Mach-O 64-bit executable x86_64`

- [ ] **Step 5: Force-add both binaries and commit**

The gitignore blocks `src-tauri/binaries/gs-*`, so use `-f`:

```bash
git add -f src-tauri/binaries/gs-aarch64-apple-darwin
git add -f src-tauri/binaries/gs-x86_64-apple-darwin
git status
```

Expected: both files shown as staged new files.

```bash
git commit -m "chore: commit GS sidecar binaries for CI distribution build"
```

---

### Task 2: Create GitHub Actions release workflow

**Context:** The workflow runs on a macOS arm64 runner. Tauri's `universal-apple-darwin` target compiles both `aarch64-apple-darwin` and `x86_64-apple-darwin` Rust targets and stitches them into a fat binary. The product name `compress[pdf]` contains glob-special characters `[` and `]`, so the DMG path is resolved with `find` rather than a glob pattern.

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Create the workflows directory and file**

```bash
mkdir -p .github/workflows
```

Create `.github/workflows/release.yml` with this exact content:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build-macos-universal:
    runs-on: macos-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin

      - name: Cache Rust build artifacts
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: 'npm'

      - name: Install npm dependencies
        run: npm ci

      - name: Build universal app
        run: npm run tauri build -- --target universal-apple-darwin

      - name: Find DMG path
        id: dmg
        run: |
          DMG_PATH=$(find src-tauri/target/universal-apple-darwin/release/bundle/dmg -name "*.dmg" -type f | head -1)
          echo "path=$DMG_PATH" >> "$GITHUB_OUTPUT"
          echo "DMG: $DMG_PATH"

      - name: Create draft GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          draft: true
          files: ${{ steps.dmg.outputs.path }}
```

- [ ] **Step 2: Verify the YAML syntax**

```bash
# Requires actionlint if available, else use python yaml check
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML valid"
```

Expected: `YAML valid`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow for universal macOS DMG"
```

---

### Task 3: Add Gatekeeper bypass instructions to README

**Context:** The app is unsigned, so macOS will quarantine it on first download. Users need a one-time bypass. The README's `## Download` section currently reads: `Download the latest release from the [Releases page](https://github.com/JBolanle/PDFCompressor/releases).` — add a `### First launch` subsection immediately after it.

**Files:**
- Modify: `README.md` (lines 15–18)

- [ ] **Step 1: Add First Launch section to README**

In `README.md`, after line 17 (`Download the latest release from...`), insert the following block. The inner code fence uses backticks — when editing the file directly, use three backticks as usual.

    ### First launch

    compress[pdf] is an unsigned open-source app — macOS will block it the first time you open it. To allow it:

    1. Open the DMG and drag the app to Applications
    2. Double-click the app — macOS shows "Apple cannot verify this app", click **Done**
    3. Open **System Settings → Privacy & Security**, scroll down, click **Open Anyway**
    4. Click **Open** in the confirmation dialog — the app opens and is trusted from then on

    Or if you prefer the terminal:

        xattr -dr com.apple.quarantine "/Applications/compress[pdf].app"

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add Gatekeeper bypass instructions for unsigned app"
```

---

### Task 4: Smoke-test the release workflow

**Context:** Verify the full pipeline works end-to-end by pushing a test tag. The release will be a draft, so it won't be publicly visible until you publish it.

- [ ] **Step 1: Push the current state**

```bash
git push
```

- [ ] **Step 2: Tag and push**

```bash
git tag v1.2.0
git push origin v1.2.0
```

- [ ] **Step 3: Watch the Actions run**

Go to `https://github.com/JBolanle/PDFCompressor/actions` — a `Release` workflow run should appear within seconds of the tag push. Watch for it to complete (~15–25 minutes).

Expected: green check, a draft release appears at `https://github.com/JBolanle/PDFCompressor/releases` with a `.dmg` attached.

- [ ] **Step 4: Verify the DMG**

Download the draft DMG. Double-click to mount it, then in Terminal:

```bash
lipo -info "/Volumes/compress[pdf]/compress[pdf].app/Contents/MacOS/compress[pdf]"
```

Expected output includes: `Architectures in the fat file: ... are: x86_64 arm64`

If the volume or binary name differs, tab-complete in Terminal after `/Volumes/` to find the exact name.

- [ ] **Step 5: Add release notes and publish the draft**

On the GitHub Releases page, open the draft, write release notes, and click **Publish release**.
