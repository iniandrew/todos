<script lang="ts">
  import { Calendar, CalendarArrowUp, CalendarClock, CalendarX, Plus, Star } from "@lucide/svelte";
  import { addDays, nextWeekKey, todayKey } from "../lib/dates";
  import { relativeDay } from "../lib/format";
  import { t } from "../lib/i18n/index.svelte";
  import { menu } from "../lib/menu.svelte";
  import { store } from "../lib/store.svelte";

  let title = $state("");
  let focused = $state(false);
  let dueDate = $state<string | null>(null);
  let important = $state(false);
  let input: HTMLInputElement | undefined = $state();
  let dueBtn: HTMLButtonElement | undefined = $state();

  $effect(() => {
    if (store.focusAddTask) input?.focus();
  });

  // Expanding only while there is text avoids layout jumps when the input loses focus.
  const expanded = $derived(title.length > 0 || !!dueDate || important);

  async function submit() {
    const value = title.trim();
    if (!value) return;
    title = "";
    const extra: { dueDate?: string; important?: boolean } = {};
    if (dueDate) extra.dueDate = dueDate;
    if (important) extra.important = true;
    dueDate = null;
    important = false;
    await store.addTask(value, extra);
  }

  function pickDue() {
    if (!dueBtn) return;
    menu.openAt(dueBtn, [
      { label: t("date.today"), icon: Calendar, run: () => (dueDate = todayKey()) },
      { label: t("date.tomorrow"), icon: CalendarArrowUp, run: () => (dueDate = addDays(todayKey(), 1)) },
      { label: t("detail.nextWeek"), icon: CalendarClock, run: () => (dueDate = nextWeekKey()) },
      ...(dueDate
        ? (["divider", { label: t("task.removeDueDate"), icon: CalendarX, danger: true, run: () => (dueDate = null) }] as const)
        : []),
    ]);
  }
</script>

<form
  class="add"
  class:expanded
  onsubmit={(e) => {
    e.preventDefault();
    submit();
  }}
>
  <div class="row">
    <span class="lead">
      {#if focused}<span class="ring"></span>{:else}<Plus size={20} strokeWidth={2} />{/if}
    </span>
    <input
      bind:this={input}
      bind:value={title}
      placeholder={t("task.add")}
      aria-label={t("task.add")}
      onfocus={() => (focused = true)}
      onblur={() => (focused = false)}
      onkeydown={(e) => {
        if (e.key === "Escape") {
          title = "";
          input?.blur();
        }
      }}
    />
  </div>
  {#if expanded}
    <div class="toolbar">
      <button
        type="button"
        bind:this={dueBtn}
        class="tool"
        class:on={dueDate}
        title={t("detail.addDueDate")}
        onmousedown={(e) => e.preventDefault()}
        onclick={pickDue}
      >
        <Calendar size={16} />
        {#if dueDate}<span>{relativeDay(dueDate)}</span>{/if}
      </button>
      <button
        type="button"
        class="tool"
        class:on={important}
        title={t("task.markImportant")}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => (important = !important)}
      >
        <Star size={16} fill={important ? "currentColor" : "none"} />
      </button>
      <span class="hint">{t("task.addHint")}</span>
      <button type="submit" class="submit" disabled={!title.trim()} onmousedown={(e) => e.preventDefault()}>
        {t("task.addButton")}
      </button>
    </div>
  {/if}
</form>

<style>
  .add {
    border-radius: 8px;
    background: var(--card);
    box-shadow: var(--shadow-sm);
    margin-bottom: 14px;
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    transition: box-shadow 0.2s;
  }
  .add.expanded,
  .add:focus-within {
    box-shadow: var(--shadow-md);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
    height: 52px;
  }
  .lead {
    display: grid;
    place-items: center;
    width: 20px;
    color: var(--accent);
  }
  .ring {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 1.5px solid var(--text-3);
  }
  input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font-size: 14.5px;
  }
  input::placeholder {
    color: var(--accent);
  }
  .add:focus-within input::placeholder {
    color: var(--text-3);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px 8px 46px;
    border-top: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface-2) 70%, transparent);
    border-radius: 0 0 8px 8px;
  }
  .tool {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 8px;
    border-radius: 6px;
    color: var(--text-2);
    font-size: 13px;
  }
  .tool:hover {
    background: var(--surface-hover);
  }
  .tool.on {
    color: var(--accent);
  }
  .hint {
    flex: 1;
    text-align: right;
    padding-right: 8px;
    font-size: 12px;
    color: var(--text-3);
  }
  .submit {
    height: 30px;
    padding: 0 14px;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
    color: var(--accent);
    font-weight: 600;
    font-size: 13px;
    background: var(--surface);
  }
  .submit:disabled {
    color: var(--text-3);
    cursor: default;
  }
  .submit:not(:disabled):hover {
    background: var(--accent);
    color: #fff;
    border-color: transparent;
  }
  @media (max-width: 640px) {
    .hint {
      display: none;
    }
    .toolbar {
      justify-content: flex-start;
    }
    .submit {
      margin-left: auto;
    }
  }
</style>
