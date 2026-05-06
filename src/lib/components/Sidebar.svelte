<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { queue } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";

  let isDragOver = false;

  async function handleAddFiles() {
    const paths = await open({ multiple: true, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!paths) return;
    const list = Array.isArray(paths) ? paths : [paths];
    for (const path of list) await addPath(path);
  }

  async function addPath(path: string) {
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

<aside
  class="sidebar"
  class:drag-over={isDragOver}
  on:dragover|preventDefault={() => (isDragOver = true)}
  on:dragleave={() => (isDragOver = false)}
  on:drop={onDrop}
  role="region"
  aria-label="File queue"
>
  <div class="header">QUEUE</div>

  {#if $queue.length === 0}
    <div class="empty">Drop PDFs here</div>
  {:else}
    <ul class="file-list">
      {#each $queue as entry (entry.id)}
        <li
          class="file-row"
          class:selected={$selectedFileId === entry.id}
          on:click={() => selectedFileId.set(entry.id)}
          tabindex="0"
          on:keydown={(e) => e.key === "Enter" && selectedFileId.set(entry.id)}
        >
          <span class="status-icon" class:done={entry.status === "done"} class:error={entry.status === "error"} class:processing={entry.status === "processing"}>
            {#if entry.status === "done"}✓{:else if entry.status === "error"}✕{:else if entry.status === "processing"}◌{:else}·{/if}
          </span>
          <span class="filename">{entry.name}</span>
          <button
            class="remove-btn"
            on:click|stopPropagation={() => queue.removeFile(entry.id)}
            aria-label="Remove {entry.name}"
          >✕</button>
        </li>
      {/each}
    </ul>
  {/if}

  <button class="add-btn" on:click={handleAddFiles}>+ Add files</button>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    overflow: hidden;
    flex-shrink: 0;
  }
  .sidebar.drag-over { outline: 2px dashed var(--accent); outline-offset: -4px; }
  .header {
    padding: 8px 12px 4px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    color: var(--text-tertiary);
    text-transform: uppercase;
  }
  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    font-size: 11px;
    text-align: center;
    padding: 16px;
    margin: 8px;
    border: 1.5px dashed var(--border);
    border-radius: var(--radius-md);
  }
  .file-list { flex: 1; overflow-y: auto; list-style: none; padding: 4px 0; }
  .file-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    margin: 1px 4px;
  }
  .file-row:hover, .file-row.selected { background: var(--bg-tertiary); }
  .file-row.selected { background: rgba(0, 122, 255, 0.15); }
  .status-icon { font-size: 9px; flex-shrink: 0; color: var(--text-tertiary); }
  .status-icon.done { color: var(--success); }
  .status-icon.error { color: var(--error); }
  .status-icon.processing { color: var(--accent); }
  .filename { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; }
  .remove-btn { display: none; background: none; border: none; color: var(--text-tertiary); cursor: pointer; font-size: 10px; padding: 2px; }
  .file-row:hover .remove-btn { display: block; }
  .add-btn {
    margin: 8px;
    padding: 6px;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 11px;
    text-align: center;
    flex-shrink: 0;
  }
  .add-btn:hover { background: var(--bg-tertiary); color: var(--text-secondary); }
</style>
