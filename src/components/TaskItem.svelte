<script lang="ts">
  import { Bell, Calendar, NotebookPen, Sun } from "@lucide/svelte";
  import { dueState, todayKey } from "../lib/dates";
  import { relativeDay, reminderLabel } from "../lib/format";
  import { t } from "../lib/i18n/index.svelte";
  import { menu } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";
  import { taskMenu } from "../lib/taskMenu";
  import type { Task } from "../lib/types";
  import Checkbox from "./Checkbox.svelte";
  import Star from "./Star.svelte";

  interface Props {
    task: Task;
    showList: boolean;
  }
  let { task, showList }: Props = $props();

  const selected = $derived(store.selectedId === task.id);
  const listName = $derived.by(() => {
    const l = store.lists.find((l) => l.id === task.listId);
    return l ? (l.isDefault ? t("view.tasks") : l.name) : "";
  });
  const doneSteps = $derived(task.steps.filter((s) => s.completed).length);
  const due = $derived(task.dueDate ? dueState(task.dueDate, store.today) : null);
  const inMyDay = $derived(task.myDayDate === todayKey());

  let justCompleted = $state(false);
  function toggle() {
    if (!task.completed) {
      justCompleted = true;
      setTimeout(() => (justCompleted = false), 600);
    }
    store.toggleComplete(task);
  }
</script>

<div
  class="task"
  class:selected
  class:completed={task.completed}
  class:flash={justCompleted}
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
  <Checkbox
    checked={task.completed}
    label={task.completed ? t("task.markIncomplete") : t("task.markComplete")}
    onchange={toggle}
  />
  <div class="body">
    <div class="title">{task.title}</div>
    {#if (showList && listName) || (inMyDay && store.view !== "myDay") || task.steps.length || task.dueDate || task.reminderAt || task.notes.trim()}
      <div class="meta">
        {#if inMyDay && store.view !== "myDay"}
          <span class="chip"><Sun size={12} /> {t("view.myDay")}</span>
        {/if}
        {#if showList && listName}
          <span class="chip">{listName}</span>
        {/if}
        {#if task.steps.length}
          <span class="chip">{t("task.steps", { done: doneSteps, total: task.steps.length })}</span>
        {/if}
        {#if task.dueDate}
          <span class="chip due {task.completed ? '' : due}">
            <Calendar size={12} />
            {due === "overdue" && !task.completed ? `${t("date.overdue")}, ` : ""}{relativeDay(task.dueDate)}
          </span>
        {/if}
        {#if task.reminderAt}
          <span class="chip"><Bell size={12} /> {reminderLabel(task.reminderAt)}</span>
        {/if}
        {#if task.notes.trim()}
          <span class="chip" title={t("task.notes")}><NotebookPen size={12} /></span>
        {/if}
      </div>
    {/if}
  </div>
  <Star
    active={task.important}
    label={task.important ? t("task.removeImportant") : t("task.markImportant")}
    onchange={() => store.toggleImportant(task)}
  />
</div>

<style>
  .task {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 52px;
    padding: 8px 8px 8px 16px;
    margin-bottom: 2px;
    border-radius: 8px;
    background: var(--card);
    box-shadow: var(--shadow-sm);
    cursor: default;
    outline: none;
    transition:
      background 0.15s,
      box-shadow 0.15s,
      transform 0.15s var(--ease);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }
  .task:hover {
    background: var(--card-hover);
  }
  .task:focus-visible {
    box-shadow:
      0 0 0 2px var(--accent),
      var(--shadow-sm);
  }
  .task.selected {
    background: var(--card-hover);
    box-shadow:
      inset 3px 0 0 var(--accent),
      var(--shadow-md);
  }
  .body {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 14.5px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: color 0.2s;
  }
  .completed .title {
    color: var(--text-3);
    text-decoration: line-through;
    text-decoration-color: var(--text-3);
  }
  .flash {
    animation: flash 0.6s var(--ease);
  }
  @keyframes flash {
    30% {
      transform: scale(1.01);
      background: color-mix(in srgb, var(--accent) 12%, var(--card-hover));
    }
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    margin-top: 2px;
    font-size: 12px;
    color: var(--text-2);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .chip + .chip::before {
    content: "•";
    margin: 0 7px;
    color: var(--text-3);
  }
  .due.today {
    color: var(--accent);
  }
  .due.overdue {
    color: var(--danger);
  }
</style>
