<script lang="ts">
  import { TriangleAlert } from "@lucide/svelte";
  import { t } from "../lib/i18n/index.svelte";
  import type { ConfirmRequest } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";

  let { req }: { req: ConfirmRequest } = $props();
</script>

<Modal label={req.title} width={400} onclose={() => req.resolve(false)}>
  <div class="body">
    <div class="icon"><TriangleAlert size={22} /></div>
    <h2>{req.title}</h2>
    <p>{req.body}</p>
  </div>
  <div class="actions">
    <button class="btn" data-autofocus onclick={() => req.resolve(false)}>{t("common.cancel")}</button>
    <button class="btn danger" onclick={() => req.resolve(true)}>{req.confirmLabel}</button>
  </div>
</Modal>

<style>
  .body {
    padding: 22px 22px 8px;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 50%;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    margin-bottom: 12px;
  }
  h2 {
    margin: 0 0 6px;
    font-size: 17px;
  }
  p {
    margin: 0;
    color: var(--text-2);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 22px 20px;
  }
  .btn {
    height: 34px;
    padding: 0 16px;
    border-radius: 8px;
    font-weight: 600;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
  }
  .btn:hover {
    background: var(--surface-hover);
  }
  .btn.danger {
    background: var(--danger);
    border-color: transparent;
    color: #fff;
  }
  .btn.danger:hover {
    filter: brightness(1.08);
  }
</style>
