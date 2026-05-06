<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DetailPanel from "$lib/components/DetailPanel.svelte";
  import ActionBar from "$lib/components/ActionBar.svelte";
  import SettingsSheet from "$lib/components/SettingsSheet.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import { settings } from "$lib/stores/settingsStore";

  let showSettings = false;
  let selectedFileId: string | null = null;

  onMount(() => settings.load());
</script>

<div class="app">
  <div class="titlebar">
    <button class="gear-btn" on:click={() => showSettings = true} aria-label="Settings">⚙</button>
  </div>

  <div class="content">
    <Sidebar bind:selectedFileId />
    <DetailPanel {selectedFileId} />
  </div>

  <ActionBar />
</div>

{#if showSettings}
  <SettingsSheet on:close={() => showSettings = false} />
{/if}

<Toast />

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
</style>
