<script lang="ts">
  import { Bell, CircleCheck, Info, Trash2 } from "@lucide/svelte";
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import { store } from "../lib/store.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each store.toasts as toast (toast.id)}
    <div class="toast" animate:flip={{ duration: 220 }} in:fly={{ y: 24, duration: 320 }} out:fly={{ y: 12, duration: 180 }}>
      <span class="ico">
        {#if toast.icon === "trash"}<Trash2 size={15} />{:else if toast.icon === "bell"}<Bell size={15} />{:else if toast.icon === "check"}<CircleCheck size={15} />{:else}<Info size={15} />{/if}
      </span>
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
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    left: 50%;
    bottom: 22px;
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
    max-width: min(480px, calc(100vw - 32px));
    height: 42px;
    padding: 0 8px 0 16px;
    border-radius: 21px;
    background: var(--hud-bg);
    color: #fff;
    backdrop-filter: blur(24px) saturate(180%);
    -webkit-backdrop-filter: blur(24px) saturate(180%);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.25);
    font-size: 13.5px;
  }
  .ico {
    display: grid;
    opacity: 0.85;
  }
  .msg {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding-right: 8px;
  }
  .action {
    height: 28px;
    padding: 0 12px;
    border-radius: 14px;
    font-weight: 600;
    color: #fff;
    background: rgba(255, 255, 255, 0.16);
  }
  .action:hover {
    background: rgba(255, 255, 255, 0.26);
  }
</style>
