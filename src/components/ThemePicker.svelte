<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { scale } from "svelte/transition";
  import { t } from "../lib/i18n/index.svelte";
  import { store } from "../lib/store.svelte";
  import { THEMES } from "../lib/themes";

  interface Props {
    anchor: HTMLElement;
    onclose: () => void;
  }
  let { anchor, onclose }: Props = $props();
  const r = $derived(anchor.getBoundingClientRect());
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onpointerdown={onclose}></div>
<div
  class="picker"
  role="dialog"
  aria-label={t("list.theme")}
  tabindex="-1"
  style="top:{r.bottom + 8}px; left:{Math.max(8, Math.min(r.right - 292, window.innerWidth - 300))}px"
  transition:scale={{ start: 0.95, duration: 130 }}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <div class="heading">{t("list.theme")}</div>
  <div class="grid">
    {#each THEMES as theme (theme.key)}
      <button
        class="swatch"
        style="background:{theme.background}"
        aria-label={theme.key}
        title={theme.key}
        onclick={() => store.setTheme(theme.key)}
      >
        {#if store.theme.key === theme.key}<span class="tick"><Check size={16} strokeWidth={3} /></span>{/if}
      </button>
    {/each}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
  }
  .picker {
    position: fixed;
    z-index: 81;
    width: 292px;
    padding: 14px;
    border-radius: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-lg);
    transform-origin: top right;
  }
  .heading {
    font-weight: 600;
    margin-bottom: 10px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 8px;
  }
  .swatch {
    aspect-ratio: 1;
    border-radius: 8px;
    display: grid;
    place-items: center;
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
    transition: transform 0.15s var(--ease);
  }
  .swatch:hover {
    transform: scale(1.08);
  }
  .tick {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.92);
    color: #1d2129;
  }
</style>
