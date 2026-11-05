<script lang="ts">
  import { CalendarDays, House, List, PanelLeftClose, Pencil, Plus, Search, Settings, Star, Sun, Trash2, X } from "@lucide/svelte";
  import type { Component } from "svelte";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { t } from "../lib/i18n/index.svelte";
  import { menu } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";
  import { DEFAULT_VIEW_THEMES, getTheme } from "../lib/themes";
  import type { SmartView, TaskList, ViewId } from "../lib/types";

  let searchInput: HTMLInputElement | undefined = $state();
  $effect(() => {
    if (store.focusSearch) searchInput?.focus();
  });

  const smart: { id: SmartView; icon: Component<any>; key: "myDay" | "important" | "planned" | "tasks" }[] = [
    { id: "myDay", icon: Sun, key: "myDay" },
    { id: "important", icon: Star, key: "important" },
    { id: "planned", icon: CalendarDays, key: "planned" },
    { id: "tasks", icon: House, key: "tasks" },
  ];

  const smartAccent = (id: SmartView) =>
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

<aside class="sidebar">
  <div class="top">
    <div class="brand">
      <img src="/favicon.svg" alt="" width="26" height="26" />
      <span>{t("app.name")}</span>
    </div>
    <button class="icon-btn" title={t("sidebar.toggle")} onclick={() => (store.sidebarOpen = false)}>
      <PanelLeftClose size={18} />
    </button>
  </div>

  <label class="search" class:active={store.searching}>
    <Search size={16} />
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
      <button class="clear" aria-label={t("detail.clear")} onclick={() => store.setSearch("")}><X size={14} /></button>
    {/if}
  </label>

  <nav>
    {#each smart as item (item.id)}
      {@const count = store.counts[item.key]}
      <button
        class="nav-item"
        class:active={active(item.id)}
        style="--item-accent:{smartAccent(item.id)}"
        onclick={() => store.setView(item.id)}
      >
        <item.icon size={18} strokeWidth={1.8} />
        <span class="name">{t(`view.${item.key}`)}</span>
        {#if count}<span class="count">{count}</span>{/if}
      </button>
    {/each}

    <div class="divider"></div>

    <div
      class="lists"
      use:dndzone={{ items: lists, flipDurationMs: 150, dropTargetStyle: {}, type: "lists" }}
      onconsider={consider}
      onfinalize={finalize}
    >
      {#each lists as list (list.id)}
        {@const count = store.counts.lists[list.id]}
        <div animate:flip={{ duration: 150 }}>
          <button
            class="nav-item"
            class:active={active(`list:${list.id}`)}
            style="--item-accent:{getTheme(list.theme).accent}"
            onclick={() => store.setView(`list:${list.id}`)}
            oncontextmenu={(e) => listMenu(e, list)}
          >
            <span class="dot" style="background:{getTheme(list.theme).background}"><List size={12} strokeWidth={2.4} /></span>
            <span class="name">{list.name}</span>
            {#if count}<span class="count">{count}</span>{/if}
          </button>
        </div>
      {/each}
    </div>
  </nav>

  <div class="bottom">
    <button class="new-list" onclick={() => store.createList()}>
      <Plus size={18} />
      <span>{t("sidebar.newList")}</span>
    </button>
    <button class="icon-btn" title={t("sidebar.settings")} onclick={() => (store.settingsOpen = true)}>
      <Settings size={18} />
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 280px;
    flex: none;
    display: flex;
    flex-direction: column;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border);
    height: 100%;
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 12px 8px 16px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 700;
    font-size: 16px;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 4px 12px 10px;
    padding: 0 10px;
    height: 34px;
    border-radius: 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-3);
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }
  .search:focus-within,
  .search.active {
    border-color: var(--accent, #2f6fed);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent, #2f6fed) 18%, transparent);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
  }
  .clear {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    color: var(--text-2);
  }
  .clear:hover {
    background: var(--surface-hover);
  }
  nav {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px;
  }
  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    height: 38px;
    padding: 0 10px 0 12px;
    border-radius: 7px;
    color: var(--text);
    text-align: left;
    transition: background 0.12s;
  }
  .nav-item :global(svg) {
    color: var(--item-accent);
    flex: none;
  }
  .nav-item:hover {
    background: var(--surface-hover);
  }
  .nav-item.active {
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    font-weight: 600;
  }
  .nav-item.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 3px;
    background: var(--item-accent);
  }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    min-width: 20px;
    padding: 0 6px;
    border-radius: 10px;
    font-size: 12px;
    text-align: center;
    color: var(--text-2);
    background: color-mix(in srgb, var(--text-3) 15%, transparent);
    font-weight: 500;
  }
  .dot {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 6px;
    flex: none;
  }
  .dot :global(svg) {
    color: #fff !important;
  }
  .divider {
    height: 1px;
    margin: 8px 6px;
    background: var(--border);
  }
  .lists {
    min-height: 8px;
    outline: none;
  }
  .bottom {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px;
    border-top: 1px solid var(--border);
  }
  .new-list {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    height: 38px;
    padding: 0 12px;
    border-radius: 7px;
    color: var(--accent, #2f6fed);
    font-weight: 500;
  }
  .new-list:hover {
    background: var(--surface-hover);
  }
</style>
