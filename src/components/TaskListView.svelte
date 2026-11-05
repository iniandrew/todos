<script lang="ts">
  import { ChevronRight, Ellipsis, Eye, EyeOff, Palette, PanelLeft, Pencil, Trash2 } from "@lucide/svelte";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { fade, slide } from "svelte/transition";
  import { diffDays, isoToDateKey } from "../lib/dates";
  import { longDate } from "../lib/format";
  import { t, type TKey } from "../lib/i18n/index.svelte";
  import { menu, type MenuEntry } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";
  import type { Task } from "../lib/types";
  import AddTask from "./AddTask.svelte";
  import EmptyState from "./EmptyState.svelte";
  import ListCustomizer from "./ListCustomizer.svelte";
  import TaskItem from "./TaskItem.svelte";

  let { macOverlay = false }: { macOverlay?: boolean } = $props();

  const FLIP_MS = 220;

  let items = $state<Task[]>([]);
  $effect(() => {
    items = store.incomplete;
  });

  let completedOpen = $state(true);
  let customizeAnchor = $state<HTMLElement | null>(null);
  let optionsBtn: HTMLButtonElement | undefined = $state();
  let customizeBtn: HTMLButtonElement | undefined = $state();
  let renameValue = $state("");

  const showList = $derived(store.searching || !(store.view === "tasks" || store.view.startsWith("list:")));
  const isCustomList = $derived(store.view.startsWith("list:") && !store.searching);
  const renaming = $derived(isCustomList && store.renamingListId === store.currentList?.id);
  const openCount = $derived(store.incomplete.length);

  $effect(() => {
    if (renaming && store.currentList) renameValue = store.currentList.name;
  });

  // Planned view: bucket tasks by date.
  type Group = { key: TKey; date?: string; tasks: Task[] };
  const groups = $derived.by<Group[] | null>(() => {
    if (store.view !== "planned" || store.searching) return null;
    const buckets: Record<string, Task[]> = {};
    for (const task of store.incomplete) {
      const date = task.dueDate ?? (task.reminderAt ? isoToDateKey(task.reminderAt) : store.today);
      const d = diffDays(date, store.today);
      const key = d < 0 ? "overdue" : d === 0 ? "today" : d === 1 ? "tomorrow" : d < 7 ? "thisWeek" : "later";
      (buckets[key] ??= []).push(task);
    }
    return (["overdue", "today", "tomorrow", "thisWeek", "later"] as const)
      .filter((k) => buckets[k]?.length)
      .map((k) => ({ key: `planned.${k}` as TKey, tasks: buckets[k] }));
  });

  function consider(e: CustomEvent<DndEvent<Task>>) {
    items = e.detail.items;
  }
  function finalize(e: CustomEvent<DndEvent<Task>>) {
    items = e.detail.items;
    store.reorderTasks(items);
  }

  function openOptions() {
    if (!optionsBtn) return;
    const entries: MenuEntry[] = [];
    const list = store.currentList;
    if (isCustomList && list) {
      entries.push({ label: t("list.rename"), icon: Pencil, run: () => (store.renamingListId = list.id) });
    }
    entries.push(
      { label: t("list.theme"), icon: Palette, run: () => (customizeAnchor = customizeBtn ?? optionsBtn!) },
      {
        label: store.showCompleted ? t("list.hideCompleted") : t("list.showCompleted"),
        icon: store.showCompleted ? EyeOff : Eye,
        run: () => store.toggleShowCompleted(),
      },
    );
    if (isCustomList && list) {
      entries.push("divider", { label: t("list.delete"), icon: Trash2, danger: true, run: () => store.deleteList(list) });
    }
    menu.openAt(optionsBtn, entries, "right");
  }

  function finishRename() {
    if (store.currentList) store.renameList(store.currentList.id, renameValue);
  }

  const subtitle = $derived.by(() => {
    if (store.searching) return t(store.tasks.length === 1 ? "task.countOne" : "task.count", { n: store.tasks.length });
    const count =
      openCount === 0 && store.tasks.length > 0
        ? t("task.allDoneCount")
        : t(openCount === 1 ? "task.countOne" : "task.count", { n: openCount });
    return store.view === "myDay" ? `${longDate()} · ${count}` : count;
  });
</script>

