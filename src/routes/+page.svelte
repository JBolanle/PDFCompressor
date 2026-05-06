<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DetailPanel from "$lib/components/DetailPanel.svelte";
  import ActionBar from "$lib/components/ActionBar.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import { settings } from "$lib/stores/settingsStore";

  onMount(() => settings.load());
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
