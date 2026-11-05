<script lang="ts">
  import { Bell, CalendarDays, Star, Sun } from "@lucide/svelte";
  import { dueState, todayKey } from "../lib/dates";
  import { relativeDay, reminderLabel } from "../lib/format";
  import { t } from "../lib/i18n/index.svelte";
  import { menu } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";
  import { taskMenu } from "../lib/taskMenu";
  import { getTheme } from "../lib/themes";
  import type { Task } from "../lib/types";
  import Checkbox from "./Checkbox.svelte";

  interface Props {
    task: Task;
    showList: boolean;
  }
  let { task, showList }: Props = $props();

  const selected = $derived(store.selectedId === task.id);
  const list = $derived(store.lists.find((l) => l.id === task.listId));
  const listName = $derived(list ? (list.isDefault ? t("view.tasks") : list.name) : "");
  const doneSteps = $derived(task.steps.filter((s) => s.completed).length);
  const due = $derived(task.dueDate ? dueState(task.dueDate, store.today) : null);
  const inMyDay = $derived(task.myDayDate === todayKey());
  const notePreview = $derived(task.notes.trim().split("\n")[0]);
  const hasMeta = $derived(
    (showList && !!listName) || (inMyDay && store.view !== "myDay") || task.steps.length > 0 || !!task.dueDate || !!task.reminderAt,
  );

  let completing = $state(false);
  function toggle() {
    if (!task.completed) {
      completing = true;
      setTimeout(() => (completing = false), 700);
    }
    store.toggleComplete(task);
  }
</script>

<div
  class="task"
  class:selected
  class:completed={task.completed}
  class:completing
  role="button"
  tabindex="0"
  data-task-id={task.id}
  aria-label={task.title}
  aria-pressed={selected}
  onclick={() => store.select(selected ? null : task.id)}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      store.select(task.id);
    }
  }}
  oncontextmenu={(e) => {
    store.select(task.id);
    menu.open(e, taskMenu(task));
  }}
>
  <div class="check-col">
    <Checkbox
      checked={task.completed}
      label={task.completed ? t("task.markIncomplete") : t("task.markComplete")}
      onchange={toggle}
    />
  </div>
  <div class="body">
    <div class="title">{task.title}</div>
    {#if notePreview}
      <div class="note">{notePreview}</div>
    {/if}
    {#if hasMeta}
      <div class="meta">
        {#if task.dueDate}
          <span class="chip due {task.completed ? '' : due}">
            <CalendarDays size={12} strokeWidth={2.2} />
            {relativeDay(task.dueDate)}
          </span>
        {/if}
        {#if task.reminderAt}
          <span class="chip"><Bell size={12} strokeWidth={2.2} /> {reminderLabel(task.reminderAt)}</span>
        {/if}
        {#if task.steps.length}
          <span class="chip">{t("task.steps", { done: doneSteps, total: task.steps.length })}</span>
        {/if}
        {#if inMyDay && store.view !== "myDay"}
          <span class="chip"><Sun size={12} strokeWidth={2.2} /> {t("view.myDay")}</span>
        {/if}
        {#if showList && listName}
          <span class="chip"><i class="list-dot" style="background:{getTheme(list?.theme).accent}"></i>{listName}</span>
        {/if}
      </div>
    {/if}
  </div>
  <button
    class="star"
    class:on={task.important}
    aria-pressed={task.important}
    aria-label={task.important ? t("task.removeImportant") : t("task.markImportant")}
    title={task.important ? t("task.removeImportant") : t("task.markImportant")}
    onclick={(e) => {
      e.stopPropagation();
      store.toggleImportant(task);
    }}
  >
    <Star size={17} strokeWidth={1.9} fill={task.important ? "currentColor" : "none"} />
  </button>
</div>

<style>
  .task {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 8px 10px 10px;
    border-radius: 10px;
    outline: none;
    transition:
      background 0.15s,
      opacity 0.3s;
  }
  .task::after {
    content: "";
    position: absolute;
    left: 44px;
    right: 8px;
    bottom: 0;
    height: 1px;
    background: var(--separator);
    transition: opacity 0.15s;
  }
  .task:hover {
    background: var(--hover);
  }
  .task.selected {
    background: color-mix(in srgb, var(--accent) 11%, transparent);
  }
  .task:hover::after,
  .task.selected::after {
    opacity: 0;
  }
  .task:focus-visible {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 40%, transparent);
  }
  .check-col {
    padding-top: 0;
  }
  .body {
    flex: 1;
    min-width: 0;
    padding-top: 1px;
  }
  .title {
    font-size: 15px;
    line-height: 20px;
    color: var(--text);
    overflow-wrap: anywhere;
    transition: color 0.25s;
  }
  .completed .title {
    color: var(--text-2);
  }
  .completing .title {
    color: var(--text-2);
  }
  .note {
    margin-top: 1px;
    font-size: 13px;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 12px;
    margin-top: 3px;
    font-size: 12.5px;
    color: var(--text-2);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }
  .due.today {
    color: var(--accent);
    font-weight: 500;
  }
  .due.overdue {
    color: var(--danger);
    font-weight: 500;
  }
  .list-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    margin-right: 1px;
  }
  .star {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    margin-top: -3px;
    border-radius: 50%;
    flex: none;
    color: var(--text-3);
    opacity: 0;
    transition:
      opacity 0.15s,
      color 0.15s,
      transform 0.25s var(--spring);
  }
  .task:hover .star,
  .task.selected .star,
  .star:focus-visible {
    opacity: 1;
  }
  .star:hover {
    color: var(--accent);
  }
  .star:active {
    transform: scale(0.8);
  }
  .star.on {
    opacity: 1;
    color: var(--accent);
    animation: pop 0.4s var(--spring);
  }
  @keyframes pop {
    50% {
      transform: scale(1.25);
    }
  }
  @media (hover: none) {
    .star {
      opacity: 1;
    }
  }
</style>
