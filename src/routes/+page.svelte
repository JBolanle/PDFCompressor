<script lang="ts">
  import { onMount } from "svelte";
  import { settings } from "$lib/stores/settingsStore";

  // Components will be imported here as they're built
  // import Sidebar from "$lib/components/Sidebar.svelte";
  // import DetailPanel from "$lib/components/DetailPanel.svelte";
  // import ActionBar from "$lib/components/ActionBar.svelte";
  // import SettingsSheet from "$lib/components/SettingsSheet.svelte";
  // import Toast from "$lib/components/Toast.svelte";

  let showSettings = false;
  let selectedFileId: string | null = null;

  onMount(() => settings.load());
</script>

<div class="app">
  <div class="titlebar">
    <button class="gear-btn" on:click={() => showSettings = true} aria-label="Settings">
      ⚙
    </button>
  </div>

  <div class="content">
    <!-- Sidebar and DetailPanel will go here -->
    <div class="placeholder-sidebar">Queue</div>
    <div class="placeholder-detail">Detail</div>
  </div>

  <!-- ActionBar will go here -->
  <div class="placeholder-action-bar">Action Bar</div>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
  }

  .titlebar {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 12px;
    -webkit-app-region: drag;
    flex-shrink: 0;
  }

  .gear-btn {
    -webkit-app-region: no-drag;
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 14px;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    line-height: 1;
  }

  .gear-btn:hover {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }

  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
    border-top: 1px solid var(--border);
  }

  .placeholder-sidebar {
    width: var(--sidebar-width);
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    padding: 12px;
    color: var(--text-tertiary);
    font-size: 11px;
  }

  .placeholder-detail {
    flex: 1;
    padding: 12px;
    color: var(--text-tertiary);
    font-size: 11px;
  }

  .placeholder-action-bar {
    height: var(--action-bar-height);
    border-top: 1px solid var(--border);
    padding: 8px 12px;
    color: var(--text-tertiary);
    font-size: 11px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }
</style>
