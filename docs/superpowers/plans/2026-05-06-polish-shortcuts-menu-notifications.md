# Polish: Keyboard Shortcuts, App Menu, System Notifications — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a macOS app menu (File + Queue), keyboard shortcuts for all primary actions, and a system notification on batch completion.

**Architecture:** Rust builds the native menu via `tauri::menu` and emits `menu:*` events to Svelte; a new `MenuRegistry` state struct lets the frontend grey/ungrey menu items via `set_menu_item_enabled`. Keyboard shortcuts live in a pure `shortcuts.ts` module (no Tauri deps, fully testable). Notification message formatting lives in `notification.ts` (also pure, fully testable). `+page.svelte` wires everything together.

**Tech Stack:** Tauri 2, Svelte 5, TypeScript, Rust, `tauri-plugin-notification`, Vitest + Testing Library

---

## File Map

| File | Action | Purpose |
|------|--------|---------|
| `src-tauri/Cargo.toml` | Modify | Add `tauri-plugin-notification` |
| `src-tauri/capabilities/default.json` | Modify | Add `notification:default` permission |
| `src-tauri/src/menu.rs` | **Create** | Build menu, MenuRegistry, `set_menu_item_enabled` |
| `src-tauri/src/lib.rs` | Modify | Register menu module, notification plugin, new command |
| `package.json` | Modify | Add `@tauri-apps/plugin-notification` |
| `src/lib/mocks/tauri-plugin-notification.ts` | **Create** | Vitest mock for notification plugin |
| `vitest.config.ts` | Modify | Alias notification plugin to mock |
| `src/lib/notification.ts` | **Create** | Pure `buildNotificationBody` + `formatSavedBytes` |
| `src/lib/shortcuts.ts` | **Create** | Pure `handleShortcut` function |
| `src/lib/fileActions.ts` | **Create** | Shared `addFiles` / `addPath` (extracted from Sidebar) |
| `src/lib/components/Sidebar.svelte` | Modify | Import `addFiles`/`addPath` from `fileActions.ts` |
| `src/lib/components/ActionBar.svelte` | Modify | Listen for `app:compress` event; send notification on completion |
| `src/routes/+page.svelte` | Modify | Keydown listener, menu event listeners, menu sync |
| `src/test/notification.test.ts` | **Create** | TDD for notification message builder |
| `src/test/shortcuts.test.ts` | **Create** | TDD for shortcut handler |
| `src/test/ActionBar.test.ts` | Modify | Add notification test |

---

### Task 1: Add notification dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `package.json` (via npm install)

- [ ] **Step 1: Add Rust dependency**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:
```toml
tauri-plugin-notification = "2"
```

- [ ] **Step 2: Add JS dependency**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm install @tauri-apps/plugin-notification
```

Expected: package added to `node_modules`, `package-lock.json` updated.

- [ ] **Step 3: Add notification permission to capabilities**

In `src-tauri/capabilities/default.json`, add `"notification:default"` to the `permissions` array:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default",
    "dialog:allow-open",
    "notification:default",
    {
      "identifier": "shell:allow-execute",
      "allow": [{ "name": "binaries/gs", "sidecar": true }]
    }
  ]
}
```

- [ ] **Step 4: Register plugin in lib.rs**

In `src-tauri/src/lib.rs`, add `.plugin(tauri_plugin_notification::init())` to the builder chain (after the existing plugins):

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_notification::init())  // ← add this line
    // ... rest unchanged
```

- [ ] **Step 5: Verify it compiles**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: `Finished` with no errors. (First run downloads the crate — may take a minute.)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/capabilities/default.json src-tauri/src/lib.rs package.json package-lock.json
git commit -m "feat: add tauri-plugin-notification dependency"
```

---

### Task 2: Build Rust menu module

**Files:**
- Create: `src-tauri/src/menu.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing Rust unit test**

Create `src-tauri/src/menu.rs` with only the constants and tests first:

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct MenuRegistry(pub Mutex<HashMap<String, MenuItem<tauri::Wry>>>);

pub const MENU_IDS: &[&str] = &[
    "add-files",
    "reveal-in-finder",
    "clear-queue",
    "compress",
    "reset-selected",
];

pub fn build_menu(app: &tauri::AppHandle) -> tauri::Result<(Menu<tauri::Wry>, MenuRegistry)> {
    todo!()
}

#[tauri::command]
pub fn set_menu_item_enabled(
    state: tauri::State<'_, MenuRegistry>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for id in MENU_IDS {
            assert!(seen.insert(*id), "Duplicate menu ID: {}", id);
        }
    }

    #[test]
    fn menu_ids_contains_all_expected() {
        let ids: std::collections::HashSet<&str> = MENU_IDS.iter().copied().collect();
        assert!(ids.contains("add-files"));
        assert!(ids.contains("reveal-in-finder"));
        assert!(ids.contains("clear-queue"));
        assert!(ids.contains("compress"));
        assert!(ids.contains("reset-selected"));
    }
}
```

