<script lang="ts">
  import { onMount } from "svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import DetailPanel from "./components/DetailPanel.svelte";
  import Menu from "./components/Menu.svelte";
  import SettingsDialog from "./components/SettingsDialog.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import TaskListView from "./components/TaskListView.svelte";
  import Toasts from "./components/Toasts.svelte";
  import { i18n } from "./lib/i18n/index.svelte";
  import { menu } from "./lib/menu.svelte";
  import { handleShortcut } from "./lib/shortcuts";
  import { store } from "./lib/store.svelte";

  let systemDark = $state(false);
  let narrow = $state(false);

  onMount(() => {
    const dark = window.matchMedia("(prefers-color-scheme: dark)");
    const small = window.matchMedia("(max-width: 720px)");
    systemDark = dark.matches;
    narrow = small.matches;
    if (narrow) store.sidebarOpen = false;
    const onDark = (e: MediaQueryListEvent) => (systemDark = e.matches);
    const onSmall = (e: MediaQueryListEvent) => {
      narrow = e.matches;
      store.sidebarOpen = !e.matches;
    };
    dark.addEventListener("change", onDark);
    small.addEventListener("change", onSmall);
    store.init().catch((e) => store.toast(String(e)));
    return () => {
      dark.removeEventListener("change", onDark);
      small.removeEventListener("change", onSmall);
    };
  });

  const dark = $derived(store.colorScheme === "dark" || (store.colorScheme === "system" && systemDark));
  $effect(() => {
    document.documentElement.dataset.theme = dark ? "dark" : "light";
    document.documentElement.lang = i18n.lang;
    document.documentElement.style.setProperty("--accent", store.theme.accent);
  });

  // Close the sidebar overlay after navigating on small windows.
  $effect(() => {
    void store.view;
    if (narrow) store.sidebarOpen = false;
  });

  // Disable the default WebView context menu everywhere except text fields.
  function oncontextmenu(e: MouseEvent) {
    const el = e.target as HTMLElement;
    if (!el.closest("input, textarea")) e.preventDefault();
  }
</script>

<svelte:window
  onkeydown={(e) => handleShortcut(e, !!(store.settingsOpen || store.confirm || menu.current))}
  {oncontextmenu}
/>

<div class="app">
  {#if store.sidebarOpen}
    {#if narrow}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="scrim" onpointerdown={() => (store.sidebarOpen = false)}></div>
    {/if}
    <div class="sidebar-wrap" class:floating={narrow}>
      <Sidebar />
    </div>
  {/if}
  <main>
    <TaskListView />
    {#if store.selectedTask}
      {#key store.selectedTask.id}
        <DetailPanel task={store.selectedTask} />
      {/key}
    {/if}
  </main>
</div>

{#if menu.current}
  <Menu x={menu.current.x} y={menu.current.y} items={menu.current.items} onclose={() => menu.close()} />
{/if}
{#if store.settingsOpen}
  <SettingsDialog />
{/if}
{#if store.confirm}
  <ConfirmDialog req={store.confirm} />
{/if}
<Toasts />

<style>
  .app {
    position: relative;
    display: flex;
    height: 100%;
  }
  main {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
  }
  .sidebar-wrap {
    display: flex;
    height: 100%;
  }
  .sidebar-wrap.floating {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    z-index: 50;
    box-shadow: var(--shadow-lg);
    animation: slide-in 0.2s var(--ease);
  }
  .scrim {
    position: absolute;
    inset: 0;
    z-index: 49;
    background: var(--overlay);
  }
  @keyframes slide-in {
    from {
      transform: translateX(-24px);
      opacity: 0;
    }
  }
</style>
