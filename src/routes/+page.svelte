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
  import { addFiles, revealInFinder } from "$lib/fileActions";
  import { handleShortcut, type ShortcutState } from "$lib/shortcuts";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { toast } from "$lib/stores/toastStore";

  $: selectedFile = $queue.find((e) => e.id === $selectedFileId) ?? null;
  $: isCompressing = $queue.some((e) => e.status === "processing");

  // ── action helpers ────────────────────────────────────────────────────────

  function revealSelected() {
    if (selectedFile?.status === "done") {
      revealInFinder(selectedFile.path);
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

  // ── update check helper ───────────────────────────────────────────────────

  async function checkAndShowUpdateToast(showUpToDate = false) {
    const version: string | null = await invoke("check_for_update");
    if (version) {
      toast.showPersistent(`v${version} is available`, {
        label: "Download",
        handler: () => openUrl("https://github.com/JBolanle/PDFCompressor/releases/latest"),
      });
    } else if (showUpToDate) {
      toast.show("You're on the latest version");
    }
  }

  $: syncMenu("reveal-in-finder", selectedFile?.status === "done");
  $: syncMenu("clear-queue", $queue.length > 0);
  $: syncMenu("compress", $pendingCount > 0 && !isCompressing);
  $: syncMenu("reset-selected", selectedFile?.status === "done" || selectedFile?.status === "error");

  // ── menu event listeners ──────────────────────────────────────────────────

  let unlisteners: Array<() => void> = [];

  onMount(async () => {
    settings.load();
    window.addEventListener("keydown", onKeyDown);

    await checkAndShowUpdateToast();

    unlisteners = await Promise.all([
      listen("menu:add-files",        () => addFiles()),
      listen("menu:compress",         () => triggerCompress()),
      listen("menu:reveal-in-finder", () => revealSelected()),
      listen("menu:clear-queue",      () => clearAll()),
      listen("menu:reset-selected",   () => resetSelected()),
      listen("menu:check-for-update", () => checkAndShowUpdateToast(true)),
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
