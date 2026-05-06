# Settings: Inline in Detail Panel

**Date:** 2026-05-06  
**Status:** Approved

## Problem

Settings (output folder, file naming) live in a modal sheet triggered by a gear button in the titlebar. This hides infrequently-but-meaningfully-used preferences behind an extra click and a dismissible overlay.

## Decision

Move settings into a persistent section at the bottom of the Detail Panel. Changes apply instantly — no Save/Cancel flow.

## Layout

The Detail Panel renders three stacked sections, each separated by a `border-top` divider:

1. **File info** — filename, original/compressed sizes, savings %, Show in Finder button (conditional on file selected + done status)
2. **Quality controls** — preset buttons + DPI slider + Apply to all (conditional on file selected + pending/processing status)
3. **Settings** — always rendered, regardless of selection state

When no file is selected, sections 1 and 2 are absent; the settings section fills the panel (no "Select a file" placeholder).

## Settings Section Contents

- **Output folder** label (10px uppercase)
  - Radio: "Same as source" (`same_as_source`)
  - Radio: "Custom folder" (`custom_folder`)
  - When `custom_folder` selected: folder path display + "Choose…" button (uses Tauri dialog picker)
- **File naming** label
  - Radio: "Add `_compressed` suffix" (`suffix`)
  - Radio: "Overwrite original" (`overwrite`)

## Data Flow

Each radio `on:change` calls `settings.save({ ...$settings, field: newValue })` directly. No draft state. The `settings` store is imported into `DetailPanel.svelte` alongside the existing queue and selection stores.

The `pickFolder` async function (Tauri dialog) runs on "Choose…" click, updates `output_folder`, then calls `settings.save()`.

## Files Changed

| File | Action |
|---|---|
| `src/lib/components/DetailPanel.svelte` | Add `settings` store import; replace empty-state with settings section; add settings section below quality controls |
| `src/routes/+page.svelte` | Remove `.titlebar` div, gear button, `showSettings` state, `SettingsSheet` import and conditional render |
| `src/lib/components/SettingsSheet.svelte` | Deleted |
| `src/test/DetailPanel.test.ts` | Add settings-related test cases (see Testing) |

## Testing

Existing `settingsStore.test.ts` is unchanged.

New cases added to `DetailPanel.test.ts`:

1. Settings section renders when no file is selected (output folder and naming labels visible)
2. Settings section renders when a file is selected
3. Changing output mode radio to "custom_folder" persists to settings store immediately
4. Changing naming radio to "overwrite" persists to settings store immediately
5. "Choose…" folder picker button appears when output mode is "custom_folder"
6. "Choose…" folder picker button is absent when output mode is "same_as_source"

The existing test `"shows placeholder when no file is selected"` is updated — the placeholder text is removed, so the test is replaced by case 1 above.

## Removals

- `.titlebar` CSS rule in `+page.svelte`
- `.gear-btn` CSS rule in `+page.svelte`
- `SettingsSheet.svelte` component (entire file)
- `showSettings` reactive variable and all references in `+page.svelte`
