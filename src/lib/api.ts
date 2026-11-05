/** Typed access to the Rust backend (or the browser mock outside Tauri). */
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { todayKey } from "./dates";
import { mock } from "./mock";
import type {
  BackupFile,
  BackupSource,
  BackupSummary,
  Counts,
  DueReminder,
  ImportMode,
  ListPatch,
  NewTask,
  Task,
  TaskList,
  TaskPatch,
  ViewId,
} from "./types";

export const inTauri = isTauri();

function call<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (inTauri) return invoke<T>(cmd, args);
  const fn = (mock as unknown as Record<string, (a: unknown) => Promise<T>>)[cmd];
  return fn(args);
}

/** Browser fallback: hand the user a file to save. */
function download(name: string, contents: string, type: string) {
  const url = URL.createObjectURL(new Blob([contents], { type }));
  const a = Object.assign(document.createElement("a"), { href: url, download: name });
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Browser fallback: let the user pick a file. Resolves `null` on cancel. */
function pickFile(accept: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = Object.assign(document.createElement("input"), { type: "file", accept });
    input.addEventListener("change", () => resolve(input.files?.[0] ?? null));
    input.addEventListener("cancel", () => resolve(null));
    input.click();
  });
}

const JSON_FILTER = [{ name: "Todos backup", extensions: ["json"] }];

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

  // --------------------------------------------------------------- backup

  /** Saves a JSON backup. Returns `null` if the user cancelled. */
  async exportBackup(): Promise<BackupSummary | null> {
    const name = `todos-backup-${todayKey()}.json`;
    if (inTauri) {
      const path = await saveDialog({ defaultPath: name, filters: JSON_FILTER });
      return path ? call<BackupSummary>("export_backup", { path }) : null;
    }
    const backup = mock.exportBackup();
    download(name, JSON.stringify(backup, null, 2), "application/json");
    return mock.summary(backup);
  },

  /** Saves a CSV of all tasks. Returns `false` if the user cancelled. */
  async exportCsv(): Promise<boolean> {
    const name = `todos-${todayKey()}.csv`;
    if (inTauri) {
      const path = await saveDialog({ defaultPath: name, filters: [{ name: "CSV", extensions: ["csv"] }] });
      if (!path) return false;
      await call<void>("export_csv", { path });
      return true;
    }
    download(name, mock.toCsv(), "text/csv;charset=utf-8");
    return true;
  },

  /** Lets the user pick a backup file and validates it. Returns `null` on cancel. */
  async pickBackup(): Promise<{ source: BackupSource; summary: BackupSummary } | null> {
    if (inTauri) {
      const path = await openDialog({ multiple: false, directory: false, filters: JSON_FILTER });
      if (!path) return null;
      return { source: { kind: "path", path }, summary: await api.previewBackup(path) };
    }
    const file = await pickFile(".json,application/json");
    if (!file) return null;
    const json = await file.text();
    return { source: { kind: "data", json }, summary: mock.summary(mock.parseBackup(json)) };
  },

  previewBackup: (path: string) => call<BackupSummary>("preview_backup", { path }),

  async importBackup(source: BackupSource, mode: ImportMode): Promise<BackupSummary> {
    if (source.kind === "path") return call<BackupSummary>("import_backup", { path: source.path, mode });
    return mock.importBackup(source.json, mode);
  },

  listAutoBackups: () => (inTauri ? call<BackupFile[]>("list_auto_backups") : Promise.resolve([])),
  openDataFolder: () => call<void>("open_data_folder"),

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
