<script lang="ts">
  import { derived, get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";
  import { queue, pendingCount } from "$lib/stores/queueStore";
  import { settings } from "$lib/stores/settingsStore";

  interface ProgressEvent {
    file: string;
    status: "processing" | "done" | "error";
    saved_bytes?: number;
    error_msg?: string;
  }

  let isCompressing = false;
  let unlisten: (() => void) | null = null;

  const isDisabled = derived(
    [pendingCount],
    ([$pending]) => $pending === 0 || isCompressing
  );

  async function startCompression() {
    isCompressing = true;

    unlisten = await listen<ProgressEvent>("compress:progress", ({ payload }) => {
      const entries = get(queue);
      const entry = entries.find((e) => e.path === payload.file);
      if (!entry) return;

      if (payload.status === "done") {
        const compressedSize = entry.size - (payload.saved_bytes ?? 0);
        queue.updateStatus(payload.file, "done", { compressedSize });
      } else if (payload.status === "error") {
        queue.updateStatus(payload.file, "error", { errorMsg: payload.error_msg });
        // Toast notification handled in Task 17
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
      unlisten?.();
      unlisten = null;
    }
  }

  onDestroy(() => unlisten?.());
</script>

<div class="action-bar">
  <button
    class="compress-btn"
    disabled={$isDisabled}
    on:click={startCompression}
  >
    {#if isCompressing}
      Compressing…
    {:else}
      Compress {$pendingCount} {$pendingCount === 1 ? "file" : "files"} ›
    {/if}
  </button>
</div>

<style>
  .action-bar { height: var(--action-bar-height); padding: 8px 12px; border-top: 1px solid var(--border); display: flex; align-items: center; flex-shrink: 0; }
  .compress-btn { width: 100%; height: 36px; background: var(--accent); color: white; border: none; border-radius: var(--radius-md); font-size: 13px; font-weight: 600; cursor: pointer; transition: background 0.1s; }
  .compress-btn:hover:not(:disabled) { background: var(--accent-hover); }
  .compress-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
