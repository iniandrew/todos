<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { scale } from "svelte/transition";
  import { t } from "../lib/i18n/index.svelte";
  import { LIST_ICONS, listIcon } from "../lib/listIcons";
  import { store } from "../lib/store.svelte";
  import { THEMES } from "../lib/themes";

  interface Props {
    anchor: HTMLElement;
    onclose: () => void;
  }
  let { anchor, onclose }: Props = $props();
  const r = $derived(anchor.getBoundingClientRect());
  const list = $derived(store.view.startsWith("list:") ? store.currentList : null);
  const Preview = $derived(listIcon(list?.icon));
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onpointerdown={onclose}></div>
<div
  class="pop"
  role="dialog"
  aria-label={t("list.theme")}
  tabindex="-1"
  style="top:{r.bottom + 8}px; left:{Math.max(8, Math.min(r.right - 300, window.innerWidth - 308))}px"
  transition:scale={{ start: 0.94, duration: 180, opacity: 0 }}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  {#if list}
    <div class="preview" style="background:{store.theme.accent}"><Preview size={30} strokeWidth={2.2} /></div>
    <div class="preview-name">{list.name}</div>
  {/if}

  <div class="label">{t("list.color")}</div>
  <div class="colors">
    {#each THEMES as theme (theme.key)}
      <button
        class="color"
        class:on={store.theme.key === theme.key}
        style="--c:{theme.accent}"
        aria-label={theme.key}
        title={theme.key}
        onclick={() => store.setTheme(theme.key)}
      >
        {#if store.theme.key === theme.key}<Check size={13} strokeWidth={3.4} />{/if}
      </button>
    {/each}
  </div>

  {#if list}
    <div class="label">{t("list.icon")}</div>
    <div class="icons">
      {#each Object.entries(LIST_ICONS) as [key, Icon] (key)}
        <button
          class="icon"
          class:on={(list.icon ?? "list") === key}
          aria-label={key}
          title={key}
          onclick={() => store.setListIcon(list.id, key)}
        >
          <Icon size={16} strokeWidth={2.1} />
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
  }
  .pop {
    position: fixed;
    z-index: 81;
    width: 300px;
    padding: 16px;
    border-radius: 14px;
    background: var(--menu-bg);
    backdrop-filter: blur(30px) saturate(180%);
    -webkit-backdrop-filter: blur(30px) saturate(180%);
    box-shadow: var(--shadow-lg);
    transform-origin: top right;
    outline: none;
  }
  .preview {
    display: grid;
    place-items: center;
    width: 64px;
    height: 64px;
    margin: 2px auto 8px;
    border-radius: 50%;
    color: #fff;
    box-shadow: 0 6px 18px color-mix(in srgb, currentColor 0%, rgba(0, 0, 0, 0.18));
    transition: background 0.25s;
  }
  .preview-name {
    text-align: center;
    font-weight: 700;
    font-size: 16px;
    margin-bottom: 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .label {
    margin: 10px 2px 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
  }
  .colors {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 10px;
    justify-items: center;
  }
  .color {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--c);
    color: #fff;
    transition: transform 0.2s var(--spring);
  }
  .color:hover {
    transform: scale(1.1);
  }
  .color.on {
    box-shadow:
      0 0 0 2px var(--elevated),
      0 0 0 4px var(--c);
  }
  .icons {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 6px;
    justify-items: center;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    color: var(--text-2);
    background: var(--fill-2);
    transition:
      background 0.15s,
      color 0.15s;
  }
  .icon:hover {
    background: var(--fill);
    color: var(--text);
  }
  .icon.on {
    background: var(--accent);
    color: #fff;
  }
</style>
