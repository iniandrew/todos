<script lang="ts">
  import { Upload } from "@lucide/svelte";
  import { inTauri } from "../lib/api";
  import { i18n, t } from "../lib/i18n/index.svelte";
  import { store } from "../lib/store.svelte";
  import type { BackupSource, BackupSummary, ImportMode } from "../lib/types";
  import Modal from "./Modal.svelte";

  let { preview }: { preview: { source: BackupSource; summary: BackupSummary } } = $props();
  let mode = $state<ImportMode>("merge");

  const s = $derived(preview.summary);
  const exported = $derived(
    new Date(s.exportedAt).toLocaleString(i18n.locale, { dateStyle: "medium", timeStyle: "short" }),
  );
  const cancel = () => (store.importPreview = null);
</script>

<Modal label={t("data.importTitle")} width={400} onclose={cancel}>
  <div class="body">
    <div class="icon"><Upload size={24} strokeWidth={2.2} /></div>
    <h2>{t("data.importTitle")}</h2>
    <p class="summary">{t("data.summary", { lists: s.lists, tasks: s.tasks, steps: s.steps })}</p>
    <p class="date">{t("data.exportedOn", { date: exported })}</p>

    <div class="options" role="radiogroup" aria-label={t("data.importTitle")}>
      <label class="option" class:on={mode === "merge"}>
        <input type="radio" name="mode" value="merge" bind:group={mode} />
        <span class="radio"></span>
        <span class="text">
          <b>{t("data.merge")}</b>
          <small>{t("data.mergeHint")}</small>
        </span>
      </label>
      <label class="option danger" class:on={mode === "replace"}>
        <input type="radio" name="mode" value="replace" bind:group={mode} />
        <span class="radio"></span>
        <span class="text">
          <b>{t("data.replace")}</b>
          <small>{t("data.replaceHint")}{inTauri ? ` ${t("data.replaceSafety")}` : ""}</small>
        </span>
      </label>
    </div>

    <div class="actions">
      <button class="btn" onclick={cancel}>{t("common.cancel")}</button>
      <button class="btn primary" class:destructive={mode === "replace"} data-autofocus onclick={() => store.confirmImport(mode)}>
        {t("data.importButton")}
      </button>
    </div>
  </div>
</Modal>

<style>
  .body {
    padding: 24px 20px 18px;
    background: var(--elevated);
    text-align: center;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    margin: 0 auto 12px;
    border-radius: 14px;
    background: var(--accent);
    color: #fff;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 16px;
    font-weight: 700;
  }
  .summary {
    margin: 0;
    font-size: 13.5px;
    font-weight: 500;
  }
  .date {
    margin: 2px 0 16px;
    font-size: 12.5px;
    color: var(--text-2);
  }
  .options {
    display: grid;
    gap: 8px;
    text-align: left;
  }
  .option {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 11px 12px;
    border-radius: 11px;
    background: var(--fill-2);
    box-shadow: inset 0 0 0 1.5px transparent;
    transition:
      box-shadow 0.15s,
      background 0.15s;
  }
  .option:hover {
    background: var(--fill);
  }
  .option.on {
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }
  .option.danger.on {
    box-shadow: inset 0 0 0 1.5px var(--danger);
  }
  .option input {
    position: absolute;
    opacity: 0;
  }
  .radio {
    flex: none;
    width: 18px;
    height: 18px;
    margin-top: 1px;
    border-radius: 50%;
    border: 1.5px solid var(--text-3);
    transition:
      border 0.15s,
      background 0.15s;
  }
  .option.on .radio {
    border: 5px solid var(--accent);
    background: #fff;
  }
  .option.danger.on .radio {
    border-color: var(--danger);
  }
  .option input:focus-visible + .radio {
    outline: 3px solid color-mix(in srgb, var(--accent) 45%, transparent);
    outline-offset: 1px;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .text b {
    font-size: 14px;
    font-weight: 600;
  }
  .text small {
    font-size: 12.5px;
    line-height: 1.4;
    color: var(--text-2);
  }
  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin-top: 18px;
  }
  .btn {
    height: 32px;
    border-radius: 8px;
    font-size: 13.5px;
    font-weight: 500;
    background: var(--fill);
  }
  .btn.primary {
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }
  .btn.primary.destructive {
    background: var(--danger);
  }
  .btn:hover {
    filter: brightness(0.96);
  }
</style>
