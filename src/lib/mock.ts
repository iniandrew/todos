/**
 * In-browser implementation of the backend, used when the UI runs outside
 * Tauri (`pnpm dev` in a normal browser). Data lives in localStorage.
 * Mirrors the behaviour of `src-tauri/src/db.rs`.
 */
import type { BackupSummary, Counts, DueReminder, ImportMode, ListPatch, NewTask, Task, TaskList, TaskPatch } from "./types";

interface Backup {
  format: string;
  version: number;
  appVersion: string;
  exportedAt: string;
  lists: TaskList[];
  tasks: Task[];
  settings: Record<string, string>;
}

const LOCAL_ONLY_SETTINGS = ["lastView"];

interface Db {
  lists: TaskList[];
  tasks: Task[];
  settings: Record<string, string>;
  fired: string[];
}

const KEY = "todos.mock.v1";
const now = () => new Date().toISOString();
const uid = () =>
  typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : Math.random().toString(36).slice(2) + Date.now().toString(36);

function load(): Db {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return JSON.parse(raw);
  } catch {
    /* ignore */
  }
  return {
    lists: [
      { id: uid(), name: "Tasks", icon: null, theme: "blue", sortOrder: 0, isDefault: true, createdAt: now() },
    ],
    tasks: [],
    settings: {},
    fired: [],
  };
}

let db = load();
const save = () => {
  try {
    localStorage.setItem(KEY, JSON.stringify(db));
  } catch {
    /* ignore */
  }
};
const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v));
const defaultList = () => db.lists.find((l) => l.isDefault)!;
const bySort = (a: { sortOrder: number }, b: { sortOrder: number }) => a.sortOrder - b.sortOrder;
function task(id: string): Task {
  const t = db.tasks.find((t) => t.id === id);
  if (!t) throw new Error(`not found: task ${id}`);
  return t;
}
const minSort = (listId: string) =>
  Math.min(0, ...db.tasks.filter((t) => t.listId === listId).map((t) => t.sortOrder));

