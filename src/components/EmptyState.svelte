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

<div class="empty" in:fade={{ duration: 200 }}>
  <div class="art">
    <Icon size={44} strokeWidth={1.4} />
  </div>
  <h2>{t(`empty.${kind}.title` as TKey)}</h2>
  <p>{t(`empty.${kind}.body` as TKey)}</p>
</div>

<style>
  .empty {
    margin: 8vh auto 0;
    max-width: 340px;
    padding: 28px 24px;
    text-align: center;
    border-radius: 16px;
    background: var(--card);
    box-shadow: var(--shadow-md);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
  }
  .art {
    display: grid;
    place-items: center;
    width: 88px;
    height: 88px;
    margin: 0 auto 14px;
    border-radius: 50%;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
    animation: float 4s ease-in-out infinite;
  }
  @keyframes float {
    50% {
      transform: translateY(-5px);
    }
  }
  h2 {
    margin: 0 0 6px;
    font-size: 17px;
  }
  p {
    margin: 0;
    color: var(--text-2);
  }
</style>
