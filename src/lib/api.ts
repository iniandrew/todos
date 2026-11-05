/** Typed access to the Rust backend (or the browser mock outside Tauri). */
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { todayKey } from "./dates";
import { mock } from "./mock";
import type { Counts, DueReminder, ListPatch, NewTask, Task, TaskList, TaskPatch, ViewId } from "./types";

export const inTauri = isTauri();

function call<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (inTauri) return invoke<T>(cmd, args);
  const fn = (mock as unknown as Record<string, (a: unknown) => Promise<T>>)[cmd];
  return fn(args);
}

export const api = {
  getLists: () => call<TaskList[]>("get_lists"),
  createList: (name: string, theme?: string) => call<TaskList>("create_list", { name, theme }),
  updateList: (id: string, patch: ListPatch) => call<TaskList>("update_list", { id, patch }),
  deleteList: (id: string) => call<void>("delete_list", { id }),
  reorderLists: (ids: string[]) => call<void>("reorder_lists", { ids }),

  getTasks: (view: ViewId) => call<Task[]>("get_tasks", { view, today: todayKey() }),
  searchTasks: (query: string) => call<Task[]>("search_tasks", { query }),
  createTask: (input: NewTask) => call<Task>("create_task", { input, today: todayKey() }),
  updateTask: (id: string, patch: TaskPatch) => call<Task>("update_task", { id, patch, today: todayKey() }),
  deleteTask: (id: string) => call<void>("delete_task", { id }),
  restoreTask: (task: Task) => call<Task>("restore_task", { task }),
  reorderTasks: (ids: string[]) => call<void>("reorder_tasks", { ids }),
  getCounts: () => call<Counts>("get_counts", { today: todayKey() }),

  addStep: (taskId: string, title: string) => call<Task>("add_step", { taskId, title }),
  updateStep: (id: string, title?: string, completed?: boolean) =>
    call<Task>("update_step", { id, title, completed }),
  deleteStep: (id: string) => call<Task>("delete_step", { id }),
  promoteStep: (id: string) => call<Task>("promote_step", { id, today: todayKey() }),

  getSettings: () => call<Record<string, string>>("get_settings"),
  setSetting: (key: string, value: string) => call<void>("set_setting", { key, value }),

  /** Subscribe to reminders fired by the backend. */
  onReminders(cb: (due: DueReminder[]) => void): () => void {
    if (inTauri) {
      const un = listen<DueReminder[]>("reminders-fired", (e) => cb(e.payload));
      return () => void un.then((f) => f());
    }
    const timer = setInterval(() => {
      const due = mock.takeDueReminders();
      if (due.length) {
        cb(due);
        if ("Notification" in window && Notification.permission === "granted")
          for (const r of due) new Notification(r.title, { body: r.listName });
      }
    }, 15_000);
    return () => clearInterval(timer);
  },
};