- [ ] **Step 2: Add `pub mod menu;` to lib.rs**

In `src-tauri/src/lib.rs`, add to the top of the mod declarations:
```rust
pub mod menu;
```

- [ ] **Step 3: Run test to verify it fails (todo! panics)**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
cargo test --manifest-path src-tauri/Cargo.toml menu -- --nocapture 2>&1 | tail -10
```

Expected: `menu_ids_are_unique` and `menu_ids_contains_all_expected` both PASS (they test constants only, not `todo!()`).

- [ ] **Step 4: Implement `build_menu` and `set_menu_item_enabled`**

Replace the `todo!()` stubs in `src-tauri/src/menu.rs` with the full implementation:

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct MenuRegistry(pub Mutex<HashMap<String, MenuItem<tauri::Wry>>>);

pub const MENU_IDS: &[&str] = &[
    "add-files",
    "reveal-in-finder",
    "clear-queue",
    "compress",
    "reset-selected",
];

pub fn build_menu(app: &tauri::AppHandle) -> tauri::Result<(Menu<tauri::Wry>, MenuRegistry)> {
    let add_files = MenuItem::with_id(app, "add-files", "Add Files\u{2026}", true, Some("cmd+o"))?;
    let reveal = MenuItem::with_id(app, "reveal-in-finder", "Reveal in Finder", false, Some("cmd+shift+r"))?;
    let sep = PredefinedMenuItem::separator(app)?;
    let clear_queue = MenuItem::with_id(app, "clear-queue", "Clear Queue", false, Some("cmd+shift+backspace"))?;

    let file_menu = Submenu::with_id_and_items(
        app, "file-menu", "File", true,
        &[&add_files, &reveal, &sep, &clear_queue],
    )?;

    let compress = MenuItem::with_id(app, "compress", "Compress", false, Some("cmd+return"))?;
    let reset = MenuItem::with_id(app, "reset-selected", "Reset Selected", false, Some("cmd+r"))?;

    let queue_menu = Submenu::with_id_and_items(
        app, "queue-menu", "Queue", true,
        &[&compress, &reset],
    )?;

    let menu = Menu::with_items(app, &[&file_menu, &queue_menu])?;

    let mut map = HashMap::new();
    map.insert("add-files".to_string(), add_files);
    map.insert("reveal-in-finder".to_string(), reveal);
    map.insert("clear-queue".to_string(), clear_queue);
    map.insert("compress".to_string(), compress);
    map.insert("reset-selected".to_string(), reset);

    Ok((menu, MenuRegistry(Mutex::new(map))))
}

#[tauri::command]
pub fn set_menu_item_enabled(
    state: tauri::State<'_, MenuRegistry>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let map = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(item) = map.get(&id) {
        item.set_enabled(enabled).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for id in MENU_IDS {
            assert!(seen.insert(*id), "Duplicate menu ID: {}", id);
        }
    }

    #[test]
    fn menu_ids_contains_all_expected() {
        let ids: std::collections::HashSet<&str> = MENU_IDS.iter().copied().collect();
        assert!(ids.contains("add-files"));
        assert!(ids.contains("reveal-in-finder"));
        assert!(ids.contains("clear-queue"));
        assert!(ids.contains("compress"));
        assert!(ids.contains("reset-selected"));
    }
}
```

- [ ] **Step 5: Wire menu into lib.rs**

Replace the `run()` function in `src-tauri/src/lib.rs` with:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use crate::compress::compress_files;
    use crate::settings::{get_settings, save_settings};
    use crate::finder::reveal_in_finder;
    use crate::menu::{build_menu, set_menu_item_enabled, MenuRegistry};

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let (menu, registry) = build_menu(app.handle())?;
            app.set_menu(menu)?;
            app.handle().on_menu_event(|app, event| {
                let name = match event.id().as_ref() {
                    "add-files"          => "menu:add-files",
                    "reveal-in-finder"   => "menu:reveal-in-finder",
                    "clear-queue"        => "menu:clear-queue",
                    "compress"           => "menu:compress",
                    "reset-selected"     => "menu:reset-selected",
                    _                    => return,
                };
                let _ = app.emit(name, ());
            });
            app.manage(registry);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            compress_files,
            get_settings,
            save_settings,
            reveal_in_finder,
            get_file_meta,
            validate_pdf,
            check_path_writable_cmd,
            set_menu_item_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Run all Rust tests**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -15
