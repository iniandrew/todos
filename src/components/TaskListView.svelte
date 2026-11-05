<script lang="ts">
  import { ChevronRight, Ellipsis, Eye, EyeOff, Palette, PanelLeft, Pencil, Trash2 } from "@lucide/svelte";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { fly, slide } from "svelte/transition";
  import { diffDays, isoToDateKey } from "../lib/dates";
  import { longDate } from "../lib/format";
  import { t, type TKey } from "../lib/i18n/index.svelte";
  import { menu, type MenuEntry } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";
  import type { Task } from "../lib/types";
  import AddTask from "./AddTask.svelte";
  import EmptyState from "./EmptyState.svelte";
  import TaskItem from "./TaskItem.svelte";
  import ThemePicker from "./ThemePicker.svelte";

  const FLIP_MS = 180;

  let items = $state<Task[]>([]);
  $effect(() => {
    items = store.incomplete;
  });

  let completedOpen = $state(true);
  let themeAnchor = $state<HTMLElement | null>(null);
  let optionsBtn: HTMLButtonElement | undefined = $state();
  let renameValue = $state("");

  const showList = $derived(store.searching || !(store.view === "tasks" || store.view.startsWith("list:")));
  const isCustomList = $derived(store.view.startsWith("list:") && !store.searching);
  const renaming = $derived(isCustomList && store.renamingListId === store.currentList?.id);

  $effect(() => {
    if (renaming && store.currentList) renameValue = store.currentList.name;
  });

  // Planned view: bucket tasks by date.
  type Group = { key: TKey; tasks: Task[] };
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
      { label: t("list.theme"), icon: Palette, run: () => (themeAnchor = optionsBtn!) },
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
</script>

<section class="pane" style="--accent:{store.theme.accent}; --pane-bg:{store.theme.background}">
  <div class="bg"></div>
  <header>
    {#if !store.sidebarOpen}
      <button class="icon-btn hdr-btn" title={t("sidebar.toggle")} onclick={() => (store.sidebarOpen = true)}>
        <PanelLeft size={18} />
      </button>
    {/if}
    <div class="titles">
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
      {#if store.view === "myDay" && !store.searching}
        <div class="subtitle">{longDate()}</div>
      {/if}
    </div>
    <div class="actions">
      {#if !store.searching}
        <button
          class="icon-btn hdr-btn"
          title={t("list.theme")}
          onclick={(e) => (themeAnchor = e.currentTarget)}
        >
          <Palette size={18} />
        </button>
        <button class="icon-btn hdr-btn" bind:this={optionsBtn} title={t("list.options")} onclick={openOptions}>
          <Ellipsis size={18} />
        </button>
      {/if}
    </div>
  </header>

  <div class="scroll">
    <div class="content">
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
        {#if store.ready && store.tasks.length > 0 && items.length === 0 && !store.searching}
          <div class="all-done" in:fly={{ y: 8, duration: 250 }}>{t("empty.allDone")}</div>
        {/if}
      {/if}

      {#if store.completed.length && store.showCompleted}
        <button class="completed-toggle" onclick={() => (completedOpen = !completedOpen)}>
          <span class="chev" class:open={completedOpen}><ChevronRight size={16} /></span>
          {t("task.completed")}
          <span class="count">{store.completed.length}</span>
        </button>
        {#if completedOpen}
          <div transition:slide={{ duration: 200 }}>
            {#each store.completed as task (task.id)}
              <div animate:flip={{ duration: FLIP_MS }} in:fly={{ y: -10, duration: 220 }}>
                <TaskItem {task} {showList} />
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  </div>
</section>

{#if themeAnchor}
  <ThemePicker anchor={themeAnchor} onclose={() => (themeAnchor = null)} />
{/if}

<style>
  .pane {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    isolation: isolate;
  }
  .bg {
    position: absolute;
    inset: 0;
    z-index: -1;
    background: var(--pane-bg);
    transition: background 0.4s;
  }
  :global([data-theme="dark"]) .bg::after {
    content: "";
    position: absolute;
    inset: 0;
    background: rgba(10, 12, 18, 0.32);
  }
  header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 26px 32px 14px;
    color: #fff;
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  h1 {
    margin: 0;
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.01em;
    line-height: 1.25;
    text-shadow: 0 1px 12px rgba(0, 0, 0, 0.18);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .title-input {
    width: 100%;
    font-size: 26px;
    font-weight: 700;
    color: #fff;
    background: rgba(255, 255, 255, 0.16);
    border: 1px solid rgba(255, 255, 255, 0.5);
    border-radius: 8px;
    padding: 0 8px;
    margin-left: -9px;
    outline: none;
  }
  .subtitle {
    margin-top: 2px;
    font-size: 14px;
    opacity: 0.92;
    text-shadow: 0 1px 8px rgba(0, 0, 0, 0.18);
  }
  .actions {
    display: flex;
    gap: 4px;
  }
  .hdr-btn {
    color: #fff;
    background: rgba(255, 255, 255, 0.14);
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
  }
  .hdr-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.28);
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0 32px 32px;
  }
  .content {
    max-width: 980px;
    margin: 0 auto;
  }
  .zone {
    min-height: 4px;
    outline: none;
  }
  .group-title {
    margin: 14px 0 6px;
    padding: 0 4px;
    font-size: 13px;
    font-weight: 600;
    color: #fff;
    text-shadow: 0 1px 8px rgba(0, 0, 0, 0.25);
  }
  .group-title span {
    opacity: 0.75;
    margin-left: 4px;
    font-weight: 500;
  }
  .completed-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin: 14px 0 8px;
    padding: 6px 12px 6px 8px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 13px;
    color: var(--text);
    background: var(--card);
    box-shadow: var(--shadow-sm);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }
  .completed-toggle:hover {
    background: var(--card-hover);
  }
  .chev {
    display: grid;
    transition: transform 0.2s var(--ease);
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .count {
    color: var(--text-2);
    font-weight: 500;
  }
  .all-done {
    margin: 18px 0 4px;
    text-align: center;
    color: #fff;
    font-weight: 600;
    text-shadow: 0 1px 8px rgba(0, 0, 0, 0.25);
  }
  @media (max-width: 640px) {
    header {
      padding: 18px 16px 12px;
    }
    .scroll {
      padding: 0 16px 24px;
    }
  }
</style>
