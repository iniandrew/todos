<script lang="ts">
  import { CalendarDays, House, List, Search, Star, Sun } from "@lucide/svelte";
  import { fade } from "svelte/transition";
  import type { TKey } from "../lib/i18n/index.svelte";
  import { t } from "../lib/i18n/index.svelte";
  import { store } from "../lib/store.svelte";

  const kind = $derived(
    store.searching ? "search" : store.view.startsWith("list:") ? "tasks" : (store.view as "myDay" | "important" | "planned" | "tasks"),
  );
  const Icon = $derived(
    { myDay: Sun, important: Star, planned: CalendarDays, tasks: store.view.startsWith("list:") ? List : House, search: Search }[kind],
  );
</script>

<div class="empty" in:fade={{ duration: 250 }}>
  <div class="art"><Icon size={34} strokeWidth={1.7} /></div>
  <h2>{t(`empty.${kind}.title` as TKey)}</h2>
  <p>{t(`empty.${kind}.body` as TKey)}</p>
</div>

<style>
  .empty {
    margin: 12vh auto 0;
    max-width: 300px;
    text-align: center;
  }
  .art {
    display: grid;
    place-items: center;
    width: 72px;
    height: 72px;
    margin: 0 auto 16px;
    border-radius: 22px;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }
  h2 {
    margin: 0 0 6px;
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  p {
    margin: 0;
    font-size: 13.5px;
    line-height: 1.5;
    color: var(--text-2);
  }
</style>
