<script lang="ts">
  import {
    ArrowUpFromLine,
    Bell,
    Calendar,
    CalendarArrowUp,
    CalendarClock,
    CalendarDays,
    Clock,
    Plus,
    Sun,
    Trash2,
    X,
    PanelRightClose,
  } from "@lucide/svelte";
  import { fly } from "svelte/transition";
  import { addDays, dueState, isoToLocalInput, laterToday, localInputToIso, nextWeekKey, reminderAt, todayKey } from "../lib/dates";
  import { dateFromIso, relativeDay, time } from "../lib/format";
  import { t } from "../lib/i18n/index.svelte";
  import { menu } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";
  import type { Task } from "../lib/types";
  import Checkbox from "./Checkbox.svelte";
  import Star from "./Star.svelte";

  interface Props {
    task: Task;
  }
  let { task }: Props = $props();

  let title = $state("");
  let notes = $state("");
  let newStep = $state("");
  let editingStep = $state<string | null>(null);
  let stepValue = $state("");
  let picking = $state<"due" | "reminder" | null>(null);
  let pickValue = $state("");
  let remindBtn: HTMLButtonElement | undefined = $state();
  let dueBtn: HTMLButtonElement | undefined = $state();
  let titleEl: HTMLTextAreaElement | undefined = $state();
  let notesTimer: ReturnType<typeof setTimeout> | undefined;

  // Reset local edit buffers whenever another task is selected.
  let lastId = "";
  $effect(() => {
    if (task.id !== lastId) {
      lastId = task.id;
      title = task.title;
      notes = task.notes;
      newStep = "";
      picking = null;
      editingStep = null;
    }
  });
  // Keep in sync with outside changes (context menu etc.) when not editing.
  $effect(() => {
    if (document.activeElement !== titleEl) title = task.title;
  });

  function autosize() {
    if (!titleEl) return;
    titleEl.style.height = "auto";
    titleEl.style.height = `${titleEl.scrollHeight}px`;
  }
  $effect(() => {
    void title;
    requestAnimationFrame(autosize);
  });

  const inMyDay = $derived(task.myDayDate === store.today);
  const due = $derived(task.dueDate ? dueState(task.dueDate, store.today) : null);

  function saveTitle() {
    const v = title.trim();
    if (v && v !== task.title) store.patchTask(task.id, { title: v });
    else title = task.title;
  }

  function onNotes() {
    clearTimeout(notesTimer);
    const id = task.id;
    const value = notes;
    notesTimer = setTimeout(() => store.patchTask(id, { notes: value }), 500);
  }

  function addStep() {
    const v = newStep.trim();
    if (!v) return;
    newStep = "";
    store.addStep(task.id, v);
  }

  function saveStep(id: string) {
    const v = stepValue.trim();
    editingStep = null;
    if (v) store.renameStep(id, v);
  }

  function remindMenu() {
    if (!remindBtn) return;
    const tomorrow = addDays(todayKey(), 1);
    const nextWeek = nextWeekKey();
    menu.openAt(remindBtn, [
      { label: `${t("detail.laterToday")} · ${time(laterToday())}`, icon: Clock, run: () => setReminder(laterToday()) },
      { label: `${t("detail.tomorrow")} · ${time(reminderAt(tomorrow, 9))}`, icon: CalendarArrowUp, run: () => setReminder(reminderAt(tomorrow, 9)) },
      { label: `${t("detail.nextWeek")} · ${relativeDay(nextWeek)}`, icon: CalendarClock, run: () => setReminder(reminderAt(nextWeek, 9)) },
      "divider",
      {
        label: t("detail.pickDateTime"),
        icon: CalendarDays,
        run: () => {
          pickValue = task.reminderAt ? isoToLocalInput(task.reminderAt) : isoToLocalInput(laterToday());
          picking = "reminder";
        },
      },
    ]);
  }

  function setReminder(iso: string | null) {
    picking = null;
    store.patchTask(task.id, { reminderAt: iso });
    if (iso && "Notification" in window && Notification.permission === "default") {
      Notification.requestPermission().catch(() => {});
    }
  }

  function dueMenu() {
    if (!dueBtn) return;
    menu.openAt(dueBtn, [
      { label: t("date.today"), icon: Calendar, run: () => setDue(todayKey()) },
      { label: t("date.tomorrow"), icon: CalendarArrowUp, run: () => setDue(addDays(todayKey(), 1)) },
      { label: `${t("detail.nextWeek")} · ${relativeDay(nextWeekKey())}`, icon: CalendarClock, run: () => setDue(nextWeekKey()) },
      "divider",
      {
        label: t("task.pickDate"),
        icon: CalendarDays,
        run: () => {
          pickValue = task.dueDate ?? todayKey();
          picking = "due";
        },
      },
    ]);
  }

  function setDue(date: string | null) {
    picking = null;
    store.patchTask(task.id, { dueDate: date });
  }

  function savePick() {
    if (!pickValue) return;
    if (picking === "due") setDue(pickValue);
    else if (picking === "reminder") setReminder(localInputToIso(pickValue));
  }
