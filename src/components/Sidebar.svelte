<script lang="ts">
  import { ArrowDownToLine, CalendarDays, House, PanelLeftClose, Pencil, Plus, Search, Settings, Star, Sun, Trash2, X } from "@lucide/svelte";
  import type { Component } from "svelte";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { t } from "../lib/i18n/index.svelte";
  import { listIcon } from "../lib/listIcons";
  import { menu } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";
  import { updates } from "../lib/updates.svelte";
  import { DEFAULT_VIEW_THEMES, getTheme } from "../lib/themes";
  import type { SmartView, TaskList, ViewId } from "../lib/types";

  let { macOverlay = false }: { macOverlay?: boolean } = $props();

  let searchInput: HTMLInputElement | undefined = $state();
  $effect(() => {
    if (store.focusSearch) searchInput?.focus();
  });

  const smart: { id: SmartView; icon: Component<any> }[] = [
    { id: "myDay", icon: Sun },
    { id: "important", icon: Star },
    { id: "planned", icon: CalendarDays },
    { id: "tasks", icon: House },
  ];

  const smartColor = (id: SmartView) =>
    getTheme(store.settings[`viewTheme.${id}`] ?? DEFAULT_VIEW_THEMES[id]).accent;

  let lists = $state<TaskList[]>([]);
  $effect(() => {
    lists = store.customLists;
  });

  function consider(e: CustomEvent<DndEvent<TaskList>>) {
    lists = e.detail.items;
  }
  function finalize(e: CustomEvent<DndEvent<TaskList>>) {
    lists = e.detail.items;
    store.reorderLists(lists);
  }

  const active = (id: ViewId) => !store.searching && store.view === id;

  function listMenu(e: MouseEvent, list: TaskList) {
    menu.open(e, [
      {
        label: t("list.rename"),
        icon: Pencil,
        run: () => {
          store.setView(`list:${list.id}`);
          store.renamingListId = list.id;
        },
      },
      "divider",
      { label: t("list.delete"), icon: Trash2, danger: true, run: () => store.deleteList(list) },
    ]);
  }
</script>

