<script lang="ts">
  import { derived } from "svelte/store";
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { fly, slide } from "svelte/transition";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { queue, type Preset } from "$lib/stores/queueStore";
  import { selectedFileId } from "$lib/stores/selectionStore";
  import { settings } from "$lib/stores/settingsStore";
  import { toast } from "$lib/stores/toastStore";
  import { formatBytes } from "$lib/notification";
  import { revealInFinder } from "$lib/fileActions";

  const selectedFile = derived([queue, selectedFileId], ([$q, $id]) => $q.find((e) => e.id === $id) ?? null);

  const reducedMotion = typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const animatedPct = tweened(0, { duration: reducedMotion ? 0 : 700, easing: cubicOut });
  const animatedRatio = tweened(0, { duration: reducedMotion ? 0 : 750, easing: cubicOut });

  $: hasFiles = $queue.length > 0;

  function savingsPct(original: number, compressed: number): string {
    return `−${Math.round(((original - compressed) / original) * 100)}%`;
  }

  function resetToCompress() {
    if (!$selectedFile) return;
    queue.resetFile($selectedFile.id);
  }

  const presetDpiRanges: Record<Preset, [number, number, number]> = {
    max:      [50,  72,  100],
    balanced: [100, 150, 200],
    minimal:  [200, 300, 400],
  };

  const presetInfo: Record<Preset, { label: string; desc: string; dpiRange: string }> = {
    max:      { label: "Max",      desc: "Smallest file",  dpiRange: "50–100 dpi"  },
    balanced: { label: "Balanced", desc: "Good trade-off", dpiRange: "100–200 dpi" },
    minimal:  { label: "Minimal",  desc: "Best quality",   dpiRange: "200–400 dpi" },
  };

  $: currentPreset = $selectedFile?.preset ?? "balanced";
  $: sliderMin = presetDpiRanges[currentPreset][0];
  $: sliderMax = presetDpiRanges[currentPreset][2];
  $: sliderValue = $selectedFile?.dpiOverride ?? presetDpiRanges[currentPreset][1];

  $: {
    if ($selectedFile?.status === "done" && $selectedFile.compressedSize !== undefined) {
      const pct = Math.round((($selectedFile.size - $selectedFile.compressedSize) / $selectedFile.size) * 100);
      const ratio = ($selectedFile.size - $selectedFile.compressedSize) / $selectedFile.size;
      (async () => {
        await Promise.all([
          animatedPct.set(0, { duration: 0 }),
          animatedRatio.set(0, { duration: 0 }),
        ]);
        animatedPct.set(pct);
        animatedRatio.set(ratio);
      })();
    } else {
      animatedPct.set(0, { duration: 0 });
      animatedRatio.set(0, { duration: 0 });
    }
  }

  let outputMode: "same_as_source" | "custom_folder" = $settings.output_mode;
  let naming: "suffix" | "overwrite" = $settings.naming;
  let defaultPreset: Preset = $settings.default_preset;
  let advancedOpen = false;
  let quickActionInstalled = false;
  let quickActionBusy = false;

  $: outputMode = $settings.output_mode;
  $: naming = $settings.naming;
  $: defaultPreset = $settings.default_preset;

  // Refresh install state the first time the drawer opens (or any time
  // it re-opens — covers the case where the user installed/uninstalled
  // the workflow via Finder while the app was open).
  $: if (advancedOpen) {
    refreshQuickActionState();
  }

  async function refreshQuickActionState() {
    try {
      quickActionInstalled = await invoke<boolean>("is_quick_action_installed");
    } catch {
      quickActionInstalled = false;
    }
  }

  async function toggleQuickAction() {
    if (quickActionBusy) return;
    quickActionBusy = true;
    try {
      if (quickActionInstalled) {
        await invoke("uninstall_quick_action");
        toast.show("Quick Action removed from Finder");
      } else {
        await invoke("install_quick_action");
        toast.show("Quick Action installed — right-click any PDF in Finder");
      }
      await refreshQuickActionState();
    } catch (e) {
      toast.show(typeof e === "string" ? e : "Quick Action update failed");
    } finally {
      quickActionBusy = false;
    }
  }

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

      {#if $selectedFile.status === "processing"}
        <div class="progress-track"><div class="progress-bar"></div></div>
      {/if}

      <div class="sizes">
        <div class="size-row"><span class="label">Original</span><span>{formatBytes($selectedFile.size)}</span></div>
        {#if $selectedFile.status === "done" && $selectedFile.compressedSize !== undefined}
          <div class="result-block" in:fly={{ y: 6, duration: reducedMotion ? 0 : 280, easing: cubicOut }}>
            <div class="savings-pct">−{Math.round($animatedPct)}%</div>
            <div class="savings-bar-track" aria-hidden="true">
              <div class="savings-bar-fill" style="transform: scaleX({$animatedRatio})"></div>
            </div>
            <div class="size-story">
              {formatBytes($selectedFile.size)} → {formatBytes($selectedFile.compressedSize)}
              · saved {formatBytes($selectedFile.size - $selectedFile.compressedSize)}
            </div>
            <div class="result-actions">
              <button class="finder-btn" on:click={() => revealInFinder($selectedFile!.path)}>Show in Finder</button>
              <button class="recompress-btn" on:click={resetToCompress}>Re-compress</button>
            </div>
          </div>
        {/if}
        {#if $selectedFile.status === "error"}
          <div class="error-block">
            <span class="error-text">{$selectedFile.errorMsg ?? "Compression failed"}</span>
            <button class="retry-btn" on:click={resetToCompress}>Retry</button>
          </div>
        {/if}
      </div>
    </div>

    {#if $selectedFile.status === "pending" || $selectedFile.status === "processing"}
      <div class="quality-controls">
        <div class="section-label">Quality</div>
        <div class="preset-control">
          {#each (["max", "balanced", "minimal"] as Preset[]) as p}
            <button class="preset-btn" class:active={$selectedFile.preset === p} on:click={() => onPresetChange(p)}>
              <span class="preset-name">{presetInfo[p].label}</span>
              <span class="preset-meta">{presetInfo[p].desc} · {presetInfo[p].dpiRange}</span>
            </button>
          {/each}
        </div>
        {#key currentPreset}
          <input type="range" class="dpi-slider" min={sliderMin} max={sliderMax} value={sliderValue} on:input={onSliderChange} />
        {/key}
        <div class="slider-range">
          <span>{sliderMin} dpi</span>
          <span>{sliderMax} dpi</span>
        </div>
        <div class="dpi-row">
          <span class="dpi-label">{sliderValue} DPI</span>
          <button class="apply-all-btn" on:click={applyToAll}>Apply DPI to all files</button>
        </div>
      </div>
    {/if}
  {:else if hasFiles}
    <div class="empty-state">
      <span>Select a file to configure</span>
    </div>
  {:else}
    <div class="empty-state onboarding">
      <span class="onboard-title">No files yet</span>
      <span class="onboard-sub">Drop PDFs onto the sidebar or<br>click "+ Add files" to get started</span>
    </div>
  {/if}

  <div class="settings-section">
    <div class="setting-row">
      <span class="setting-label">Output</span>
      <div class="segmented" role="radiogroup" aria-label="Output folder">
        <label class="segment" class:active={outputMode === "same_as_source"}>
          <input type="radio" name="output_mode" bind:group={outputMode} value="same_as_source"
            on:change={() => settings.save({ ...$settings, output_mode: outputMode })} />
          <span>Same as source</span>
        </label>
        <label class="segment" class:active={outputMode === "custom_folder"}>
          <input type="radio" name="output_mode" bind:group={outputMode} value="custom_folder"
            on:change={() => settings.save({ ...$settings, output_mode: outputMode })} />
          <span>Custom folder</span>
        </label>
      </div>
    </div>

    {#if $settings.output_mode === "custom_folder"}
      <div class="setting-row setting-row--detail">
        <span class="setting-label" aria-hidden="true"></span>
        <div class="folder-row">
          <span class="folder-path" title={$settings.output_folder ?? ""}>{$settings.output_folder ?? "No folder selected"}</span>
          <button on:click={pickFolder}>Choose…</button>
        </div>
      </div>
    {/if}

    <div class="setting-row">
      <span class="setting-label">Naming</span>
      <div class="segmented" role="radiogroup" aria-label="File naming">
        <label class="segment" class:active={naming === "suffix"}>
          <input type="radio" name="naming" bind:group={naming} value="suffix"
            on:change={() => settings.save({ ...$settings, naming })} />
          <span><code>_compressed</code></span>
        </label>
        <label class="segment" class:active={naming === "overwrite"}>
          <input type="radio" name="naming" bind:group={naming} value="overwrite"
            on:change={() => settings.save({ ...$settings, naming })} />
          <span>Overwrite original</span>
        </label>
      </div>
    </div>

    {#if naming === "overwrite"}
      <div class="setting-row setting-row--detail">
        <span class="setting-label" aria-hidden="true"></span>
        <p class="overwrite-warn">Original replaced; cannot be recovered.</p>
      </div>
    {/if}

    <button type="button" class="advanced-toggle" on:click={() => (advancedOpen = !advancedOpen)}
      aria-expanded={advancedOpen} aria-controls="advanced-content">
      <svg class="chevron" class:open={advancedOpen} viewBox="0 0 8 12" aria-hidden="true">
        <path d="M2 1l4 5-4 5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      Advanced
    </button>

    {#if advancedOpen}
      <div id="advanced-content" class="advanced-content" transition:slide={{ duration: reducedMotion ? 0 : 180, easing: cubicOut }}>
        <div class="setting-row">
          <span class="setting-label">Default preset</span>
          <div class="segmented" role="radiogroup" aria-label="Finder right-click preset">
            {#each (["max", "balanced", "minimal"] as Preset[]) as p}
              <label class="segment" class:active={defaultPreset === p}>
                <input type="radio" name="default_preset" bind:group={defaultPreset} value={p}
                  on:change={() => settings.save({ ...$settings, default_preset: defaultPreset })} />
                <span>{presetInfo[p].label}</span>
              </label>
            {/each}
          </div>
        </div>

        <div class="setting-row setting-row--detail">
          <span class="setting-label" aria-hidden="true"></span>
          <p class="setting-hint">Applied when you right-click a PDF in Finder and choose <em>Open With → compress[pdf]</em>. In-app files keep the per-file Quality preset above.</p>
        </div>

        <div class="setting-row">
          <span class="setting-label">Quick Action</span>
          <div class="quick-action-row">
            <span class="quick-action-status" class:installed={quickActionInstalled}>
              {quickActionInstalled ? "Installed" : "Not installed"}
            </span>
            <button class="quick-action-btn" on:click={toggleQuickAction} disabled={quickActionBusy}>
              {quickActionBusy ? "…" : quickActionInstalled ? "Remove" : "Install"}
            </button>
          </div>
        </div>

        <div class="setting-row setting-row--detail">
          <span class="setting-label" aria-hidden="true"></span>
          <p class="setting-hint">Adds a top-level <em>Compress with compress[pdf]</em> entry to Finder's right-click menu so you don't have to navigate the Open With submenu.</p>
        </div>
      </div>
    {/if}
  </div>
</section>

<style>
  .detail-panel { flex: 1; padding: 16px; display: flex; flex-direction: column; gap: 16px; overflow-y: auto; min-width: 0; border-top: 1px solid var(--border); }
  .detail-panel > * { max-width: 560px; width: 100%; }

  .filename {
    font-family: var(--font-display);
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .progress-track {
    height: 2px;
    background: var(--border);
    border-radius: 1px;
    overflow: hidden;
    margin: 4px 0;
  }
  .progress-bar {
    height: 100%;
    width: 40%;
    background: var(--accent);
    border-radius: 1px;
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    0%   { transform: translateX(-100%); }
    100% { transform: translateX(350%); }
  }

  .sizes { display: flex; flex-direction: column; gap: 4px; }
  .size-row { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-secondary); }
  .label { color: var(--text-tertiary); }

  .result-block { display: flex; flex-direction: column; gap: 6px; margin-top: 4px; }
  .savings-bar-track {
    height: 2px;
    background: var(--border);
    border-radius: 1px;
    overflow: hidden;
    margin: 0 0 2px;
  }
  .savings-bar-fill {
    height: 100%;
    width: 100%;
    background: var(--success);
    border-radius: 1px;
    transform-origin: left;
    transform: scaleX(0);
  }
  .savings-pct {
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: var(--weight-bold);
    color: var(--success);
    line-height: 1;
  }
  .size-story { font-size: var(--text-sm); color: var(--text-secondary); }
  .result-actions { display: flex; gap: 6px; margin-top: 2px; }

  .finder-btn {
    padding: 5px 10px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 12px;
  }
  .finder-btn:hover { background: var(--bg-tertiary); }

  .recompress-btn {
    padding: 5px 10px;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: var(--text-sm);
  }
  .recompress-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .error-block {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 6px;
    padding: 8px 10px;
    background: oklch(63% 0.22 25 / 0.1);
    border-radius: var(--radius-sm);
    border: 1px solid oklch(63% 0.22 25 / 0.25);
  }
  .error-text {
    font-size: var(--text-sm);
    color: var(--error);
    flex: 1;
  }
  .retry-btn {
    background: none;
    border: 1px solid var(--error);
    color: var(--error);
    cursor: pointer;
    font-size: var(--text-xs);
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }
  .retry-btn:hover { background: oklch(63% 0.22 25 / 0.15); }

  .quality-controls { display: flex; flex-direction: column; gap: 12px; padding: 16px 0; border-top: 1px solid var(--border); }

  .section-label {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    letter-spacing: 0.01em;
    color: var(--text-tertiary);
  }

  .preset-control { display: flex; gap: 4px; }
  .preset-btn {
    flex: 1;
    padding: 12px 4px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .preset-btn.active { background: var(--accent); border-color: var(--accent); color: white; }
  .preset-name { font-size: var(--text-base); font-weight: var(--weight-semibold); }
  .preset-meta { font-size: var(--text-xs); color: var(--text-secondary); opacity: 0.7; }
  .preset-btn.active .preset-meta { color: rgba(255, 255, 255, 0.75); opacity: 1; }

  .dpi-slider {
    width: 100%;
    -webkit-appearance: none;
    appearance: none;
    height: 3px;
    background: var(--border);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .dpi-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
    border: 2px solid var(--bg-primary);
    box-shadow: 0 0 0 1px var(--accent);
    transition: transform 0.1s, box-shadow 0.1s;
  }
  .dpi-slider::-webkit-slider-thumb:hover {
    transform: scale(1.15);
    box-shadow: 0 0 0 3px var(--accent-muted);
  }
  .dpi-slider:active::-webkit-slider-thumb {
    transform: scale(0.95);
  }

  .slider-range {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    margin-top: -4px;
  }

  .dpi-row { display: flex; justify-content: space-between; align-items: center; }
  .dpi-label { font-size: var(--text-xs); color: var(--text-tertiary); }
  .apply-all-btn {
    background: none;
    border: 1px solid var(--accent);
    color: var(--accent);
    cursor: pointer;
    font-size: var(--text-xs);
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    transition: background 0.1s;
  }
  .apply-all-btn:hover { background: var(--accent-muted); }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }
  .onboarding { gap: 6px; }
  .onboard-title { font-size: 13px; color: var(--text-secondary); font-weight: var(--weight-medium); }
  .onboard-sub { font-size: 11px; color: var(--text-tertiary); text-align: center; line-height: 1.5; }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-top: var(--space-4);
    border-top: 1px solid var(--border-subtle);
    margin-top: auto;
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .setting-label {
    flex: 0 0 88px;
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    letter-spacing: 0.01em;
  }
  .setting-row--detail {
    margin-top: -6px;
  }

  .segmented {
    flex: 1;
    display: flex;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px;
    gap: 2px;
  }
  .segment {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 5px 8px;
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    border-radius: 3px;
    text-align: center;
    transition: background 120ms ease, color 120ms ease;
    user-select: none;
  }
  .segment input[type="radio"] {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
    pointer-events: none;
  }
  .segment:hover { color: var(--text-secondary); }
  .segment.active {
    background: var(--bg-overlay);
    color: var(--text-primary);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .segment code {
    font-family: inherit;
    font-size: inherit;
    color: inherit;
    background: none;
    padding: 0;
  }

  .overwrite-warn {
    font-size: var(--text-xs);
    color: var(--warning);
    line-height: 1.4;
    margin: 0;
  }
  .setting-hint {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    line-height: 1.5;
    margin: 0;
  }
  .setting-hint em {
    font-style: normal;
    color: var(--text-secondary);
  }

  .advanced-toggle {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    align-self: flex-start;
    background: none;
    border: 0;
    padding: 2px 0;
    font-family: inherit;
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color 120ms ease;
    margin-top: var(--space-1);
  }
  .advanced-toggle:hover { color: var(--text-secondary); }
  .chevron {
    width: 8px;
    height: 8px;
    transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
  }
  .chevron.open { transform: rotate(90deg); }

  .advanced-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .quick-action-row {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    min-width: 0;
  }
  .quick-action-status {
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .quick-action-status.installed {
    color: var(--accent);
  }
  .quick-action-btn {
    padding: 3px 12px;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-family: inherit;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--bg-overlay);
    color: var(--text-primary);
    transition: background 120ms ease, border-color 120ms ease;
    min-width: 72px;
  }
  .quick-action-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }
  .quick-action-btn:disabled {
    cursor: default;
    opacity: 0.5;
  }

  .folder-row {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }
  .folder-path {
    flex: 1;
    font-size: var(--text-sm);
    color: var(--accent);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
  }
  .folder-row button {
    padding: 3px 10px;
    border-radius: 3px;
    font-size: var(--text-sm);
    font-family: inherit;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--bg-overlay);
    color: var(--text-primary);
    transition: background 120ms ease, border-color 120ms ease;
  }
  .folder-row button:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
  }
</style>
