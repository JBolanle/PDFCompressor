# Polish: Keyboard Shortcuts, App Menu, System Notifications

**Date:** 2026-05-06  
**Status:** Approved

## Goal

Make compress[pdf] feel like a real macOS citizen: discoverable keyboard shortcuts, a proper app menu bar, and system notifications on batch completion.

## Menu Structure

Two custom menus added to the macOS app menu bar (alongside the auto-managed app menu Tauri provides):

```
File
  Add Files…          ⌘O
  Reveal in Finder    ⌘⇧R    (disabled unless selected file status is "done")
  ─────
  Clear Queue         ⌘⇧⌫   (disabled when queue is empty)

Queue
  Compress            ⌘↵     (disabled when no pending files or compressing)
  Reset Selected      ⌘R     (disabled unless selected file is done/error)
```

"Queue" is a custom domain menu — keeps compression actions grouped and labelled for discoverability. Avoided a conventional "Edit" menu since the app has no text editing.

## Keyboard Shortcuts

Single `keydown` listener on `window` in `+page.svelte`. Same bindings as menu items, plus navigation:

| Key | Action | Condition |
|-----|--------|-----------|
| `⌘O` | Add files (open picker) | always |
| `⌘↵` | Start compression | pending files exist |
| `⌘R` | Reset selected file to pending | selected file is done/error |
| `⌘⇧R` | Reveal in Finder | selected file is done |
| `⌘⇧⌫` | Clear queue | queue not empty |
| `⌫` | Remove selected file | file selected |
| `↑` / `↓` | Move selection up/down in queue | queue not empty |
| `Esc` | Deselect | file selected |

## System Notifications

Fired once when a batch completes (all files done or error), only when compression was user-triggered in the current session.

- **All succeeded:** `"3 PDFs compressed — saved 2.4 MB total"`
- **Mixed:** `"2 of 3 PDFs compressed — 1 failed"`

Uses `tauri-plugin-notification`. macOS prompts for permission on first use; clicking the notification brings the app to front (default behaviour).

## Architecture

### Rust — `src-tauri/src/menu.rs` (new file)

- Builds the menu using `tauri::menu` API at app startup in `lib.rs`
- `on_menu_event` handler emits Tauri events to the frontend:
  - `menu:add-files`
  - `menu:compress`
  - `menu:reveal-in-finder`
  - `menu:clear-queue`
  - `menu:reset-selected`
- New Tauri command `set_menu_item_enabled(id: String, enabled: bool)` — frontend calls this to grey/ungrey items as app state changes

### Svelte — `src/routes/+page.svelte`

- Single `keydown` listener on `window` (in `onMount`, cleaned up in `onDestroy`)
- Calls the same action functions already used by buttons — no logic duplication
- `listen()` calls for each `menu:*` event, routing to the same action functions
- Reactive `$:` blocks sync menu item enabled state to stores (`pendingCount`, `selectedFileId`, `queue`, `isCompressing`)

### Notifications — `src/lib/components/ActionBar.svelte`

- At the end of `startCompression()`, after `invoke('compress_files')` resolves, compute totals from queue store and call `sendNotification`
- Logic stays co-located with the compression trigger

### No new stores needed

All conditions read from existing stores: `queue`, `pendingCount`, `selectedFileId`, `allFinished`.

## Dependencies to add

- Rust: `tauri-plugin-notification = "2"` in `Cargo.toml`
- JS: `@tauri-apps/plugin-notification` in `package.json`
- `tauri-plugin-notification::init()` registered in `lib.rs`
- Notification permission declared in `tauri.conf.json` capabilities

## Testing

- Unit: menu event routing (Rust handler emits correct event for each menu ID)
- Integration: keyboard shortcut conditions (disabled when queue empty, etc.)
- Manual: notification appears on batch complete; clicking it focuses the app