```

Expected: all tests pass including the two new `menu::tests`.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/menu.rs src-tauri/src/lib.rs
git commit -m "feat: add native macOS app menu with File and Queue submenus"
```

---

### Task 3: Notification message builder (TDD)

**Files:**
- Create: `src/lib/notification.ts`
- Create: `src/test/notification.test.ts`

- [ ] **Step 1: Write the failing test**

Create `src/test/notification.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { buildNotificationBody, formatSavedBytes } from "$lib/notification";

describe("formatSavedBytes", () => {
  it("formats bytes under 1 KB", () => {
    expect(formatSavedBytes(500)).toBe("500 B");
  });

  it("formats KB (rounds to nearest KB)", () => {
    expect(formatSavedBytes(840_000)).toBe("840 KB");
  });

  it("formats MB to one decimal place", () => {
    expect(formatSavedBytes(2_400_000)).toBe("2.4 MB");
  });

  it("formats exactly 1 MB", () => {
    expect(formatSavedBytes(1_000_000)).toBe("1.0 MB");
  });
});

describe("buildNotificationBody", () => {
  it("singular noun when 1 PDF succeeds with no errors", () => {
    expect(buildNotificationBody(1, 0, 840_000)).toBe(
      "1 PDF compressed — saved 840 KB total"
    );
  });

  it("plural noun when 3 PDFs succeed with no errors", () => {
    expect(buildNotificationBody(3, 0, 2_400_000)).toBe(
      "3 PDFs compressed — saved 2.4 MB total"
    );
  });

  it("shows mixed result with done and error counts", () => {
    expect(buildNotificationBody(2, 1, 0)).toBe(
      "2 of 3 PDFs compressed — 1 failed"
    );
  });

  it("shows mixed result when only 1 succeeded", () => {
    expect(buildNotificationBody(1, 2, 500_000)).toBe(
      "1 of 3 PDFs compressed — 2 failed"
    );
  });

  it("handles zero saved bytes in success case", () => {
    expect(buildNotificationBody(1, 0, 0)).toBe(
      "1 PDF compressed — saved 0 B total"
    );
  });
});
```

- [ ] **Step 2: Run to verify it fails**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | grep -A 5 "notification"
```

Expected: FAIL — `Cannot find module '$lib/notification'`

- [ ] **Step 3: Implement the module**

Create `src/lib/notification.ts`:

```typescript
export function formatSavedBytes(bytes: number): string {
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
  if (bytes >= 1_000) return `${Math.round(bytes / 1_000)} KB`;
  return `${bytes} B`;
}

