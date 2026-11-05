<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade, scale } from "svelte/transition";

  interface Props {
    label: string;
    width?: number;
    onclose: () => void;
    children: Snippet;
  }
  let { label, width = 460, onclose, children }: Props = $props();
  let el: HTMLDivElement | undefined = $state();

  $effect(() => {
    const prev = document.activeElement as HTMLElement | null;
    el?.querySelector<HTMLElement>("[data-autofocus], button, input, select")?.focus();
    return () => prev?.focus?.();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" transition:fade={{ duration: 150 }} onpointerdown={(e) => e.target === e.currentTarget && onclose()}>
  <div
    bind:this={el}
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-label={label}
    tabindex="-1"
    style="width:min({width}px, calc(100vw - 32px))"
    transition:scale={{ start: 0.96, duration: 160 }}
    onkeydown={(e) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        onclose();
      }
    }}
  >
    {@render children()}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    background: var(--overlay);
    backdrop-filter: blur(2px);
    -webkit-backdrop-filter: blur(2px);
  }
  .dialog {
    max-height: calc(100vh - 48px);
    overflow-y: auto;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg);
    outline: none;
  }
</style>