<section class="pane">
  <div class="toolbar" class:mac={macOverlay && !store.sidebarOpen} data-tauri-drag-region>
    {#if !store.sidebarOpen}
      <button class="tool-btn" title={t("sidebar.toggle")} onclick={() => (store.sidebarOpen = true)}>
        <PanelLeft size={17} strokeWidth={1.8} />
      </button>
    {/if}
    <div class="spacer" data-tauri-drag-region></div>
    {#if !store.searching}
      <button
        class="tool-btn"
        bind:this={customizeBtn}
        title={t("list.theme")}
        onclick={(e) => (customizeAnchor = e.currentTarget)}
      >
        <Palette size={17} strokeWidth={1.8} />
      </button>
      <button class="tool-btn" bind:this={optionsBtn} title={t("list.options")} onclick={openOptions}>
        <Ellipsis size={18} strokeWidth={1.8} />
      </button>
    {/if}
  </div>

  <div class="scroll">
    <div class="content">
      <header>
        {#if renaming}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="title-input"
            bind:value={renameValue}
            autofocus
            onfocus={(e) => e.currentTarget.select()}
            onblur={finishRename}
            onkeydown={(e) => {
              if (e.key === "Enter") e.currentTarget.blur();
              if (e.key === "Escape") store.renamingListId = null;
            }}
          />
        {:else}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <h1
            title={store.viewTitle}
            ondblclick={() => isCustomList && store.currentList && (store.renamingListId = store.currentList.id)}
          >
            {store.viewTitle}
          </h1>
        {/if}
        {#key subtitle}
          <div class="subtitle" in:fade={{ duration: 150 }}>{subtitle}</div>
        {/key}
      </header>

      {#if !store.searching}
        <AddTask />
      {/if}

      {#if store.ready && store.tasks.length === 0}
        <EmptyState />
      {:else if groups}
        {#each groups as group (group.key)}
          <div class="group-title">{t(group.key)} <span>{group.tasks.length}</span></div>
          {#each group.tasks as task (task.id)}
            <div animate:flip={{ duration: FLIP_MS }}>
              <TaskItem {task} {showList} />
            </div>
          {/each}
        {/each}
      {:else}
        <div
          class="zone"
          use:dndzone={{
            items,
            flipDurationMs: FLIP_MS,
            dragDisabled: !store.canReorder,
            dropTargetStyle: {},
            type: "tasks",
          }}
          onconsider={consider}
          onfinalize={finalize}
        >
          {#each items as task (task.id)}
            <div animate:flip={{ duration: FLIP_MS }}>
              <TaskItem {task} {showList} />
            </div>
          {/each}
        </div>
      {/if}

      {#if store.completed.length && store.showCompleted}
        <div class="completed-head">
          <button class="completed-toggle" onclick={() => (completedOpen = !completedOpen)}>
            <span class="chev" class:open={completedOpen}><ChevronRight size={14} strokeWidth={2.6} /></span>
            {t("task.completed")}
            <span class="count">{store.completed.length}</span>
          </button>
          <button class="link" onclick={() => (completedOpen = !completedOpen)}>
            {completedOpen ? t("list.hide") : t("list.show")}
          </button>
        </div>
        {#if completedOpen}
          <div transition:slide={{ duration: 260, easing: (x) => 1 - Math.pow(1 - x, 3) }}>
            {#each store.completed as task (task.id)}
              <div animate:flip={{ duration: FLIP_MS }} in:fade={{ duration: 200 }}>
                <TaskItem {task} {showList} />
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  </div>
</section>

{#if customizeAnchor}
  <ListCustomizer anchor={customizeAnchor} onclose={() => (customizeAnchor = null)} />
{/if}

<style>
  .pane {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 48px;
    padding: 10px 14px 0;
    flex: none;
  }
  .toolbar.mac {
    padding-left: 84px;
  }
  .spacer {
    flex: 1;
    align-self: stretch;
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0 28px 40px;
  }
  .content {
    max-width: 760px;
    margin: 0 auto;
  }
  header {
    padding: 6px 10px 14px;
  }
  h1 {
    margin: 0;
    font-size: 30px;
    font-weight: 700;
    letter-spacing: -0.025em;
    line-height: 1.2;
    color: var(--accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: color 0.3s;
  }
  .title-input {
    width: 100%;
    font-size: 30px;
    font-weight: 700;
    letter-spacing: -0.025em;
    line-height: 1.2;
    color: var(--accent);
    background: transparent;
    border: none;
    outline: none;
    padding: 0;
    border-radius: 6px;
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent) 22%, transparent);
  }
  .subtitle {
    margin-top: 3px;
    font-size: 14px;
    color: var(--text-2);
  }
  .zone {
    min-height: 4px;
    outline: none;
  }
  .group-title {
    margin: 20px 10px 4px;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text);
  }
  .group-title span {
    margin-left: 4px;
    font-weight: 500;
    color: var(--text-2);
  }
  .completed-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 22px 10px 4px;
  }
  .completed-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
    color: var(--text);
  }
  .chev {
    display: grid;
    color: var(--text-2);
    transition: transform 0.25s var(--ease);
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .count {
    color: var(--text-2);
    font-weight: 500;
  }
  .link {
    font-size: 13.5px;
    font-weight: 500;
    color: var(--accent);
  }
  .link:hover {
    opacity: 0.75;
  }
  @media (max-width: 640px) {
    .scroll {
      padding: 0 12px 24px;
    }
    h1,
    .title-input {
      font-size: 26px;
    }
  }
</style>
