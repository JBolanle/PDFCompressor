<script lang="ts">
  import { toast } from "$lib/stores/toastStore";
</script>

<div class="toast-container" aria-live="polite">
  {#each $toast as msg (msg.id)}
    <div class="toast" role="alert">
      <span>{msg.message}</span>
      <button on:click={() => toast.dismiss(msg.id)} aria-label="Dismiss">✕</button>
    </div>
  {/each}
</div>

<style>
  .toast-container { position: fixed; bottom: 60px; left: 50%; transform: translateX(-50%); display: flex; flex-direction: column; gap: 8px; z-index: 200; pointer-events: none; }
  .toast { background: var(--bg-tertiary); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 8px 12px; display: flex; align-items: center; gap: 10px; font-size: 12px; color: var(--text-primary); pointer-events: all; max-width: 340px; box-shadow: 0 4px 16px rgba(0,0,0,0.4); }
  .toast button { background: none; border: none; color: var(--text-tertiary); cursor: pointer; font-size: 10px; padding: 2px; flex-shrink: 0; }
</style>
