# Settings: Inline in Detail Panel — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move app settings (output folder, file naming) from a modal sheet into a persistent section at the bottom of the Detail Panel, with instant-save on every change.

**Architecture:** The settings section is inlined directly into `DetailPanel.svelte` — no new component. Each radio `on:change` calls `settings.save()` directly, updating the store and persisting to disk in one step. The modal `SettingsSheet.svelte` and its trigger (titlebar + gear button) are deleted entirely.

**Tech Stack:** Svelte 5, TypeScript, Tauri 2, `@tauri-apps/plugin-dialog` (folder picker), Vitest + Testing Library (tests)

---

## File Map

| File | Change |
|---|---|
| `src/lib/components/DetailPanel.svelte` | Add `settings` store + `open` dialog import; remove empty-state; add settings section at bottom |
| `src/routes/+page.svelte` | Remove titlebar div + CSS, `showSettings` state, `SettingsSheet` import and conditional |
| `src/lib/components/SettingsSheet.svelte` | Deleted |
| `src/test/DetailPanel.test.ts` | Replace placeholder test; add 6 settings test cases |

---

## Task 1: Clean up `+page.svelte`

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Replace the full content of `+page.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DetailPanel from "$lib/components/DetailPanel.svelte";
  import ActionBar from "$lib/components/ActionBar.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import { settings } from "$lib/stores/settingsStore";

  onMount(() => settings.load());
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

- [ ] **Step 2: Run the full test suite to confirm no regressions**

```bash
npx vitest run
```

Expected: all existing tests pass (DetailPanel, Sidebar, ActionBar, Toast, queueStore, settingsStore).

- [ ] **Step 3: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "chore: remove settings modal trigger and titlebar from page"
```

---

## Task 2: Write failing tests for the settings section

**Files:**
- Modify: `src/test/DetailPanel.test.ts`

- [ ] **Step 1: Replace the full content of `DetailPanel.test.ts`**

```ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { get } from "svelte/store";
import { queue } from "$lib/stores/queueStore";
import { selectedFileId } from "$lib/stores/selectionStore";
import { settings } from "$lib/stores/settingsStore";
import DetailPanel from "$lib/components/DetailPanel.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";

describe("DetailPanel", () => {
  beforeEach(async () => {
    queue.clear();
    selectedFileId.set(null);
    await settings.save({ output_mode: "same_as_source", output_folder: null, naming: "suffix" });
    vi.clearAllMocks();
  });

  // ── existing tests ────────────────────────────────────────────────────────

  it("shows file name pre-compression", () => {
    queue.addFile({ path: "/tmp/report.pdf", name: "report.pdf", size: 3_200_000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByText("report.pdf")).toBeInTheDocument();
  });

  it("shows Show in Finder button when status is done", () => {
    queue.addFile({ path: "/tmp/report.pdf", name: "report.pdf", size: 3_200_000 });
    queue.updateStatus("/tmp/report.pdf", "done", { compressedSize: 1_100_000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByText(/show in finder/i)).toBeInTheDocument();
  });

  it("shows Apply to all button when a pending file is selected", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByRole("button", { name: /apply to all/i })).toBeInTheDocument();
  });

  it("Apply to all updates preset on all other pending files", async () => {
    const user = userEvent.setup();
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/b.pdf", name: "b.pdf", size: 2000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    await user.click(screen.getByRole("button", { name: /apply to all/i }));
    expect(get(queue)[1].preset).toBe("balanced");
  });

  it("does not show Apply to all button when selected file is done", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.updateStatus("/tmp/a.pdf", "done", { compressedSize: 500 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.queryByRole("button", { name: /apply to all/i })).not.toBeInTheDocument();
  });

  // ── settings section ──────────────────────────────────────────────────────

  it("shows settings section when no file is selected", () => {
    render(DetailPanel);
    expect(screen.getByText(/output folder/i)).toBeInTheDocument();
    expect(screen.getByText(/file naming/i)).toBeInTheDocument();
  });

  it("shows settings section when a file is selected", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByText(/output folder/i)).toBeInTheDocument();
    expect(screen.getByText(/file naming/i)).toBeInTheDocument();
  });

  it("changing output mode to custom_folder saves immediately", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("radio", { name: /custom folder/i }));
    expect(invoke).toHaveBeenCalledWith("save_settings", {
      settings: expect.objectContaining({ output_mode: "custom_folder" }),
    });
  });

  it("changing file naming to overwrite saves immediately", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("radio", { name: /overwrite original/i }));
    expect(invoke).toHaveBeenCalledWith("save_settings", {
      settings: expect.objectContaining({ naming: "overwrite" }),
    });
  });

  it("shows Choose button when output mode is custom_folder", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("radio", { name: /custom folder/i }));
    expect(screen.getByRole("button", { name: /choose/i })).toBeInTheDocument();
  });

  it("does not show Choose button when output mode is same_as_source", () => {
    render(DetailPanel);
    expect(screen.queryByRole("button", { name: /choose/i })).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run only the DetailPanel tests to confirm they fail**

```bash
npx vitest run src/test/DetailPanel.test.ts
```

Expected: the 6 new settings tests FAIL (settings section not yet rendered). The 5 existing tests pass.

---

## Task 3: Implement the settings section in `DetailPanel.svelte`

**Files:**
- Modify: `src/lib/components/DetailPanel.svelte`

- [ ] **Step 1: Replace the full content of `DetailPanel.svelte`**

```svelte
<script lang="ts">
  import { derived } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { queue, type Preset } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";
  import { settings } from "$lib/stores/settingsStore";

  const selectedFile = derived([queue, selectedFileId], ([$q, $id]) => $q.find((e) => e.id === $id) ?? null);

  function formatSize(bytes: number): string {
    if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
    if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(0)} KB`;
    return `${bytes} B`;
  }

  function savingsPct(original: number, compressed: number): string {
    return `−${Math.round(((original - compressed) / original) * 100)}%`;
  }

  function revealInFinder(path: string) {
    invoke("reveal_in_finder", { path });
  }

  const presetDpiRanges: Record<Preset, [number, number, number]> = {
    max:      [50,  72,  100],
    balanced: [100, 150, 200],
    minimal:  [200, 300, 400],
  };

  $: currentPreset = $selectedFile?.preset ?? "balanced";
  $: sliderMin = presetDpiRanges[currentPreset][0];
  $: sliderMax = presetDpiRanges[currentPreset][2];
  $: sliderValue = $selectedFile?.dpiOverride ?? presetDpiRanges[currentPreset][1];

  function onPresetChange(preset: Preset) {
    if (!$selectedFile) return;
    queue.updatePreset($selectedFile.id, preset, presetDpiRanges[preset][1]);
  }

  function onSliderChange(e: Event) {
    if (!$selectedFile) return;
    queue.updatePreset($selectedFile.id, $selectedFile.preset, Number((e.target as HTMLInputElement).value));
  }

  function applyToAll() {
    if (!$selectedFile) return;
    queue.updateAllPresets($selectedFile.preset, $selectedFile.dpiOverride ?? presetDpiRanges[$selectedFile.preset][1]);
  }

  async function pickFolder() {
    const folder = await open({ directory: true });
    if (typeof folder === "string") {
      await settings.save({ ...$settings, output_folder: folder });
    }
  }
</script>

<section class="detail-panel">
  {#if $selectedFile}
    <div class="file-info">
      <h2 class="filename">{$selectedFile.name}</h2>
      <div class="sizes">
        <div class="size-row"><span class="label">Original</span><span>{formatSize($selectedFile.size)}</span></div>
        {#if $selectedFile.status === "done" && $selectedFile.compressedSize !== undefined}
          <div class="size-row">
            <span class="label">Compressed</span>
            <span class="compressed">{formatSize($selectedFile.compressedSize)}</span>
          </div>
          <div class="savings">{savingsPct($selectedFile.size, $selectedFile.compressedSize)}</div>
          <button class="finder-btn" on:click={() => revealInFinder($selectedFile!.path)}>
            Show in Finder
          </button>
        {/if}
        {#if $selectedFile.status === "error"}
          <div class="error-msg">{$selectedFile.errorMsg ?? "Compression failed"}</div>
        {/if}
      </div>
    </div>

    {#if $selectedFile.status === "pending" || $selectedFile.status === "processing"}
      <div class="quality-controls">
        <div class="section-label">Quality</div>
        <div class="preset-control">
          {#each (["max", "balanced", "minimal"] as Preset[]) as p}
            <button class="preset-btn" class:active={$selectedFile.preset === p} on:click={() => onPresetChange(p)}>
              {p.charAt(0).toUpperCase() + p.slice(1)}
            </button>
          {/each}
        </div>
        <input type="range" class="dpi-slider" min={sliderMin} max={sliderMax} value={sliderValue} on:input={onSliderChange} />
        <div class="dpi-row">
          <span class="dpi-label">{sliderValue} DPI</span>
          <button class="apply-all-btn" on:click={applyToAll}>Apply to all</button>
        </div>
      </div>
    {/if}
  {/if}

  <div class="settings-section">
    <div class="section-label">Settings</div>

    <div class="field">
      <div class="field-label">Output Folder</div>
      <label class="radio-label">
        <input type="radio" name="output_mode" value="same_as_source"
          checked={$settings.output_mode === "same_as_source"}
          on:change={() => settings.save({ ...$settings, output_mode: "same_as_source" })} />
        Same as source
      </label>
      <label class="radio-label">
        <input type="radio" name="output_mode" value="custom_folder"
          checked={$settings.output_mode === "custom_folder"}
          on:change={() => settings.save({ ...$settings, output_mode: "custom_folder" })} />
        Custom folder
      </label>
      {#if $settings.output_mode === "custom_folder"}
        <div class="folder-row">
          <span class="folder-path">{$settings.output_folder ?? "No folder selected"}</span>
          <button on:click={pickFolder}>Choose…</button>
        </div>
      {/if}
    </div>

    <div class="field">
      <div class="field-label">File Naming</div>
      <label class="radio-label">
        <input type="radio" name="naming" value="suffix"
          checked={$settings.naming === "suffix"}
          on:change={() => settings.save({ ...$settings, naming: "suffix" })} />
        Add <code>_compressed</code> suffix
      </label>
      <label class="radio-label">
        <input type="radio" name="naming" value="overwrite"
          checked={$settings.naming === "overwrite"}
          on:change={() => settings.save({ ...$settings, naming: "overwrite" })} />
        Overwrite original
      </label>
    </div>
  </div>
</section>

<style>
  .detail-panel { flex: 1; padding: 16px; display: flex; flex-direction: column; gap: 16px; overflow-y: auto; }
  .filename { font-size: 14px; font-weight: 600; margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sizes { display: flex; flex-direction: column; gap: 4px; }
  .size-row { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-secondary); }
  .label { color: var(--text-tertiary); }
  .compressed { color: var(--success); }
  .savings { font-size: 22px; font-weight: 700; color: var(--success); margin-top: 4px; }
  .finder-btn { margin-top: 8px; padding: 6px 12px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-primary); cursor: pointer; font-size: 12px; }
  .finder-btn:hover { background: var(--bg-tertiary); }
  .error-msg { margin-top: 4px; font-size: 11px; color: var(--error); }
  .quality-controls { display: flex; flex-direction: column; gap: 8px; padding-top: 12px; border-top: 1px solid var(--border); }
  .section-label { font-size: 10px; font-weight: 600; letter-spacing: 0.06em; text-transform: uppercase; color: var(--text-tertiary); }
  .preset-control { display: flex; gap: 4px; }
  .preset-btn { flex: 1; padding: 5px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-tertiary); cursor: pointer; font-size: 11px; }
  .preset-btn.active { background: var(--accent); border-color: var(--accent); color: white; }
  .dpi-slider { width: 100%; accent-color: var(--accent); }
  .dpi-row { display: flex; justify-content: space-between; align-items: center; }
  .dpi-label { font-size: 10px; color: var(--text-tertiary); }
  .apply-all-btn { background: none; border: none; color: var(--accent); cursor: pointer; font-size: 10px; padding: 0; }
  .apply-all-btn:hover { text-decoration: underline; }
  .settings-section { display: flex; flex-direction: column; gap: 8px; padding-top: 12px; border-top: 1px solid var(--border); margin-top: auto; }
  .field { display: flex; flex-direction: column; gap: 6px; }
  .field-label { font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-tertiary); }
  .radio-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 12px; }
  .folder-row { display: flex; align-items: center; gap: 8px; }
  .folder-path { flex: 1; font-size: 11px; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder-row button { padding: 5px 10px; border-radius: var(--radius-sm); font-size: 12px; cursor: pointer; border: 1px solid var(--border); background: var(--bg-tertiary); color: var(--text-primary); }
</style>
```

- [ ] **Step 2: Run DetailPanel tests to confirm they all pass**

```bash
npx vitest run src/test/DetailPanel.test.ts
```

Expected: all 11 tests PASS.

- [ ] **Step 3: Run the full suite to confirm no regressions**

```bash
npx vitest run
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/DetailPanel.svelte src/test/DetailPanel.test.ts
git commit -m "feat: inline settings into detail panel with instant save"
```

---

## Task 4: Delete `SettingsSheet.svelte` and final commit

**Files:**
- Delete: `src/lib/components/SettingsSheet.svelte`

- [ ] **Step 1: Delete the file**

```bash
rm src/lib/components/SettingsSheet.svelte
```

- [ ] **Step 2: Run the full suite one more time**

```bash
npx vitest run
```

Expected: all tests pass (nothing imports SettingsSheet anymore).

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "chore: delete SettingsSheet component"
```
