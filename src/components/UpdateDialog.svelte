<script lang="ts">
  import { t } from "../lib/i18n/index.svelte";
  import { renderNotes } from "../lib/markdown";
  import { updates } from "../lib/updates.svelte";
  import Modal from "./Modal.svelte";

  const a = $derived(updates.available);
  const busy = $derived(updates.status === "downloading" || updates.status === "installing");
  const percent = $derived(updates.progress < 0 ? "" : `${Math.round(updates.progress * 100)}%`);
  const close = () => {
    if (!busy) updates.dialogOpen = false;
  };
</script>

{#if a}
  <Modal label={t("update.title")} width={460} onclose={close}>
    <div class="body">
      <img src="/favicon.svg" alt="" width="60" height="60" />
      <h2>{t("update.title")}</h2>
      <p class="sub">{t("update.subtitle", { version: a.version, current: updates.current })}</p>

      {#if a.notes.trim()}
        <div class="notes-label">{t("update.whatsNew")}</div>
        <div class="notes">{@html renderNotes(a.notes)}</div>
      {/if}

      {#if updates.mode === "notify"}
        <p class="hint">{t("update.packageHint")}</p>
      {/if}

      {#if busy}
        <div class="progress" class:indeterminate={updates.progress < 0 || updates.status === "installing"}>
          <span style="width:{updates.progress < 0 ? 100 : updates.progress * 100}%"></span>
        </div>
        <div class="progress-label">
          {updates.status === "installing" ? t("update.installing") : t("update.downloading", { percent })}
        </div>
      {:else}
        {#if updates.status === "error" && updates.error}
          <p class="error">{updates.error}</p>
        {/if}
        <div class="actions">
          <button class="link" onclick={() => updates.skip()}>{t("update.skip")}</button>
          <span class="spacer"></span>
          <button class="btn" onclick={close}>{t("update.later")}</button>
          <button class="btn primary" data-autofocus onclick={() => updates.install()}>
            {updates.mode === "inApp" ? t("update.install") : t("update.download")}
          </button>
        </div>
      {/if}
    </div>
  </Modal>
{/if}

<style>
  .body {
    padding: 26px 22px 18px;
    background: var(--elevated);
    text-align: center;
  }
  img {
    display: block;
    margin: 0 auto 12px;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 16px;
    font-weight: 700;
  }
  .sub {
    margin: 0 0 16px;
    font-size: 13px;
    color: var(--text-2);
  }
  .notes-label {
    text-align: left;
    margin: 0 2px 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
  }
  .notes {
    max-height: 240px;
    overflow-y: auto;
    padding: 4px 14px 10px;
    border-radius: 10px;
    background: var(--fill-2);
    text-align: left;
    font-size: 13px;
    line-height: 1.5;
    user-select: text;
    -webkit-user-select: text;
  }
  .notes :global(h4) {
    margin: 10px 0 4px;
    font-size: 13px;
    font-weight: 700;
  }
  .notes :global(ul) {
    margin: 0;
    padding-left: 18px;
  }
  .notes :global(li) {
    margin: 3px 0;
  }
  .notes :global(p) {
    margin: 6px 0;
  }
  .notes :global(code) {
    font-size: 12px;
    padding: 1px 4px;
    border-radius: 4px;
    background: var(--fill);
  }
  .notes :global(a) {
    color: var(--accent);
  }
  .hint {
    margin: 12px 0 0;
    font-size: 12.5px;
    color: var(--text-2);
    text-align: left;
  }
  .error {
    margin: 12px 0 0;
    font-size: 12.5px;
    color: var(--danger);
    text-align: left;
    overflow-wrap: anywhere;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 18px;
  }
  .spacer {
    flex: 1;
  }
  .link {
    font-size: 13px;
    color: var(--text-2);
  }
  .link:hover {
    color: var(--text);
  }
  .btn {
    height: 32px;
    padding: 0 14px;
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
  .btn:hover {
    filter: brightness(0.96);
  }
  .progress {
    position: relative;
    height: 6px;
    margin-top: 20px;
    border-radius: 3px;
    background: var(--fill);
    overflow: hidden;
  }
  .progress span {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: var(--accent);
    transition: width 0.2s;
  }
  .progress.indeterminate span {
    width: 35% !important;
    animation: slide 1.1s ease-in-out infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(300%);
    }
  }
  .progress-label {
    margin-top: 8px;
    font-size: 12.5px;
    color: var(--text-2);
  }
</style>
