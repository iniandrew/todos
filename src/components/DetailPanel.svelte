<script lang="ts">
  import { ArrowUpFromLine, Bell, CalendarDays, ChevronsUpDown, Plus, Star, Sun, Trash2, X } from "@lucide/svelte";
  import { untrack } from "svelte";
  import { fly, slide } from "svelte/transition";
  import { addDays, dueState, isoToLocalInput, laterToday, localInputToIso, nextWeekKey, reminderAt, todayKey } from "../lib/dates";
  import { dateFromIso, relativeDay, time } from "../lib/format";
  import { t } from "../lib/i18n/index.svelte";
  import { listIcon } from "../lib/listIcons";
  import { store } from "../lib/store.svelte";
  import { getTheme } from "../lib/themes";
  import type { Task } from "../lib/types";
  import Checkbox from "./Checkbox.svelte";

  let { task, macOverlay = false }: { task: Task; macOverlay?: boolean } = $props();

  // The panel is re-mounted per task (keyed in App), so the initial values are what we want.
  let title = $state(untrack(() => task.title));
  let notes = $state(untrack(() => task.notes));
  let newStep = $state("");
  let editingStep = $state<string | null>(null);
  let stepValue = $state("");
  let titleEl: HTMLTextAreaElement | undefined = $state();
  let notesEl: HTMLTextAreaElement | undefined = $state();
  let notesTimer: ReturnType<typeof setTimeout> | undefined;

  // Keep in sync with outside changes (context menu, shortcuts) while not editing.
  $effect(() => {
    const incoming = task.title;
    if (document.activeElement !== titleEl) title = incoming;
  });

  function autosize(el: HTMLTextAreaElement | undefined) {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
  }
  $effect(() => {
    void title;
    void notes;
    requestAnimationFrame(() => {
      autosize(titleEl);
      autosize(notesEl);
    });
  });

  const inMyDay = $derived(task.myDayDate === store.today);
  const due = $derived(task.dueDate ? dueState(task.dueDate, store.today) : null);
  const list = $derived(store.lists.find((l) => l.id === task.listId));
  const ListIcon = $derived(listIcon(list?.icon));

  function saveTitle() {
    const v = title.trim();
    if (v && v !== task.title) store.patchTask(task.id, { title: v });
    else title = task.title;
  }

  function onNotes() {
    clearTimeout(notesTimer);
    const id = task.id;
    const value = notes;
    if (value === task.notes) return;
    notesTimer = setTimeout(() => store.patchTask(id, { notes: value }), 400);
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

  function setDue(date: string | null) {
    store.patchTask(task.id, { dueDate: date });
  }

  function setReminder(iso: string | null) {
    store.patchTask(task.id, { reminderAt: iso });
    if (iso && "Notification" in window && Notification.permission === "default") {
      Notification.requestPermission().catch(() => {});
    }
  }

  const createdLabel = $derived.by(() => {
    if (task.completed && task.completedAt) {
      const d = dateFromIso(task.completedAt);
      return d === t("date.today") ? t("detail.completedToday") : t("detail.completedOn", { date: d });
    }
    const d = dateFromIso(task.createdAt);
    return d === t("date.today") ? t("detail.createdToday") : t("detail.createdOn", { date: d });
  });

  const tomorrow = $derived(addDays(store.today, 1));
  const nextWeek = $derived(nextWeekKey(store.today));
</script>

<aside class="panel" transition:fly={{ x: 24, duration: 260, opacity: 0 }}>
  <div class="head" class:mac={macOverlay} data-tauri-drag-region>
    <button class="close" title={t("detail.close")} aria-label={t("detail.close")} onclick={() => store.select(null)}>
      <X size={13} strokeWidth={2.8} />
    </button>
  </div>

  <div class="scroll">
    <!-- Title + notes -->
    <section class="group">
      <div class="title-row">
        <Checkbox
          checked={task.completed}
          size={24}
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
      </div>
      <textarea
        bind:this={notesEl}
        bind:value={notes}
        rows="2"
        class="notes"
        placeholder={t("detail.notes")}
        aria-label={t("task.notes")}
        oninput={onNotes}
        onblur={onNotes}
      ></textarea>
    </section>

    <!-- Steps -->
    <div class="group-label">{t("detail.steps")}</div>
    <section class="group">
      {#each task.steps as step (step.id)}
        <div class="row step" transition:slide={{ duration: 180 }}>
          <Checkbox
            checked={step.completed}
            size={19}
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
        </div>
      {/each}
      <form
        class="row add-step"
        onsubmit={(e) => {
          e.preventDefault();
          addStep();
        }}
      >
        <span class="add-dot"><Plus size={12} strokeWidth={3} /></span>
        <input bind:value={newStep} placeholder={t("detail.addStep")} aria-label={t("detail.addStep")} />
      </form>
    </section>

    <!-- Flags -->
    <section class="group">
      <label class="row">
        <span class="glyph" style="background:{getTheme('orange').accent}"><Sun size={16} strokeWidth={2.4} /></span>
        <span class="row-label">{t("view.myDay")}</span>
        <span class="switch"><input type="checkbox" checked={inMyDay} onchange={() => store.toggleMyDay(task)} /><span></span></span>
      </label>
      <label class="row">
        <span class="glyph" style="background:{getTheme('pink').accent}"><Star size={15} strokeWidth={2.4} fill="currentColor" /></span>
        <span class="row-label">{t("detail.important")}</span>
        <span class="switch"><input type="checkbox" checked={task.important} onchange={() => store.toggleImportant(task)} /><span></span></span>
      </label>
    </section>

    <!-- Dates -->
    <section class="group">
      <label class="row">
        <span class="glyph" style="background:{getTheme('red').accent}"><CalendarDays size={16} strokeWidth={2.4} /></span>
        <span class="row-label">
          {t("detail.dueDate")}
          {#if task.dueDate}<small class={task.completed ? "" : due}>{relativeDay(task.dueDate)}</small>{/if}
        </span>
        <span class="switch">
          <input type="checkbox" checked={!!task.dueDate} onchange={(e) => setDue(e.currentTarget.checked ? todayKey() : null)} /><span></span>
        </span>
      </label>
      {#if task.dueDate}
        <div class="sub" transition:slide={{ duration: 200 }}>
          <div class="chips">
            <button class="chip" class:on={task.dueDate === store.today} onclick={() => setDue(store.today)}>{t("date.today")}</button>
            <button class="chip" class:on={task.dueDate === tomorrow} onclick={() => setDue(tomorrow)}>{t("date.tomorrow")}</button>
            <button class="chip" class:on={task.dueDate === nextWeek} onclick={() => setDue(nextWeek)}>{t("detail.nextWeek")}</button>
          </div>
          <input type="date" value={task.dueDate} onchange={(e) => e.currentTarget.value && setDue(e.currentTarget.value)} />
        </div>
      {/if}

      <label class="row">
        <span class="glyph" style="background:{getTheme('indigo').accent}"><Bell size={15} strokeWidth={2.4} /></span>
        <span class="row-label">
          {t("detail.reminder")}
          {#if task.reminderAt}<small>{dateFromIso(task.reminderAt)}, {time(task.reminderAt)}</small>{/if}
        </span>
        <span class="switch">
          <input type="checkbox" checked={!!task.reminderAt} onchange={(e) => setReminder(e.currentTarget.checked ? laterToday() : null)} /><span></span>
        </span>
      </label>
      {#if task.reminderAt}
        <div class="sub" transition:slide={{ duration: 200 }}>
          <div class="chips">
            <button class="chip" onclick={() => setReminder(laterToday())}>{t("detail.laterToday")}</button>
            <button class="chip" class:on={task.reminderAt === reminderAt(tomorrow, 9)} onclick={() => setReminder(reminderAt(tomorrow, 9))}
              >{t("detail.tomorrow")} 9:00</button
            >
            <button class="chip" class:on={task.reminderAt === reminderAt(nextWeek, 9)} onclick={() => setReminder(reminderAt(nextWeek, 9))}
              >{t("detail.nextWeek")}</button
            >
          </div>
          <input
            type="datetime-local"
            value={isoToLocalInput(task.reminderAt)}
            onchange={(e) => e.currentTarget.value && setReminder(localInputToIso(e.currentTarget.value))}
          />
        </div>
      {/if}
    </section>

    <!-- List -->
    <section class="group">
      <label class="row">
        <span class="glyph" style="background:{getTheme(list?.theme).accent}"><ListIcon size={15} strokeWidth={2.4} /></span>
        <span class="row-label">{t("detail.list")}</span>
        <span class="select">
          <select value={task.listId} onchange={(e) => store.patchTask(task.id, { listId: e.currentTarget.value })}>
            {#each store.lists as l (l.id)}
              <option value={l.id}>{l.isDefault ? t("view.tasks") : l.name}</option>
            {/each}
          </select>
          <ChevronsUpDown size={13} strokeWidth={2.2} />
        </span>
      </label>
    </section>

    <section class="group">
      <button class="row danger" onclick={() => store.deleteTask(task)}>
        <Trash2 size={16} strokeWidth={2} />
        {t("task.delete")}
      </button>
    </section>

    <div class="foot">{createdLabel}</div>
  </div>
</aside>

<style>
  .panel {
    width: 350px;
    flex: none;
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--grouped-bg);
    border-left: 1px solid var(--separator);
  }
  .head {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    height: 48px;
    padding: 10px 14px 0;
    flex: none;
  }
  .close {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--fill);
    color: var(--text-2);
    transition: background 0.15s;
  }
  .close:hover {
    background: var(--text-3);
    color: var(--bg);
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 4px 16px 24px;
  }
  .group {
    background: var(--cell);
    border-radius: 12px;
    margin-bottom: 18px;
    overflow: hidden;
  }
  .group-label {
    margin: 0 14px 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
  }
  .title-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 14px 14px 0;
  }
  .title-row :global(.check) {
    margin-top: 2px;
  }
  .title {
    flex: 1;
    resize: none;
    border: none;
    outline: none;
    background: transparent;
    font-size: 19px;
    font-weight: 650;
    letter-spacing: -0.015em;
    line-height: 1.35;
    padding: 0;
    overflow: hidden;
  }
  .title.done {
    color: var(--text-2);
  }
  .notes {
    display: block;
    width: 100%;
    resize: none;
    border: none;
    outline: none;
    background: transparent;
    padding: 6px 14px 14px 50px;
    font-size: 14px;
    line-height: 1.5;
    color: var(--text-2);
    overflow: hidden;
    min-height: 44px;
  }
  .notes:focus {
    color: var(--text);
  }
  .notes::placeholder {
    color: var(--text-3);
  }
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    min-height: 46px;
    padding: 8px 14px;
    text-align: left;
  }
  .row + .row::before,
  .sub + .row::before,
  .row + .sub::before {
    content: "";
    position: absolute;
    top: 0;
    left: 54px;
    right: 0;
    height: 1px;
    background: var(--separator);
  }
  .step + .step::before,
  .step + .add-step::before {
    left: 45px;
  }
  .row-label {
    flex: 1;
    display: flex;
    flex-direction: column;
    font-size: 14.5px;
  }
  .row-label small {
    font-size: 12.5px;
    color: var(--accent);
    font-weight: 500;
  }
  .row-label small.overdue {
    color: var(--danger);
  }
  .step {
    min-height: 42px;
  }
  .step-title {
    flex: 1;
    text-align: left;
    padding: 2px 0;
    font-size: 14.5px;
    overflow-wrap: anywhere;
    color: var(--text);
  }
  .step-title.done {
    color: var(--text-2);
    text-decoration: line-through;
  }
  .step-input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font-size: 14.5px;
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
    width: 24px;
    height: 24px;
    border-radius: 6px;
    color: var(--text-2);
  }
  .mini:hover {
    background: var(--fill);
    color: var(--text);
  }
  .add-step {
    min-height: 42px;
  }
  .add-dot {
    display: grid;
    place-items: center;
    width: 19px;
    height: 19px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
    flex: none;
  }
  .add-step input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font-size: 14.5px;
  }
  .add-step input::placeholder {
    color: var(--text-2);
  }
  .sub {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px 14px 12px 54px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    height: 26px;
    padding: 0 10px;
    border-radius: 13px;
    font-size: 12.5px;
    font-weight: 500;
    background: var(--fill-2);
    color: var(--text);
    transition: background 0.15s;
  }
  .chip:hover {
    background: var(--fill);
  }
  .chip.on {
    background: var(--accent);
    color: #fff;
  }
  .sub input {
    height: 32px;
    padding: 0 10px;
    border-radius: 8px;
    border: none;
    background: var(--fill-2);
    font-size: 13.5px;
    outline: none;
  }
  .sub input:focus {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 30%, transparent);
  }
  .select {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--text-2);
  }
  .select select {
    appearance: none;
    -webkit-appearance: none;
    border: none;
    background: transparent;
    font-size: 14px;
    color: var(--text-2);
    text-align: right;
    padding-right: 2px;
    max-width: 150px;
    outline: none;
  }
  .danger {
    justify-content: center;
    gap: 8px;
    color: var(--danger);
    font-weight: 500;
    font-size: 14.5px;
  }
  .danger:hover {
    background: color-mix(in srgb, var(--danger) 8%, transparent);
  }
  .foot {
    text-align: center;
    font-size: 12px;
    color: var(--text-3);
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
