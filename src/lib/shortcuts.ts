import { store } from "./store.svelte";
import type { TKey } from "./i18n/index.svelte";
import type { ViewId } from "./types";

export const isMac = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
export const MOD = isMac ? "⌘" : "Ctrl";

export const SHORTCUTS: { keys: string[]; label: TKey }[] = [
  { keys: [MOD, "N"], label: "shortcut.newTask" },
  { keys: [MOD, "L"], label: "shortcut.newList" },
  { keys: [MOD, "F"], label: "shortcut.search" },
  { keys: [MOD, "D"], label: "shortcut.complete" },
  { keys: [MOD, "T"], label: "shortcut.myDay" },
  { keys: [MOD, "I"], label: "shortcut.important" },
  { keys: ["Del"], label: "shortcut.delete" },
  { keys: ["↑", "↓"], label: "shortcut.navigate" },
  { keys: [MOD, "1–4"], label: "shortcut.views" },
  { keys: [MOD, "B"], label: "shortcut.sidebar" },
  { keys: [MOD, ","], label: "shortcut.settings" },
  { keys: ["Esc"], label: "shortcut.close" },
];

const VIEWS: ViewId[] = ["myDay", "important", "planned", "tasks"];

function isTyping(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  return !!el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable);
}

export function handleShortcut(e: KeyboardEvent, overlayOpen: boolean) {
  const mod = isMac ? e.metaKey : e.ctrlKey;
  const key = e.key.toLowerCase();
  const task = store.selectedTask;

  if (overlayOpen) return;

  if (mod && !e.shiftKey && !e.altKey) {
    let handled = true;
    if (key === "n") store.focusAddTask++;
    else if (key === "l") store.createList();
    else if (key === "f") store.focusSearch++;
    else if (key === "b") store.sidebarOpen = !store.sidebarOpen;
    else if (key === ",") store.settingsOpen = true;
    else if (key >= "1" && key <= "4") store.setView(VIEWS[Number(key) - 1]);
    else if (key === "d" && task) store.toggleComplete(task);
    else if (key === "t" && task) store.toggleMyDay(task);
    else if (key === "i" && task) store.toggleImportant(task);
    else handled = false;
    if (handled) e.preventDefault();
    return;
  }

  if (isTyping(e.target)) return;

  if (e.key === "Escape" && store.selectedId) {
    store.select(null);
  } else if ((e.key === "Delete" || (isMac && e.key === "Backspace")) && task) {
    e.preventDefault();
    store.deleteTask(task);
  } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    store.moveSelection(e.key === "ArrowDown" ? 1 : -1, store.visibleTasks);
    const id = store.selectedId;
    if (id) document.querySelector(`[data-task-id="${id}"]`)?.scrollIntoView({ block: "nearest" });
  }
}