export function buildNotificationBody(
  doneCount: number,
  errorCount: number,
  savedBytes: number
): string {
  if (errorCount === 0) {
    const noun = doneCount === 1 ? "PDF" : "PDFs";
    return `${doneCount} ${noun} compressed — saved ${formatSavedBytes(savedBytes)} total`;
  }
  const total = doneCount + errorCount;
  return `${doneCount} of ${total} PDFs compressed — ${errorCount} failed`;
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | grep -E "(PASS|FAIL|notification)"
```

Expected: all notification tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/notification.ts src/test/notification.test.ts
git commit -m "feat: add notification message builder with tests"
```

---

### Task 4: Keyboard shortcut handler (TDD)

**Files:**
- Create: `src/lib/shortcuts.ts`
- Create: `src/test/shortcuts.test.ts`

- [ ] **Step 1: Write the failing tests**

Create `src/test/shortcuts.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { handleShortcut, type ShortcutState, type ShortcutActions } from "$lib/shortcuts";

function makeEvent(overrides: Partial<{
  metaKey: boolean; shiftKey: boolean; key: string; altKey: boolean; ctrlKey: boolean;
}>): KeyboardEvent {
  return {
    metaKey: false, shiftKey: false, key: "", altKey: false, ctrlKey: false,
    ...overrides,
  } as KeyboardEvent;
}

function makeState(overrides: Partial<ShortcutState> = {}): ShortcutState {
  return { hasPending: false, selectedStatus: null, hasFiles: false, isCompressing: false, ...overrides };
}

function makeActions(): ShortcutActions & { calls: string[] } {
  const calls: string[] = [];
  return {
    calls,
    addFiles:       () => calls.push("addFiles"),
    compress:       () => calls.push("compress"),
    resetSelected:  () => calls.push("resetSelected"),
    revealInFinder: () => calls.push("revealInFinder"),
    clearQueue:     () => calls.push("clearQueue"),
    removeSelected: () => calls.push("removeSelected"),
    selectNext:     () => calls.push("selectNext"),
    selectPrev:     () => calls.push("selectPrev"),
    deselect:       () => calls.push("deselect"),
  };
}

describe("handleShortcut", () => {
  it("cmd+o calls addFiles", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "o" }), makeState(), a);
    expect(a.calls).toContain("addFiles");
  });

  it("cmd+enter calls compress when hasPending and not compressing", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "Enter" }), makeState({ hasPending: true }), a);
    expect(a.calls).toContain("compress");
  });

  it("cmd+enter does nothing when no pending files", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "Enter" }), makeState({ hasPending: false }), a);
    expect(a.calls).not.toContain("compress");
  });

  it("cmd+enter does nothing while compressing", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "Enter" }), makeState({ hasPending: true, isCompressing: true }), a);
    expect(a.calls).not.toContain("compress");
  });

  it("cmd+r resets selected file when status is done", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "r" }), makeState({ selectedStatus: "done" }), a);
    expect(a.calls).toContain("resetSelected");
  });

  it("cmd+r resets selected file when status is error", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "r" }), makeState({ selectedStatus: "error" }), a);
    expect(a.calls).toContain("resetSelected");
  });

  it("cmd+r does nothing when selected file is pending", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "r" }), makeState({ selectedStatus: "pending" }), a);
    expect(a.calls).not.toContain("resetSelected");
  });

  it("cmd+shift+r reveals in Finder when selected is done", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "R" }), makeState({ selectedStatus: "done" }), a);
    expect(a.calls).toContain("revealInFinder");
  });

  it("cmd+shift+r does nothing when selected is not done", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "R" }), makeState({ selectedStatus: "pending" }), a);
    expect(a.calls).not.toContain("revealInFinder");
  });

  it("cmd+shift+backspace clears queue when files exist", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "Backspace" }), makeState({ hasFiles: true }), a);
    expect(a.calls).toContain("clearQueue");
  });

  it("cmd+shift+backspace does nothing when queue is empty", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "Backspace" }), makeState({ hasFiles: false }), a);
    expect(a.calls).not.toContain("clearQueue");
  });

  it("backspace removes selected file", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Backspace" }), makeState({ selectedStatus: "pending" }), a);
    expect(a.calls).toContain("removeSelected");
  });

  it("backspace does nothing when no file is selected", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Backspace" }), makeState({ selectedStatus: null }), a);
    expect(a.calls).not.toContain("removeSelected");
  });

  it("arrow down calls selectNext when files exist", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "ArrowDown" }), makeState({ hasFiles: true }), a);
    expect(a.calls).toContain("selectNext");
  });

  it("arrow up calls selectPrev when files exist", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "ArrowUp" }), makeState({ hasFiles: true }), a);
    expect(a.calls).toContain("selectPrev");
  });

  it("escape deselects when a file is selected", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Escape" }), makeState({ selectedStatus: "done" }), a);
    expect(a.calls).toContain("deselect");
  });

  it("escape does nothing when no file is selected", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Escape" }), makeState({ selectedStatus: null }), a);
    expect(a.calls).not.toContain("deselect");
  });

  it("returns true when a shortcut is handled", () => {
    const a = makeActions();
    const result = handleShortcut(makeEvent({ metaKey: true, key: "o" }), makeState(), a);
    expect(result).toBe(true);
  });

  it("returns false for an unrecognized key combination", () => {
    const a = makeActions();
    const result = handleShortcut(makeEvent({ key: "z" }), makeState(), a);
    expect(result).toBe(false);
  });
});
```

- [ ] **Step 2: Run to verify it fails**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | grep -E "(PASS|FAIL|shortcuts)"
```

Expected: FAIL — `Cannot find module '$lib/shortcuts'`

- [ ] **Step 3: Implement the module**

Create `src/lib/shortcuts.ts`:

```typescript
import type { FileStatus } from "$lib/stores/queueStore";

export interface ShortcutState {
  hasPending: boolean;
  selectedStatus: FileStatus | null;
  hasFiles: boolean;
  isCompressing: boolean;
}

export interface ShortcutActions {
  addFiles: () => void;
  compress: () => void;
  resetSelected: () => void;
  revealInFinder: () => void;
  clearQueue: () => void;
  removeSelected: () => void;
  selectNext: () => void;
  selectPrev: () => void;
  deselect: () => void;
}

export function handleShortcut(
  e: KeyboardEvent,
  state: ShortcutState,
  actions: ShortcutActions
): boolean {
  const { metaKey: cmd, shiftKey: shift, key } = e;
  const k = key.toLowerCase();

  if (cmd && !shift && k === "o") { actions.addFiles(); return true; }
  if (cmd && !shift && key === "Enter" && state.hasPending && !state.isCompressing) { actions.compress(); return true; }
  if (cmd && !shift && k === "r" && (state.selectedStatus === "done" || state.selectedStatus === "error")) { actions.resetSelected(); return true; }
  if (cmd && shift && k === "r" && state.selectedStatus === "done") { actions.revealInFinder(); return true; }
  if (cmd && shift && key === "Backspace" && state.hasFiles) { actions.clearQueue(); return true; }
  if (!cmd && !shift && key === "Backspace" && state.selectedStatus !== null) { actions.removeSelected(); return true; }
  if (!cmd && key === "ArrowDown" && state.hasFiles) { actions.selectNext(); return true; }
  if (!cmd && key === "ArrowUp" && state.hasFiles) { actions.selectPrev(); return true; }
  if (key === "Escape" && state.selectedStatus !== null) { actions.deselect(); return true; }

  return false;
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | grep -E "(PASS|FAIL|shortcuts)"
```

Expected: all shortcut tests PASS.

- [ ] **Step 5: Run full test suite to check for regressions**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib/shortcuts.ts src/test/shortcuts.test.ts
git commit -m "feat: add keyboard shortcut handler with tests"
```

---

### Task 5: Extract file actions to shared module

**Files:**
- Create: `src/lib/fileActions.ts`
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Create `fileActions.ts`**

Create `src/lib/fileActions.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { queue } from "$lib/stores/queueStore";

export async function addPath(path: string): Promise<void> {
  const name = path.split("/").pop() ?? path;
  try {
    const isPdf = await invoke<boolean>("validate_pdf", { path });
    if (!isPdf) {
      queue.addFile({ path, name, size: 0 });
      queue.updateStatus(path, "error", { errorMsg: "Not a valid PDF file" });
      return;
    }
    const meta = await invoke<{ size: number }>("get_file_meta", { path });
    queue.addFile({ path, name, size: meta.size });
  } catch {
    queue.addFile({ path, name, size: 0 });
  }
}

export async function addFiles(): Promise<void> {
  const paths = await open({ multiple: true, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (!paths) return;
  const list = Array.isArray(paths) ? paths : [paths];
  for (const path of list) await addPath(path);
}
```

- [ ] **Step 2: Update Sidebar.svelte to use the shared module**

In `src/lib/components/Sidebar.svelte`, replace the `<script>` block:

```svelte
<script lang="ts">
  import { queue } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";
  import { addFiles, addPath } from "$lib/fileActions";

  let isDragOver = false;

  function onDrop(e: DragEvent) {
    isDragOver = false;
    e.preventDefault();
    const items = Array.from(e.dataTransfer?.items ?? []);
    for (const item of items) {
      if (item.kind === "file") {
        const file = item.getAsFile();
        if (file) addPath((file as any).path ?? file.name);
      }
    }
  }
</script>
```

(The template and styles are unchanged — only the script block changes.)

- [ ] **Step 3: Run tests to verify no regressions**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | tail -10
```

Expected: all tests pass (Sidebar tests import `open` mock which is still in place).

- [ ] **Step 4: Commit**

```bash
git add src/lib/fileActions.ts src/lib/components/Sidebar.svelte
git commit -m "refactor: extract addFiles/addPath to shared fileActions module"
```

---

### Task 6: Add notification mock and vitest alias

**Files:**
- Create: `src/lib/mocks/tauri-plugin-notification.ts`
- Modify: `vitest.config.ts`

- [ ] **Step 1: Create the notification mock**

Create `src/lib/mocks/tauri-plugin-notification.ts`:

```typescript
export const isPermissionGranted = async () => true;
export const requestPermission = async () => "granted";
export const sendNotification = (_options: { title: string; body: string }) => {};
```

- [ ] **Step 2: Add alias to vitest.config.ts**

In `vitest.config.ts`, add the notification alias to `resolve.alias`:

```typescript
import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";
import { fileURLToPath, URL } from "url";

export default defineConfig({
  plugins: [sveltekit()],

  resolve: {
    conditions: ["browser"],
    alias: {
      "@tauri-apps/plugin-dialog": fileURLToPath(
        new URL("./src/lib/mocks/tauri-plugin-dialog.ts", import.meta.url)
      ),
      "@tauri-apps/plugin-notification": fileURLToPath(
        new URL("./src/lib/mocks/tauri-plugin-notification.ts", import.meta.url)
      ),
    },
  },

  test: {
    globals: true,
    environment: "happy-dom",
    setupFiles: ["src/test/setup.ts"],
    include: ["src/test/**/*.test.ts"],
    environmentMatchGlobs: [
      ["src/test/queueStore.test.ts", "node"],
      ["src/test/settingsStore.test.ts", "node"],
    ],
    server: {
      deps: {
        inline: [/svelte/],
      },
    },
  },
});
```

- [ ] **Step 3: Run tests to verify no regressions**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | tail -10
```

Expected: all tests still pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/mocks/tauri-plugin-notification.ts vitest.config.ts
git commit -m "test: add vitest mock for tauri-plugin-notification"
```

---

### Task 7: Add notification to ActionBar and test it

**Files:**
- Modify: `src/lib/components/ActionBar.svelte`
- Modify: `src/test/ActionBar.test.ts`

- [ ] **Step 1: Write the failing test**

Add to `src/test/ActionBar.test.ts` (inside the `describe("ActionBar", ...)` block, after the existing tests):

```typescript
import { waitFor } from "@testing-library/svelte";
import { sendNotification } from "@tauri-apps/plugin-notification";

// At the top of the describe block, add this import and spy setup:
// (Add these lines after the existing vi.mock calls at the top of the file)
```

Insert these additions at the top of `src/test/ActionBar.test.ts` (after the existing `vi.mock` calls):

```typescript
vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: vi.fn().mockResolvedValue(true),
  requestPermission: vi.fn(),
  sendNotification: vi.fn(),
}));
```

And add this import near the top:

```typescript
import { sendNotification } from "@tauri-apps/plugin-notification";
import { waitFor } from "@testing-library/svelte";
```

Then add this test inside the `describe` block:

```typescript
it("sends a notification after compression completes with done files", async () => {
  let capturedHandler: ((e: { payload: unknown }) => void) | null = null;

  vi.mocked(listen).mockImplementationOnce((_event, handler) => {
    capturedHandler = handler as (e: { payload: unknown }) => void;
    return Promise.resolve(() => {});
  });

  vi.mocked(invoke).mockImplementation(async (cmd: string) => {
    if (cmd === "compress_files" && capturedHandler) {
      capturedHandler({
        payload: { file: "/tmp/a.pdf", status: "done", compressed_size: 600_000 },
      });
    }
    return undefined;
  });

  queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1_000_000 });
  render(ActionBar);

  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: /compress 1 pdf/i }));

  await waitFor(() => {
    expect(sendNotification).toHaveBeenCalledWith({
      title: "compress[pdf]",
      body: "1 PDF compressed — saved 400 KB total",
    });
  });

  vi.mocked(invoke).mockResolvedValue(undefined);
});
```

Also add `import { listen } from "@tauri-apps/api/event";` and `import { invoke } from "@tauri-apps/api/core";` to the top if not already present (they are already mocked — just import the typed versions).

- [ ] **Step 2: Run to verify the new test fails**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test -- ActionBar 2>&1 | tail -15
```