</script>

<aside class="panel" style="--accent:{store.theme.accent}" transition:fly={{ x: 40, duration: 220 }}>
  <div class="scroll">
    <section class="card head">
      <div class="title-row">
        <Checkbox
          checked={task.completed}
          size={22}
          label={task.completed ? t("task.markIncomplete") : t("task.markComplete")}
          onchange={() => store.toggleComplete(task)}
        />
        <textarea
          bind:this={titleEl}
          bind:value={title}
          rows="1"
          class="title"
          class:done={task.completed}
          aria-label={t("task.add")}
          oninput={autosize}
          onblur={saveTitle}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              e.currentTarget.blur();
            }
            if (e.key === "Escape") {
              title = task.title;
              e.currentTarget.blur();
            }
          }}
        ></textarea>
        <Star
          active={task.important}
          label={task.important ? t("task.removeImportant") : t("task.markImportant")}
          onchange={() => store.toggleImportant(task)}
        />
      </div>

      <ul class="steps">
        {#each task.steps as step (step.id)}
          <li class="step">
            <Checkbox
              checked={step.completed}
              size={17}
              label={step.completed ? t("task.markIncomplete") : t("task.markComplete")}
              onchange={() => store.toggleStep(step.id, !step.completed)}
            />
            {#if editingStep === step.id}
              <!-- svelte-ignore a11y_autofocus -->
              <input
                class="step-input"
                bind:value={stepValue}
                autofocus
                onblur={() => saveStep(step.id)}
                onkeydown={(e) => {
                  if (e.key === "Enter") e.currentTarget.blur();
                  if (e.key === "Escape") editingStep = null;
                }}
              />
            {:else}
              <button
                class="step-title"
                class:done={step.completed}
                onclick={() => {
                  editingStep = step.id;
                  stepValue = step.title;
                }}>{step.title}</button
              >
            {/if}
            <div class="step-actions">
              <button class="mini" title={t("detail.promote")} onclick={() => store.promoteStep(step.id)}>
                <ArrowUpFromLine size={14} />
              </button>
              <button class="mini" title={t("detail.deleteStep")} onclick={() => store.deleteStep(step.id)}>
                <X size={14} />
              </button>
            </div>
          </li>
        {/each}
      </ul>
      <form
        class="add-step"
        onsubmit={(e) => {
          e.preventDefault();
          addStep();
        }}
      >
        <Plus size={17} />
        <input
          bind:value={newStep}
          placeholder={task.steps.length ? t("detail.nextStep") : t("detail.addStep")}
          aria-label={t("detail.addStep")}
        />
      </form>
    </section>

    <section class="card">
      <button class="row-btn" class:on={inMyDay} onclick={() => store.toggleMyDay(task)}>
        <Sun size={18} />
        <span class="label">{inMyDay ? t("task.addedToMyDay") : t("task.addToMyDay")}</span>
        {#if inMyDay}
          <span
            class="clear"
            role="button"
            tabindex="0"
            title={t("task.removeFromMyDay")}
            onclick={(e) => {
              e.stopPropagation();
              store.toggleMyDay(task);
            }}
            onkeydown={(e) => e.key === "Enter" && store.toggleMyDay(task)}><X size={14} /></span
          >
        {/if}
      </button>
    </section>

    <section class="card">
      <button class="row-btn" class:on={task.reminderAt} bind:this={remindBtn} onclick={remindMenu}>
        <Bell size={18} />
        <span class="label">
          {#if task.reminderAt}
            {t("detail.remindAt", { time: time(task.reminderAt) })}
            <small>{dateFromIso(task.reminderAt)}</small>
          {:else}
            {t("detail.remindMe")}
          {/if}
        </span>
        {#if task.reminderAt}
          <span
            class="clear"
            role="button"
            tabindex="0"
            title={t("detail.clear")}
            onclick={(e) => {
              e.stopPropagation();
              setReminder(null);
            }}
            onkeydown={(e) => e.key === "Enter" && setReminder(null)}><X size={14} /></span
          >
        {/if}
      </button>
      {#if picking === "reminder"}
        <div class="picker" transition:fly={{ y: -6, duration: 150 }}>
          <input type="datetime-local" bind:value={pickValue} />
          <button class="btn primary" onclick={savePick}>{t("detail.save")}</button>
        </div>
      {/if}
      <div class="sep"></div>
      <button class="row-btn {task.dueDate && !task.completed ? due : ''}" class:on={task.dueDate} bind:this={dueBtn} onclick={dueMenu}>
        <CalendarDays size={18} />
        <span class="label">
          {task.dueDate ? t("detail.due", { date: relativeDay(task.dueDate) }) : t("detail.addDueDate")}
        </span>
        {#if task.dueDate}
          <span
            class="clear"
            role="button"
            tabindex="0"
            title={t("task.removeDueDate")}
            onclick={(e) => {
              e.stopPropagation();
              setDue(null);
            }}
            onkeydown={(e) => e.key === "Enter" && setDue(null)}><X size={14} /></span
          >
        {/if}
      </button>
      {#if picking === "due"}
        <div class="picker" transition:fly={{ y: -6, duration: 150 }}>
          <input type="date" bind:value={pickValue} />
          <button class="btn primary" onclick={savePick}>{t("detail.save")}</button>
        </div>
      {/if}
    </section>

    <section class="card notes">
      <textarea bind:value={notes} oninput={onNotes} onblur={onNotes} placeholder={t("detail.addNote")} aria-label={t("task.notes")}
      ></textarea>
    </section>
  </div>

  <footer>
    <button class="icon-btn" title={t("detail.close")} onclick={() => store.select(null)}>
      <PanelRightClose size={18} />
    </button>
    <span class="meta">
      {#if task.completed && task.completedAt}
        {dateFromIso(task.completedAt) === t("date.today")
          ? t("detail.completedToday")
          : t("detail.completedOn", { date: dateFromIso(task.completedAt) })}
      {:else if dateFromIso(task.createdAt) === t("date.today")}
        {t("detail.createdToday")}
      {:else}
        {t("detail.createdOn", { date: dateFromIso(task.createdAt) })}
      {/if}
    </span>
    <button class="icon-btn danger" title={t("task.delete")} onclick={() => store.deleteTask(task)}>
      <Trash2 size={18} />
    </button>
  </footer>
</aside>

<style>
  .panel {
    width: 360px;
    flex: none;
    display: flex;
    flex-direction: column;
    background: var(--sidebar-bg);
    border-left: 1px solid var(--border);
    height: 100%;
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 16px 14px;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    margin-bottom: 10px;
    overflow: hidden;
  }
  .head {
    padding: 14px 6px 6px 14px;
  }
  .title-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .title-row :global(.check) {
    margin-top: 4px;
  }
  .title {
    flex: 1;
    resize: none;
    border: none;
    outline: none;
    background: transparent;
    font-size: 18px;
    font-weight: 600;
    line-height: 1.35;
    padding: 2px 0;
    overflow: hidden;
    border-radius: 4px;
  }
  .title:focus {
    box-shadow: 0 2px 0 var(--accent);
  }
  .title.done {
    text-decoration: line-through;
    color: var(--text-3);
  }
  .steps {
    list-style: none;
    margin: 8px 0 0;
    padding: 0;
  }
  .step {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 38px;
    padding-left: 2px;
    border-bottom: 1px solid var(--border);
    margin-right: 8px;
  }
  .step-title {
    flex: 1;
    text-align: left;
    padding: 8px 0;
    color: var(--text);
    word-break: break-word;
  }
  .step-title.done {
    text-decoration: line-through;
    color: var(--text-3);
  }
  .step-input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    padding: 8px 0;
    box-shadow: 0 1px 0 var(--accent);
  }
  .step-actions {
    display: flex;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .step:hover .step-actions,
  .step:focus-within .step-actions {
    opacity: 1;
  }
  .mini {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 6px;
    color: var(--text-2);
  }
  .mini:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .add-step {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 42px;
    padding-left: 2px;
    color: var(--accent);
  }
  .add-step input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
  }
  .add-step input::placeholder {
    color: var(--accent);
  }
  .row-btn {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    min-height: 50px;
    padding: 8px 10px 8px 16px;
    color: var(--text-2);
    text-align: left;
    transition: background 0.12s;
  }
  .row-btn:hover {
    background: var(--surface-hover);
  }
  .row-btn.on {
    color: var(--accent);
  }
  .row-btn.overdue {
    color: var(--danger);
  }
  .label {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .label small {
    color: var(--text-2);
    font-size: 12px;
  }
  .clear {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 6px;
    color: var(--text-2);
  }
  .clear:hover {
    background: var(--surface-2);
    color: var(--text);
  }
  .sep {
    height: 1px;
    background: var(--border);
    margin: 0 12px;
  }
  .picker {
    display: flex;
    gap: 8px;
    padding: 4px 12px 12px 48px;
  }
  .picker input {
    flex: 1;
    min-width: 0;
    height: 32px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
  }
  .btn {
    height: 32px;
    padding: 0 14px;
    border-radius: 6px;
    font-weight: 600;
  }
  .btn.primary {
    background: var(--accent);
    color: #fff;
  }
  .notes textarea {
    display: block;
    width: 100%;
    min-height: 120px;
    resize: vertical;
    border: none;
    outline: none;
    background: transparent;
    padding: 12px 16px;
    line-height: 1.5;
  }
  footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-top: 1px solid var(--border);
  }
  .meta {
    flex: 1;
    text-align: center;
    font-size: 12px;
    color: var(--text-2);
  }
  .danger:hover {
    color: var(--danger);
  }
  @media (max-width: 900px) {
    .panel {
      position: absolute;
      right: 0;
      top: 0;
      bottom: 0;
      z-index: 40;
      box-shadow: var(--shadow-lg);
    }
  }
  @media (max-width: 480px) {
    .panel {
      width: 100%;
    }
  }
</style>
