<script lang="ts">
  import { CalendarDays, CalendarArrowUp, CalendarClock, CalendarX, Plus, Star } from "@lucide/svelte";
  import { fade } from "svelte/transition";
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

  const showTools = $derived(title.length > 0 || !!dueDate || important);

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
      { label: t("date.today"), icon: CalendarDays, run: () => (dueDate = todayKey()) },
      { label: t("date.tomorrow"), icon: CalendarArrowUp, run: () => (dueDate = addDays(todayKey(), 1)) },
      { label: t("detail.nextWeek"), icon: CalendarClock, run: () => (dueDate = nextWeekKey()) },
      ...(dueDate
        ? (["divider", { label: t("task.removeDueDate"), icon: CalendarX, danger: true, run: () => (dueDate = null) }] as const)
        : []),
    ], "right");
  }
</script>

<form
  class="add"
  class:focused
  onsubmit={(e) => {
    e.preventDefault();
    submit();
  }}
>
  <span class="plus"><Plus size={15} strokeWidth={2.6} /></span>
  <input
    bind:this={input}
    bind:value={title}
    placeholder={t("task.newTask")}
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
  {#if showTools}
    <div class="tools" transition:fade={{ duration: 120 }}>
      <button
        type="button"
        bind:this={dueBtn}
        class="pill"
        class:on={dueDate}
        title={t("detail.addDueDate")}
        onmousedown={(e) => e.preventDefault()}
        onclick={pickDue}
      >
        <CalendarDays size={14} strokeWidth={2.1} />
        {#if dueDate}{relativeDay(dueDate)}{/if}
      </button>
      <button
        type="button"
        class="pill"
        class:on={important}
        title={t("task.markImportant")}
        onmousedown={(e) => e.preventDefault()}
        onclick={() => (important = !important)}
      >
        <Star size={14} strokeWidth={2.1} fill={important ? "currentColor" : "none"} />
      </button>
    </div>
  {/if}
</form>

<style>
  .add {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 44px;
    padding: 0 8px 0 10px;
    margin-bottom: 4px;
    border-radius: 10px;
    transition: background 0.15s;
  }
  .add:hover,
  .add.focused {
    background: var(--hover);
  }
  .plus {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
    flex: none;
    transition: transform 0.25s var(--spring);
  }
  .focused .plus {
    transform: rotate(90deg);
  }
  input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    font-size: 15px;
  }
  input::placeholder {
    color: var(--text-2);
  }
  .tools {
    display: flex;
    gap: 6px;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
    padding: 0 9px;
    border-radius: 13px;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-2);
    background: var(--fill-2);
    transition:
      background 0.15s,
      color 0.15s;
  }
  .pill:hover {
    background: var(--fill);
  }
  .pill.on {
    color: #fff;
    background: var(--accent);
  }
</style>
