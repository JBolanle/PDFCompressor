<script lang="ts">
  import { createEventDispatcher, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { settings } from "$lib/stores/settingsStore";
  import type { AppSettings } from "$lib/stores/settingsStore";

  const dispatch = createEventDispatcher();

  let draft: AppSettings;
  const unsub = settings.subscribe((s) => { draft = { ...s }; });
  onDestroy(unsub);

  async function pickFolder() {
    const folder = await open({ directory: true });
    if (typeof folder === "string") draft = { ...draft, output_folder: folder };
  }

  async function save() {
    await settings.save(draft);
    dispatch("close");
  }
</script>

<div class="overlay" on:click|self={() => dispatch("close")} role="dialog" aria-modal="true">
  <div class="sheet">
    <h2>Settings</h2>

    <div class="field">
      <div class="field-label">Output Folder</div>
      <label class="radio-label">
        <input type="radio" bind:group={draft.output_mode} value="same_as_source" />
        Same as source
      </label>
      <label class="radio-label">
        <input type="radio" bind:group={draft.output_mode} value="custom_folder" />
        Custom folder
      </label>
      {#if draft.output_mode === "custom_folder"}
        <div class="folder-row">
          <span class="folder-path">{draft.output_folder ?? "No folder selected"}</span>
          <button on:click={pickFolder}>Choose…</button>
        </div>
      {/if}
    </div>

    <div class="field">
      <div class="field-label">File Naming</div>
      <label class="radio-label">
        <input type="radio" bind:group={draft.naming} value="suffix" />
        Add <code>_compressed</code> suffix
      </label>
      <label class="radio-label">
        <input type="radio" bind:group={draft.naming} value="overwrite" />
        Overwrite original
      </label>
    </div>

    <div class="actions">
      <button class="cancel-btn" on:click={() => dispatch("close")}>Cancel</button>
      <button class="save-btn" on:click={save}>Save</button>
    </div>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .sheet { background: var(--bg-secondary); border-radius: var(--radius-lg); padding: 20px; width: 340px; display: flex; flex-direction: column; gap: 16px; border: 1px solid var(--border); }
  h2 { font-size: 15px; font-weight: 600; }
  .field { display: flex; flex-direction: column; gap: 8px; }
  .field-label { font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-tertiary); }
  .radio-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 12px; }
  .folder-row { display: flex; align-items: center; gap: 8px; }
  .folder-path { flex: 1; font-size: 11px; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder-row button, .cancel-btn, .save-btn { padding: 5px 10px; border-radius: var(--radius-sm); font-size: 12px; cursor: pointer; border: 1px solid var(--border); background: var(--bg-tertiary); color: var(--text-primary); }
  .actions { display: flex; justify-content: flex-end; gap: 8px; }
  .save-btn { background: var(--accent); border-color: var(--accent); color: white; }
</style>
