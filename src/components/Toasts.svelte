<script lang="ts">
  import { Bell, Info, Trash2, X } from "@lucide/svelte";
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import { store } from "../lib/store.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each store.toasts as toast (toast.id)}
    <div class="toast" animate:flip={{ duration: 200 }} in:fly={{ y: 20, duration: 220 }} out:fly={{ y: 10, duration: 160 }}>
      {#if toast.icon === "trash"}<Trash2 size={16} />{:else if toast.icon === "bell"}<Bell size={16} />{:else}<Info size={16} />{/if}
      <span class="msg">{toast.message}</span>
      {#if toast.action}
        <button
          class="action"
          onclick={() => {
            toast.action?.run();
            store.dismissToast(toast.id);
          }}>{toast.action.label}</button
        >
      {/if}
      <button class="close" aria-label="Dismiss" onclick={() => store.dismissToast(toast.id)}><X size={14} /></button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    z-index: 120;
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    pointer-events: none;
  }
  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 260px;
    max-width: min(520px, calc(100vw - 32px));
    padding: 10px 10px 10px 14px;
    border-radius: 10px;
    background: #23252b;
    color: #f2f3f5;
    box-shadow: var(--shadow-lg);
  }
  :global([data-theme="dark"]) .toast {
    background: #3a3c45;
  }
  .msg {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .action {
    padding: 4px 10px;
    border-radius: 6px;
    font-weight: 700;
    color: #8fb4ff;
  }
  .action:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .close {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
    opacity: 0.7;
  }
  .close:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.1);
  }
</style>
