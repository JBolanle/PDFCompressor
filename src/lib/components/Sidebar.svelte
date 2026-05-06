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

<aside
  class="sidebar"
  class:drag-over={isDragOver}
  on:dragover|preventDefault={() => (isDragOver = true)}
  on:dragleave={() => (isDragOver = false)}
  on:drop={onDrop}
  role="region"
  aria-label="File queue"
>
  <div class="header">
    Queue
    {#if $queue.length > 0}<span class="count">{$queue.length}</span>{/if}
  </div>

  {#if $queue.length === 0}
    <div class="empty">
      <span class="empty-title">Drop PDFs here</span>
      <span class="empty-sub">or use "+ Add files" below</span>
    </div>
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
            {#if entry.status === "done"}✓{:else if entry.status === "error"}✕{:else if entry.status === "processing"}<span class="spinner"></span>{:else}○{/if}
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

  <button class="add-btn" on:click={addFiles}>+ Add files</button>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    border-top: 1px solid var(--border);
    overflow: hidden;
    flex-shrink: 0;
  }
  .sidebar.drag-over { background: color-mix(in oklch, var(--bg-secondary), var(--accent) 8%); }
  .sidebar.drag-over .empty { border-color: var(--accent); color: var(--accent); }
  .header {
    padding: 8px 12px 4px;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    letter-spacing: 0.01em;
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
  }
  .count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: 9px;
    font-weight: var(--weight-semibold);
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    margin-left: 6px;
  }
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 16px;
    margin: 8px;
    border: 1.5px dashed var(--border);
    border-radius: var(--radius-md);
    transition: border-color 0.15s, color 0.15s;
  }
  .empty-title { font-size: 12px; color: var(--text-secondary); font-weight: var(--weight-medium); }
  .empty-sub { font-size: 10px; color: var(--text-tertiary); margin-top: 4px; }
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
  .file-row.selected { background: var(--accent-muted); }
  .status-icon { font-size: 10px; flex-shrink: 0; color: var(--text-tertiary); display: flex; align-items: center; }
  .status-icon.done { color: var(--success); }
  .status-icon.error { color: var(--error); }
  .status-icon.processing { color: var(--accent); }
  .spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--accent);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .filename { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; }
  .remove-btn { opacity: 0; background: none; border: none; color: var(--text-tertiary); cursor: pointer; font-size: 10px; padding: 2px; flex-shrink: 0; transition: opacity 0.1s; }
  .file-row:hover .remove-btn { opacity: 1; }
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
