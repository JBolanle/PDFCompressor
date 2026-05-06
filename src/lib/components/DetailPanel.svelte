<script lang="ts">
  import { derived } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { queue, type Preset } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";

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
</script>

<section class="detail-panel">
  {#if !$selectedFile}
    <div class="empty-state">Select a file from the queue</div>
  {:else}
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
        <div class="dpi-label">{sliderValue} DPI</div>
      </div>
    {/if}
  {/if}
</section>

<style>
  .detail-panel { flex: 1; padding: 16px; display: flex; flex-direction: column; gap: 16px; overflow-y: auto; }
  .empty-state { flex: 1; display: flex; align-items: center; justify-content: center; color: var(--text-tertiary); font-size: 12px; }
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
  .dpi-label { font-size: 10px; color: var(--text-tertiary); text-align: right; }
</style>