export const mock = {
  async get_lists(): Promise<TaskList[]> {
    return clone(
      [...db.lists].sort((a, b) => Number(b.isDefault) - Number(a.isDefault) || a.sortOrder - b.sortOrder),
    );
  },
  async create_list({ name, theme }: { name: string; theme?: string }): Promise<TaskList> {
    const list: TaskList = {
      id: uid(),
      name: name.trim(),
      icon: null,
      theme: theme ?? "purple",
      sortOrder: Math.max(0, ...db.lists.map((l) => l.sortOrder)) + 1,
      isDefault: false,
      createdAt: now(),
    };
    db.lists.push(list);
    save();
    return clone(list);
  },
  async update_list({ id, patch }: { id: string; patch: ListPatch }): Promise<TaskList> {
    const list = db.lists.find((l) => l.id === id)!;
    if (patch.name !== undefined) list.name = patch.name.trim() || list.name;
    if (patch.theme !== undefined) list.theme = patch.theme;
    if (patch.icon !== undefined) list.icon = patch.icon;
    save();
    return clone(list);
  },
  async delete_list({ id }: { id: string }): Promise<void> {
    db.lists = db.lists.filter((l) => l.id !== id || l.isDefault);
    db.tasks = db.tasks.filter((t) => db.lists.some((l) => l.id === t.listId));
    save();
  },
  async reorder_lists({ ids }: { ids: string[] }): Promise<void> {
    ids.forEach((id, i) => {
      const l = db.lists.find((l) => l.id === id);
      if (l) l.sortOrder = i + 1;
    });
    save();
  },
  async get_tasks({ view, today }: { view: string; today: string }): Promise<Task[]> {
    let out: Task[];
    if (view === "myDay") out = db.tasks.filter((t) => t.myDayDate === today).sort(bySort);
    else if (view === "important") out = db.tasks.filter((t) => t.important).sort(bySort);
    else if (view === "planned")
      out = db.tasks
        .filter((t) => t.dueDate || t.reminderAt)
        .sort((a, b) =>
          (a.dueDate ?? a.reminderAt!.slice(0, 10)).localeCompare(b.dueDate ?? b.reminderAt!.slice(0, 10)),
        );
    else {
      const listId = view === "tasks" ? defaultList().id : view.slice(5);
      out = db.tasks.filter((t) => t.listId === listId).sort(bySort);
    }
    return clone(out);
  },
  async search_tasks({ query }: { query: string }): Promise<Task[]> {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return clone(
      db.tasks.filter(
        (t) =>
          t.title.toLowerCase().includes(q) ||
          t.notes.toLowerCase().includes(q) ||
          t.steps.some((s) => s.title.toLowerCase().includes(q)),
      ),
    );
  },
  async create_task({ input, today }: { input: NewTask; today: string }): Promise<Task> {
    const listId = input.listId ?? defaultList().id;
    const ts = now();
    const t: Task = {
      id: uid(),
      listId,
      title: input.title.trim(),
      notes: "",
      completed: false,
      completedAt: null,
      important: !!input.important,
      dueDate: input.dueDate ?? null,
      reminderAt: null,
      myDayDate: input.myDay ? today : null,
      sortOrder: minSort(listId) - 1,
      createdAt: ts,
      updatedAt: ts,
      steps: [],
    };
    db.tasks.push(t);
    save();
    return clone(t);
  },
  async update_task({ id, patch, today }: { id: string; patch: TaskPatch; today: string }): Promise<Task> {
    const t = task(id);
    if (patch.title !== undefined) t.title = patch.title.trim() || t.title;
    if (patch.notes !== undefined) t.notes = patch.notes;
    if (patch.completed !== undefined) {
      t.completed = patch.completed;
      t.completedAt = patch.completed ? now() : null;
    }
    if (patch.important !== undefined) t.important = patch.important;
    if (patch.listId !== undefined && patch.listId !== t.listId) {
      t.sortOrder = minSort(patch.listId) - 1;
      t.listId = patch.listId;
    }
    if (patch.myDay !== undefined) t.myDayDate = patch.myDay ? today : null;
    if (patch.dueDate !== undefined) t.dueDate = patch.dueDate;
    if (patch.reminderAt !== undefined) {
      t.reminderAt = patch.reminderAt;
      db.fired = db.fired.filter((f) => f !== id);
    }
    t.updatedAt = now();
    save();
    return clone(t);
  },
  async delete_task({ id }: { id: string }): Promise<void> {
    db.tasks = db.tasks.filter((t) => t.id !== id);
    save();
  },
  async restore_task({ task: t }: { task: Task }): Promise<Task> {
    if (!db.lists.some((l) => l.id === t.listId)) t.listId = defaultList().id;
    db.tasks.push(clone(t));
    save();
    return clone(t);
  },
  async reorder_tasks({ ids }: { ids: string[] }): Promise<void> {
    ids.forEach((id, i) => {
      const t = db.tasks.find((t) => t.id === id);
      if (t) t.sortOrder = i;
    });
    save();
  },
  async get_counts({ today }: { today: string }): Promise<Counts> {
    const open = db.tasks.filter((t) => !t.completed);
    const lists: Record<string, number> = {};
    for (const t of open) lists[t.listId] = (lists[t.listId] ?? 0) + 1;
    return {
      myDay: open.filter((t) => t.myDayDate === today).length,
      important: open.filter((t) => t.important).length,
      planned: open.filter((t) => t.dueDate || t.reminderAt).length,
      tasks: lists[defaultList().id] ?? 0,
      lists,
    };
  },
  async add_step({ taskId, title }: { taskId: string; title: string }): Promise<Task> {
    const t = task(taskId);
    t.steps.push({
      id: uid(),
      taskId,
      title: title.trim(),
      completed: false,
      sortOrder: Math.max(0, ...t.steps.map((s) => s.sortOrder)) + 1,
    });
    t.updatedAt = now();
    save();
    return clone(t);
  },
  async update_step({ id, title, completed }: { id: string; title?: string; completed?: boolean }): Promise<Task> {
    const t = db.tasks.find((t) => t.steps.some((s) => s.id === id))!;
    const s = t.steps.find((s) => s.id === id)!;
    if (title !== undefined) s.title = title.trim() || s.title;
    if (completed !== undefined) s.completed = completed;
    save();
    return clone(t);
  },
  async delete_step({ id }: { id: string }): Promise<Task> {
    const t = db.tasks.find((t) => t.steps.some((s) => s.id === id))!;
    t.steps = t.steps.filter((s) => s.id !== id);
    save();
    return clone(t);
  },
  async promote_step({ id, today }: { id: string; today: string }): Promise<Task> {
    const t = db.tasks.find((t) => t.steps.some((s) => s.id === id))!;
    const s = t.steps.find((s) => s.id === id)!;
    await mock.create_task({ input: { listId: t.listId, title: s.title }, today });
    return mock.delete_step({ id });
  },
  async get_settings(): Promise<Record<string, string>> {
    return clone(db.settings);
  },
  async set_setting({ key, value }: { key: string; value: string }): Promise<void> {
    db.settings[key] = value;
    save();
  },
  // ------------------------------------------------------- backup (mirrors backup.rs)
  exportBackup(): Backup {
    const settings = { ...db.settings };
    for (const k of LOCAL_ONLY_SETTINGS) delete settings[k];
    return clone({
      format: "todos-backup",
      version: 1,
      appVersion: "0.1.0",
      exportedAt: now(),
      lists: db.lists,
      tasks: db.tasks,
      settings,
    });
  },
  summary(b: Backup): BackupSummary {
    return {
      lists: b.lists.filter((l) => !l.isDefault).length,
      tasks: b.tasks.length,
      steps: b.tasks.reduce((n, t) => n + (t.steps?.length ?? 0), 0),
      exportedAt: b.exportedAt,
    };
  },
  parseBackup(json: string): Backup {
    let b: Partial<Backup>;
    try {
      b = JSON.parse(json);
    } catch {
      throw new Error("not a Todos backup file");
    }
    if (b?.format !== "todos-backup") throw new Error("not a Todos backup file");
    if (typeof b.version !== "number") throw new Error("backup has no version");
    if (b.version > 1) throw new Error("this backup was made by a newer version of Todos; please update the app");
    if (!Array.isArray(b.lists) || !Array.isArray(b.tasks)) throw new Error("damaged backup file");
    return { settings: {}, appVersion: "", ...b } as Backup;
  },
  importBackup(json: string, mode: ImportMode): BackupSummary {
    const b = mock.parseBackup(json);
    const local = defaultList();
    if (mode === "replace") {
      db.lists = [local];
      db.tasks = [];
    }
    const listIds = new Map<string, string>();
    for (const l of b.lists) {
      if (l.isDefault) {
        local.theme = l.theme;
        local.icon = l.icon;
        listIds.set(l.id, local.id);
      } else {
        const existing = db.lists.find((x) => x.id === l.id);
        if (existing) Object.assign(existing, { name: l.name, icon: l.icon, theme: l.theme, sortOrder: l.sortOrder });
        else db.lists.push({ ...clone(l), isDefault: false });
        listIds.set(l.id, l.id);
      }
    }
    for (const t of b.tasks) {
      const listId = listIds.get(t.listId) ?? (db.lists.some((l) => l.id === t.listId) ? t.listId : local.id);
      const task: Task = { ...clone(t), listId, steps: clone(t.steps ?? []) };
      const i = db.tasks.findIndex((x) => x.id === t.id);
      if (i >= 0) {
        // Merge keeps local-only steps, like the SQL upsert.
        const keep = db.tasks[i].steps.filter((s) => !task.steps.some((n) => n.id === s.id));
        task.steps = [...task.steps, ...keep];
        db.tasks[i] = task;
      } else db.tasks.push(task);
      if (t.reminderAt && new Date(t.reminderAt) <= new Date()) {
        if (!db.fired.includes(t.id)) db.fired.push(t.id);
      } else db.fired = db.fired.filter((f) => f !== t.id);
    }
    for (const [k, v] of Object.entries(b.settings ?? {})) if (!LOCAL_ONLY_SETTINGS.includes(k)) db.settings[k] = v;
    save();
    return mock.summary(b);
  },
  toCsv(): string {
    const names = new Map(db.lists.map((l) => [l.id, l.name]));
    const field = (v: string) => (/[",\r\n]/.test(v) ? `"${v.replaceAll('"', '""')}"` : v);
    const yn = (b: boolean) => (b ? "yes" : "no");
    const rows = [["List", "Title", "Completed", "Important", "Due date", "Reminder", "My Day", "Notes", "Steps", "Created", "Completed at"]];
    for (const t of db.tasks)
      rows.push([
        names.get(t.listId) ?? "",
        t.title,
        yn(t.completed),
        yn(t.important),
        t.dueDate ?? "",
        t.reminderAt ?? "",
        t.myDayDate ?? "",
        t.notes,
        t.steps.map((s) => `[${s.completed ? "x" : " "}] ${s.title}`).join(" | "),
        t.createdAt,
        t.completedAt ?? "",
      ]);
    return "\ufeff" + rows.map((r) => r.map(field).join(",")).join("\r\n") + "\r\n";
  },
  /** Mock equivalent of the Rust reminder thread. */
  takeDueReminders(): DueReminder[] {
    const due: DueReminder[] = [];
    for (const t of db.tasks) {
      if (t.reminderAt && !t.completed && !db.fired.includes(t.id) && new Date(t.reminderAt) <= new Date()) {
        db.fired.push(t.id);
        due.push({ taskId: t.id, title: t.title, listName: db.lists.find((l) => l.id === t.listId)?.name ?? "" });
      }
    }
    if (due.length) save();
    return due;
  },
};
