<script lang="ts">
  import { t } from "../lib/i18n/index.svelte";
  import type { ConfirmRequest } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";

  let { req }: { req: ConfirmRequest } = $props();
</script>

<Modal label={req.title} width={300} onclose={() => req.resolve(false)}>
  <div class="body">
    <img src="/favicon.svg" alt="" width="56" height="56" />
    <h2>{req.title}</h2>
    <p>{req.body}</p>
    <div class="actions">
      <button class="btn" data-autofocus onclick={() => req.resolve(false)}>{t("common.cancel")}</button>
      <button class="btn danger" onclick={() => req.resolve(true)}>{req.confirmLabel}</button>
    </div>
  </div>
</Modal>

<style>
  .body {
    padding: 22px 18px 16px;
    text-align: center;
    background: var(--elevated);
  }
  img {
    display: block;
    margin: 0 auto 10px;
  }
  h2 {
    margin: 0 0 6px;
    font-size: 15px;
    font-weight: 700;
  }
  p {
    margin: 0 0 18px;
    font-size: 12.5px;
    line-height: 1.45;
    color: var(--text-2);
  }
  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .btn {
    height: 30px;
    border-radius: 8px;
    font-size: 13.5px;
    font-weight: 500;
    background: var(--fill);
  }
  .btn:hover {
    filter: brightness(0.96);
  }
  .btn.danger {
    background: var(--danger);
    color: #fff;
  }
</style>