<aside class="sidebar" class:mac={macOverlay}>
  <div class="top" data-tauri-drag-region>
    <label class="search">
      <Search size={14} strokeWidth={2.2} />
      <input
        bind:this={searchInput}
        value={store.searchQuery}
        placeholder={t("sidebar.search")}
        oninput={(e) => store.setSearch(e.currentTarget.value)}
        onkeydown={(e) => {
          if (e.key === "Escape") {
            store.setSearch("");
            e.currentTarget.blur();
          }
        }}
      />
      {#if store.searchQuery}
        <button class="clear" aria-label={t("detail.clear")} onclick={() => store.setSearch("")}>
          <X size={10} strokeWidth={3} />
        </button>
      {/if}
    </label>
    <button class="tool-btn" title={t("sidebar.toggle")} onclick={() => (store.sidebarOpen = false)}>
      <PanelLeftClose size={17} strokeWidth={1.8} />
    </button>
  </div>

  <nav>
    <div class="tiles">
      {#each smart as item (item.id)}
        {@const count = store.counts[item.id]}
        <button
          class="tile"
          class:active={active(item.id)}
          style="--c:{smartColor(item.id)}"
          onclick={() => store.setView(item.id)}
        >
          <span class="tile-icon"><item.icon size={15} strokeWidth={2.4} fill={item.id === "important" ? "currentColor" : "none"} /></span>
          <span class="tile-count">{count}</span>
          <span class="tile-name">{t(`view.${item.id}`)}</span>
        </button>
      {/each}
    </div>

    {#if lists.length}
      <div class="section">{t("sidebar.myLists")}</div>
    {/if}
    <div
      class="lists"
      use:dndzone={{ items: lists, flipDurationMs: 160, dropTargetStyle: {}, type: "lists" }}
      onconsider={consider}
      onfinalize={finalize}
    >
      {#each lists as list (list.id)}
        {@const count = store.counts.lists[list.id]}
        {@const Icon = listIcon(list.icon)}
        <div animate:flip={{ duration: 160 }}>
          <button
            class="row"
            class:active={active(`list:${list.id}`)}
            onclick={() => store.setView(`list:${list.id}`)}
            oncontextmenu={(e) => listMenu(e, list)}
          >
            <span class="dot" style="background:{getTheme(list.theme).accent}"><Icon size={13} strokeWidth={2.4} /></span>
            <span class="name">{list.name}</span>
            {#if count}<span class="count">{count}</span>{/if}
          </button>
        </div>
      {/each}
    </div>
  </nav>

  {#if updates.available && (updates.status === "available" || updates.status === "downloading" || updates.status === "installing")}
    <button class="update" onclick={() => (updates.dialogOpen = true)}>
      <span class="update-icon"><ArrowDownToLine size={15} strokeWidth={2.4} /></span>
      <span class="update-text">
        <b>{t("update.banner")}</b>
        <small>Todos {updates.available.version}</small>
      </span>
      <span class="update-view">{t("update.view")}</span>
    </button>
  {/if}

  <div class="bottom">
    <button class="add-list" onclick={() => store.createList()}>
      <span class="plus"><Plus size={13} strokeWidth={3} /></span>
      {t("sidebar.newList")}
    </button>
    <button class="tool-btn" title={t("sidebar.settings")} onclick={() => (store.settingsOpen = true)}>
      <Settings size={17} strokeWidth={1.8} />
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 268px;
    flex: none;
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--separator);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 14px 10px 10px 14px;
  }
  .mac .top {
    padding-top: 38px;
  }
  .search {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 8px;
    border-radius: 8px;
    background: var(--fill);
    color: var(--text-2);
    transition: box-shadow 0.2s;
  }
  .search:focus-within {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 35%, transparent);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    font-size: 13.5px;
    color: var(--text);
  }
  .search input::placeholder {
    color: var(--text-2);
  }
  .clear {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-3);
    color: var(--sidebar-bg);
  }
  nav {
    flex: 1;
    overflow-y: auto;
    padding: 2px 12px 12px;
  }
  .tiles {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin-bottom: 18px;
  }
  .tile {
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-rows: auto auto;
    gap: 6px 4px;
    padding: 9px 10px 8px;
    border-radius: 11px;
    background: var(--cell);
    box-shadow: var(--shadow-sm);
    text-align: left;
    transition:
      background 0.2s,
      transform 0.15s var(--ease),
      box-shadow 0.2s;
  }
  :global([data-theme="dark"]) .tile {
    background: var(--fill-2);
    box-shadow: none;
  }
  .tile:active {
    transform: scale(0.97);
  }
  .tile-icon {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--c);
    color: #fff;
  }
  .tile-count {
    font-size: 20px;
    font-weight: 700;
    line-height: 26px;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }
  .tile-name {
    grid-column: 1 / -1;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-2);
  }
  .tile.active {
    background: var(--c) !important;
    box-shadow: 0 4px 14px color-mix(in srgb, var(--c) 35%, transparent);
  }
  .tile.active .tile-icon {
    background: #fff;
    color: var(--c);
  }
  .tile.active .tile-count,
  .tile.active .tile-name {
    color: #fff;
  }
  .section {
    padding: 0 8px 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
  }
  .lists {
    min-height: 4px;
    outline: none;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 36px;
    padding: 0 10px 0 8px;
    border-radius: 8px;
    text-align: left;
    color: var(--text);
  }
  .row:hover {
    background: var(--hover);
  }
  .row.active {
    background: var(--selected);
  }
  .dot {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    color: #fff;
    flex: none;
  }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
  }
  .row.active .name {
    font-weight: 600;
  }
  .count {
    font-size: 13px;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
  }
  .update {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 12px 4px;
    padding: 9px 10px;
    border-radius: 11px;
    background: var(--cell);
    box-shadow: var(--shadow-sm);
    text-align: left;
    animation: rise 0.35s var(--spring);
  }
  :global([data-theme="dark"]) .update {
    background: var(--fill-2);
  }
  @keyframes rise {
    from {
      transform: translateY(8px);
      opacity: 0;
    }
  }
  .update-icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
    flex: none;
  }
  .update-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    font-size: 13px;
  }
  .update-text small {
    font-size: 12px;
    color: var(--text-2);
  }
  .update-view {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
  }
  .bottom {
    display: flex;
    align-items: center;
    padding: 8px 10px 10px 14px;
  }
  .add-list {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-2);
    text-align: left;
    transition: color 0.15s;
  }
  .add-list:hover {
    color: var(--text);
  }
  .plus {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--fill);
  }
</style>