Expected: the new `sends a notification` test FAILS.

- [ ] **Step 3: Update ActionBar.svelte with notification logic**

Replace the `<script>` block in `src/lib/components/ActionBar.svelte`:

```svelte
<script lang="ts">
  import { get, derived } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
  import { queue, pendingCount, allFinished } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";
  import { settings } from "$lib/stores/settingsStore";
  import { toast } from "$lib/stores/toastStore";
  import { buildNotificationBody } from "$lib/notification";

  const doneCount = derived(queue, ($q) => $q.filter((e) => e.status === "done").length);
  const errorCount = derived(queue, ($q) => $q.filter((e) => e.status === "error").length);

  interface ProgressEvent {
    file: string;
    status: "processing" | "done" | "error";
    saved_bytes?: number;
    compressed_size?: number;
    error_msg?: string;
  }

  let isCompressing = false;
  let unlisten: (() => void) | null = null;
  let compressTotal = 0;
  let compressDone = 0;

  async function notifyBatchComplete(done: number, errors: number, savedBytes: number) {
    try {
      let permitted = await isPermissionGranted();
      if (!permitted) {
        const result = await requestPermission();
        permitted = result === "granted";
      }
      if (!permitted) return;
      sendNotification({
        title: "compress[pdf]",
        body: buildNotificationBody(done, errors, savedBytes),
      });
    } catch {
      // Notification is non-critical — never throw
    }
  }

  async function startCompression() {
    compressTotal = get(pendingCount);
    compressDone = 0;
    isCompressing = true;

    unlisten = await listen<ProgressEvent>("compress:progress", ({ payload }) => {
      const entries = get(queue);
      const entry = entries.find((e) => e.path === payload.file);
      if (!entry) return;

      if (payload.status === "done") {
        const compressedSize = payload.compressed_size ?? (entry.size - (payload.saved_bytes ?? 0));
        queue.updateStatus(payload.file, "done", { compressedSize });
        compressDone++;
      } else if (payload.status === "error") {
        queue.updateStatus(payload.file, "error", { errorMsg: payload.error_msg });
        const name = payload.file.split("/").pop() ?? payload.file;
        toast.show(`${name}: ${payload.error_msg ?? "Compression failed"}`);
        compressDone++;
      } else {
        queue.updateStatus(payload.file, "processing");
      }
    });

    const $queue = get(queue);
    const $settings = get(settings);

    const jobs = $queue
      .filter((e) => e.status === "pending")
      .map((e) => ({ path: e.path, preset: e.preset, dpi_override: e.dpiOverride ?? null }));

    try {
      await invoke("compress_files", { jobs, settings: $settings });
    } finally {
      isCompressing = false;
      compressTotal = 0;
      compressDone = 0;
      unlisten?.();
      unlisten = null;

      const $q = get(queue);
      const done = $q.filter((e) => e.status === "done").length;
      const errors = $q.filter((e) => e.status === "error").length;
      if (done + errors > 0) {
        const savedBytes = $q
          .filter((e) => e.status === "done")
          .reduce((acc, e) => acc + (e.size - (e.compressedSize ?? e.size)), 0);
        notifyBatchComplete(done, errors, savedBytes);
      }
    }
  }

  function clearQueue() {
    queue.clear();
    selectedFileId.set(null);
  }

  onMount(() => {
    window.addEventListener("app:compress", startCompression);
  });

  onDestroy(() => {
    unlisten?.();
    window.removeEventListener("app:compress", startCompression);
  });
</script>
```

