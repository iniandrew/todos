<script lang="ts">
  import { ChevronRight, Check } from "@lucide/svelte";
  import { onMount, tick } from "svelte";
  import type { MenuEntry, MenuItem } from "../lib/menu.svelte";
  import { scale } from "svelte/transition";

  interface Props {
    x: number;
    y: number;
    items: MenuEntry[];
    onclose: () => void;
  }
  let { x, y, items, onclose }: Props = $props();

  let el: HTMLDivElement | undefined = $state();
  let pos = $state({ left: 0, top: 0 });
  let expanded = $state<string | null>(null);

  async function place() {
    await tick();
    if (!el) return;
    const r = el.getBoundingClientRect();
    pos = {
      left: Math.max(8, Math.min(x, window.innerWidth - r.width - 8)),
      top: Math.max(8, Math.min(y, window.innerHeight - r.height - 8)),
    };
  }

  onMount(() => {
    pos = { left: x, top: y };
    place();
    el?.querySelector<HTMLButtonElement>("button")?.focus();
  });

  $effect(() => {
    void expanded;
    place();
  });

  function choose(item: MenuItem) {
    if (item.children) {
      expanded = expanded === item.label ? null : item.label;
      return;
    }
    onclose();
    item.run?.();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      onclose();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const buttons = [...(el?.querySelectorAll<HTMLButtonElement>("button") ?? [])];
      const i = buttons.indexOf(document.activeElement as HTMLButtonElement);
      const next = (i + (e.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length;
      buttons[next]?.focus();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="backdrop"
  onpointerdown={onclose}
  oncontextmenu={(e) => {
    e.preventDefault();
    onclose();
  }}
></div>
<div
  bind:this={el}
  class="menu"
  role="menu"
  tabindex="-1"
  style="left:{pos.left}px; top:{pos.top}px"
  transition:scale={{ start: 0.97, duration: 140, opacity: 0 }}
  {onkeydown}
>
  {#each items as item, i (i)}
    {#if item === "divider"}
      <div class="divider"></div>
    {:else}
      <button role="menuitem" class="item" class:danger={item.danger} onclick={() => choose(item)}>
        {#if item.icon}<item.icon size={16} strokeWidth={1.8} />{:else}<span class="spacer"></span>{/if}
        <span class="label">{item.label}</span>
        {#if item.shortcut}<kbd>{item.shortcut}</kbd>{/if}
        {#if item.checked}<Check size={15} />{/if}
        {#if item.children}<span class="chev" class:open={expanded === item.label}><ChevronRight size={15} /></span>{/if}
      </button>
      {#if item.children && expanded === item.label}
        <div class="sub">
          {#each item.children as child, j (j)}
            <button role="menuitem" class="item" onclick={() => choose(child)}>
              {#if child.icon}<child.icon size={15} strokeWidth={1.8} />{:else}<span class="spacer"></span>{/if}
              <span class="label">{child.label}</span>
              {#if child.checked}<Check size={15} />{/if}
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  {/each}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
  }
  .menu {
    position: fixed;
    z-index: 91;
    min-width: 220px;
    max-width: 300px;
    max-height: calc(100vh - 16px);
    overflow-y: auto;
    padding: 5px;
    border-radius: 11px;
    background: var(--menu-bg);
    backdrop-filter: blur(30px) saturate(180%);
    -webkit-backdrop-filter: blur(30px) saturate(180%);
    box-shadow: var(--shadow-lg);
    transform-origin: top left;
    font-size: 13.5px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 28px;
    padding: 0 9px;
    border-radius: 6px;
    text-align: left;
    color: var(--text);
  }
  .item :global(svg) {
    color: var(--text-2);
    flex: none;
  }
  .item:hover,
  .item:focus-visible {
    background: var(--accent);
    color: #fff;
    outline: none;
  }
  .item:hover :global(svg),
  .item:focus-visible :global(svg) {
    color: #fff;
  }
  .item:hover kbd,
  .item:focus-visible kbd {
    background: transparent;
    color: rgba(255, 255, 255, 0.85);
  }
  .danger,
  .danger :global(svg) {
    color: var(--danger);
  }
  .danger:hover,
  .danger:focus-visible {
    background: var(--danger);
  }
  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .spacer {
    width: 16px;
  }
  kbd {
    background: transparent;
    color: var(--text-3);
    padding: 0;
  }
  .chev {
    display: grid;
    transition: transform 0.15s;
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .sub {
    margin: 2px 0 2px 14px;
  }
  .divider {
    height: 1px;
    margin: 5px 9px;
    background: var(--separator);
  }
</style>
