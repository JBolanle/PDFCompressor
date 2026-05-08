<script lang="ts">
  import { get, derived } from "svelte/store";
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  const reducedMotion = typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
  import { queue, pendingCount } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";
  import { settings } from "$lib/stores/settingsStore";
  import { toast } from "$lib/stores/toastStore";
  import { basename } from "$lib/fileActions";
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
        const name = basename(payload.file);
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
    if (isCompressing) {
      isCompressing = false;
      compressTotal = 0;
      compressDone = 0;
      unlisten?.();
      unlisten = null;
    }
  }

  onMount(() => {
    window.addEventListener("app:compress", startCompression);
  });

  onDestroy(() => {
    unlisten?.();
    window.removeEventListener("app:compress", startCompression);
  });
</script>

<div class="action-bar">
  {#if $queue.length > 0 && !isCompressing}
    <button class="clear-btn" on:click={clearQueue}>Clear queue</button>
  {/if}
  {#if $doneCount > 0 || $errorCount > 0}
    <div class="status-summary">
      {#if $doneCount > 0}<span class="done-count" in:scale={{ duration: reducedMotion ? 0 : 200, start: 0.6, easing: cubicOut }}>{$doneCount} done</span>{/if}
      {#if $errorCount > 0}<span class="error-count">{$errorCount} error{$errorCount > 1 ? "s" : ""}</span>{/if}
    </div>
  {/if}
  <button
    class="compress-btn"
    disabled={$pendingCount === 0 || isCompressing}
    on:click={startCompression}
  >
    {#if isCompressing}
      {#if compressTotal > 1}
        Compressing {compressDone + 1}/{compressTotal}…
      {:else}
        Compressing…
      {/if}
    {:else}
      Compress {$pendingCount} {$pendingCount === 1 ? "PDF" : "PDFs"}
    {/if}
  </button>
</div>

<style>
  .action-bar { height: var(--action-bar-height); padding: 8px 12px; border-top: 1px solid var(--border); display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
  .compress-btn { flex: 1; height: 36px; background: var(--accent); color: white; border: none; border-radius: var(--radius-md); font-size: 13px; font-weight: 600; font-family: var(--font-ui); cursor: pointer; transition: background 0.1s, opacity 0.15s, transform 0.08s, box-shadow 0.12s; }
  .compress-btn:hover:not(:disabled) { background: var(--accent-hover); box-shadow: 0 2px 10px oklch(59% 0.17 251 / 0.38); transform: translateY(-1px); }
  .compress-btn:active:not(:disabled) { transform: scale(0.97); box-shadow: none; }
  @media (prefers-reduced-motion: reduce) {
    .compress-btn:hover:not(:disabled) { transform: none; }
    .compress-btn { transition: background 0.1s, opacity 0.15s; }
  }
  .compress-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .clear-btn { height: 36px; padding: 0 14px; background: none; border: 1px solid var(--border); border-radius: var(--radius-md); color: var(--text-secondary); font-size: 13px; cursor: pointer; white-space: nowrap; transition: background 0.1s; }
  .clear-btn:hover { background: var(--bg-tertiary); }
  .status-summary { display: flex; gap: 8px; align-items: center; font-size: var(--text-xs); white-space: nowrap; }
  .done-count { color: var(--success); }
  .error-count { color: var(--error); }
</style>