(The template and styles are unchanged.)

- [ ] **Step 4: Run ActionBar tests to verify they pass**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test -- ActionBar 2>&1 | tail -15
```

Expected: all ActionBar tests PASS including the new notification test.

- [ ] **Step 5: Run full test suite**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/ActionBar.svelte src/test/ActionBar.test.ts
git commit -m "feat: send system notification on batch compression completion"
```

---

### Task 8: Wire +page.svelte (keyboard shortcuts + menu events + menu sync)

**Files:**
- Modify: `src/routes/+page.svelte`

No new tests needed — shortcut logic is fully covered in `shortcuts.test.ts`; menu sync is visual/integration.

- [ ] **Step 1: Replace +page.svelte**

Overwrite `src/routes/+page.svelte` with:

```svelte
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DetailPanel from "$lib/components/DetailPanel.svelte";
  import ActionBar from "$lib/components/ActionBar.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import { settings } from "$lib/stores/settingsStore";
  import { queue, pendingCount } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";
  import { addFiles } from "$lib/fileActions";
  import { handleShortcut, type ShortcutState } from "$lib/shortcuts";

  $: selectedFile = $queue.find((e) => e.id === $selectedFileId) ?? null;
  $: isCompressing = $queue.some((e) => e.status === "processing");

  // ── action helpers ────────────────────────────────────────────────────────

  function revealSelected() {
    if (selectedFile?.status === "done") {
      invoke("reveal_in_finder", { path: selectedFile.path });
    }
  }

  function resetSelected() {
    if (selectedFile) queue.resetFile(selectedFile.id);
  }

  function removeSelected() {
    if (selectedFile) queue.removeFile(selectedFile.id);
  }

  function clearAll() {
    queue.clear();
    selectedFileId.set(null);
  }

  function triggerCompress() {
    window.dispatchEvent(new CustomEvent("app:compress"));
  }

  function selectNext() {
    const list = get(queue);
    const idx = list.findIndex((e) => e.id === get(selectedFileId));
    const next = list[idx + 1] ?? list[0];
    if (next) selectedFileId.set(next.id);
  }

  function selectPrev() {
    const list = get(queue);
    const idx = list.findIndex((e) => e.id === get(selectedFileId));
    const prev = list[idx - 1] ?? list[list.length - 1];
    if (prev) selectedFileId.set(prev.id);
  }

  // ── keyboard handler ──────────────────────────────────────────────────────

  function onKeyDown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const state: ShortcutState = {
      hasPending: get(pendingCount) > 0,
      selectedStatus: selectedFile?.status ?? null,
      hasFiles: get(queue).length > 0,
      isCompressing,
    };
    const handled = handleShortcut(e, state, {
      addFiles,
      compress: triggerCompress,
      resetSelected,
      revealInFinder: revealSelected,
      clearQueue: clearAll,
      removeSelected,
      selectNext,
      selectPrev,
      deselect: () => selectedFileId.set(null),
    });
    if (handled) e.preventDefault();
  }

  // ── menu item enabled sync ────────────────────────────────────────────────

  function syncMenu(id: string, enabled: boolean) {
    invoke("set_menu_item_enabled", { id, enabled }).catch(() => {});
  }

  $: syncMenu("reveal-in-finder", selectedFile?.status === "done");
  $: syncMenu("clear-queue", $queue.length > 0);
  $: syncMenu("compress", get(pendingCount) > 0 && !isCompressing);
  $: syncMenu("reset-selected", selectedFile?.status === "done" || selectedFile?.status === "error");

  // ── menu event listeners ──────────────────────────────────────────────────

  let unlisteners: Array<() => void> = [];

  onMount(async () => {
    settings.load();
    window.addEventListener("keydown", onKeyDown);
    unlisteners = await Promise.all([
      listen("menu:add-files",        () => addFiles()),
      listen("menu:compress",         () => triggerCompress()),
      listen("menu:reveal-in-finder", () => revealSelected()),
      listen("menu:clear-queue",      () => clearAll()),
      listen("menu:reset-selected",   () => resetSelected()),
    ]);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", onKeyDown);
    unlisteners.forEach((u) => u());
  });
</script>

<div class="app">
  <div class="content">
    <Sidebar />
    <DetailPanel />
  </div>
  <ActionBar />
</div>

<Toast />

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
  }
  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
</style>
```

- [ ] **Step 2: Run full test suite**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm test 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 3: Run all Rust tests**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: wire keyboard shortcuts and menu events in page"
```

---

### Task 9: Manual verification

- [ ] **Step 1: Start the app in dev mode**

```bash
cd "/Users/k4iju/Projects/compress[pdf]"
npm run tauri dev
```

- [ ] **Step 2: Verify the menu bar**

Check the macOS menu bar while the app is focused:
- "File" menu appears with: Add Files…(⌘O), Reveal in Finder (⌘⇧R, greyed), separator, Clear Queue (⌘⇧⌫, greyed)
- "Queue" menu appears with: Compress (⌘↵, greyed), Reset Selected (⌘R, greyed)

- [ ] **Step 3: Verify menu item state sync**

- Add a PDF → "Clear Queue" and "Compress" become enabled
- Select a file → "Remove" (Delete) works in menu bar? (No — only via keyboard)
- Compress a file → select the done result → "Reveal in Finder" and "Reset Selected" become enabled

- [ ] **Step 4: Verify keyboard shortcuts**

| Shortcut | Expected behaviour |
|----------|--------------------|
| ⌘O | File picker opens |
| Drop a PDF, then ⌘↵ | Compression starts |
| ↑ / ↓ | Selection moves through the queue |
| ⌫ | Selected file removed |
| ⌘⇧⌫ | Queue clears |
| After compress: ⌘⇧R | Finder opens and highlights the file |
| After compress: ⌘R | File resets to pending |
| Esc | File deselected |

- [ ] **Step 5: Verify notification**

Compress at least one file. A macOS notification should appear titled "compress[pdf]" with the correct body. If it's the first run, macOS will prompt for notification permission — allow it.

- [ ] **Step 6: Final commit tag**

```bash
git tag v1.2.0
```
