import { api } from "./api";
import { todayKey } from "./dates";
import { i18n, t } from "./i18n/index.svelte";
import { playDing } from "./sound";
import { DEFAULT_VIEW_THEMES, getTheme, themeForIndex, type ListTheme } from "./themes";
import type { Counts, DueReminder, NewTask, Task, TaskList, TaskPatch, ViewId } from "./types";

export type ColorScheme = "system" | "light" | "dark";

export interface Toast {
  id: number;
  message: string;
  icon?: "trash" | "bell" | "sun";
  action?: { label: string; run: () => void };
}

export interface ConfirmRequest {
  title: string;
  body: string;
  confirmLabel: string;
  resolve: (ok: boolean) => void;
}

const EMPTY_COUNTS: Counts = { myDay: 0, important: 0, planned: 0, tasks: 0, lists: {} };

class AppStore {
  lists = $state<TaskList[]>([]);
  tasks = $state<Task[]>([]);
  counts = $state<Counts>(EMPTY_COUNTS);
  settings = $state<Record<string, string>>({});

  view = $state<ViewId>("myDay");
  searchQuery = $state("");
  selectedId = $state<string | null>(null);
  sidebarOpen = $state(true);
  settingsOpen = $state(false);
  /** List id whose name is being edited inline in the header. */
  renamingListId = $state<string | null>(null);
  toasts = $state<Toast[]>([]);
  confirm = $state<ConfirmRequest | null>(null);
  ready = $state(false);
  /** Bumped to ask the add-task input to focus itself. */
  focusAddTask = $state(0);
  focusSearch = $state(0);
  today = $state(todayKey());

  searching = $derived(this.searchQuery.trim().length > 0);
  currentList = $derived.by<TaskList | null>(() => {
    if (this.view === "tasks") return this.lists.find((l) => l.isDefault) ?? null;
    if (this.view.startsWith("list:")) return this.lists.find((l) => l.id === this.view.slice(5)) ?? null;
    return null;
  });
  customLists = $derived(this.lists.filter((l) => !l.isDefault));
  incomplete = $derived(this.tasks.filter((t) => !t.completed));
  completed = $derived(
    this.tasks
      .filter((t) => t.completed)
      .sort((a, b) => (b.completedAt ?? "").localeCompare(a.completedAt ?? "")),
  );
  selectedTask = $derived(this.tasks.find((t) => t.id === this.selectedId) ?? null);
  theme = $derived.by<ListTheme>(() => {
    if (this.searching) return getTheme(DEFAULT_VIEW_THEMES.search);
    if (this.view.startsWith("list:")) return getTheme(this.currentList?.theme);
    return getTheme(this.settings[`viewTheme.${this.view}`] ?? DEFAULT_VIEW_THEMES[this.view]);
  });
  colorScheme = $derived<ColorScheme>((this.settings.colorScheme as ColorScheme) ?? "system");
  soundEnabled = $derived(this.settings.sound !== "off");
  confirmListDelete = $derived(this.settings.confirmDelete !== "off");
  showCompleted = $derived(this.settings[`showCompleted.${this.view}`] !== "off");
  /** Manual drag-and-drop ordering only makes sense inside a real list. */
  canReorder = $derived(!this.searching && (this.view === "tasks" || this.view.startsWith("list:")));
  /** Tasks in on-screen order, for keyboard navigation. */
  visibleTasks = $derived(this.showCompleted ? [...this.incomplete, ...this.completed] : this.incomplete);

  viewTitle = $derived.by(() => {
    if (this.searching) return t("view.search", { query: this.searchQuery.trim() });
    switch (this.view) {
      case "myDay":
        return t("view.myDay");
      case "important":
        return t("view.important");
      case "planned":
        return t("view.planned");
      case "tasks":
        return t("view.tasks");
      default:
        return this.currentList?.name ?? "";
    }
  });

  private searchTimer: ReturnType<typeof setTimeout> | undefined;
  private toastSeq = 0;

  async init() {
    const [lists, settings] = await Promise.all([api.getLists(), api.getSettings()]);
    this.lists = lists;
    this.settings = settings;
    i18n.preference = settings.language ?? "system";
    const last = settings.lastView as ViewId | undefined;
    if (last && (!last.startsWith("list:") || lists.some((l) => `list:${l.id}` === last))) this.view = last;
    await Promise.all([this.loadTasks(), this.loadCounts()]);
    this.ready = true;

    api.onReminders((due) => this.onReminders(due));
    // Roll My Day over at midnight / when the machine wakes up.
    setInterval(() => {
      const now = todayKey();
      if (now !== this.today) {
        this.today = now;
        this.refresh();
      }
    }, 60_000);
  }

  // ---------------------------------------------------------------- loading

  async loadTasks() {
    const query = this.searchQuery.trim();
    this.tasks = query ? await api.searchTasks(query) : await api.getTasks(this.view);
  }

  async loadCounts() {
    this.counts = await api.getCounts();
  }

  async refresh() {
    await Promise.all([this.loadTasks(), this.loadCounts()]);
  }

  async loadLists() {
    this.lists = await api.getLists();
  }

  // ------------------------------------------------------------- navigation

  setView(view: ViewId) {
    this.searchQuery = "";
    if (this.view !== view) this.selectedId = null;
    this.view = view;
    this.renamingListId = null;
    this.saveSetting("lastView", view);
    this.loadTasks();
  }

  setSearch(query: string) {
    this.searchQuery = query;
    clearTimeout(this.searchTimer);
    this.searchTimer = setTimeout(() => this.loadTasks(), 120);
  }

  select(id: string | null) {
    this.selectedId = id;
  }

  /** Move the selection up/down through visible tasks. */
  moveSelection(delta: number, visible: Task[]) {
    if (!visible.length) return;
    const idx = visible.findIndex((t) => t.id === this.selectedId);
    const next = idx < 0 ? (delta > 0 ? 0 : visible.length - 1) : Math.min(visible.length - 1, Math.max(0, idx + delta));
    this.selectedId = visible[next].id;
  }

  // ------------------------------------------------------------------ tasks

  async addTask(title: string, extra: Partial<NewTask> = {}) {
    const input: NewTask = { title, ...extra };
    // Tasks added from a smart view inherit that view's property.
    if (this.view === "myDay") input.myDay = true;
    if (this.view === "important") input.important = true;
    if (this.view === "planned" && !input.dueDate) input.dueDate = todayKey();
    if (this.view.startsWith("list:")) input.listId = this.view.slice(5);
    const task = await api.createTask(input);
    if (!this.searching) this.tasks = [task, ...this.tasks];
    await this.loadCounts();
    return task;
  }

  private replaceLocal(task: Task) {
    this.tasks = this.tasks.map((t) => (t.id === task.id ? task : t));
  }

  async patchTask(id: string, patch: TaskPatch) {
    // Optimistic update for snappy UI.
    const current = this.tasks.find((t) => t.id === id);
    if (current) {
      const optimistic: Task = { ...current };
      if (patch.title !== undefined) optimistic.title = patch.title;
      if (patch.notes !== undefined) optimistic.notes = patch.notes;
      if (patch.completed !== undefined) optimistic.completed = patch.completed;
      if (patch.important !== undefined) optimistic.important = patch.important;
      if (patch.dueDate !== undefined) optimistic.dueDate = patch.dueDate;
      if (patch.reminderAt !== undefined) optimistic.reminderAt = patch.reminderAt;
      if (patch.myDay !== undefined) optimistic.myDayDate = patch.myDay ? todayKey() : null;
      if (patch.listId !== undefined) optimistic.listId = patch.listId;
      this.replaceLocal(optimistic);
    }
    try {
      const updated = await api.updateTask(id, patch);
      this.replaceLocal(updated);
      if (this.leavesView(updated)) {
        // Let the user see the change briefly, then drop it from the view.
        setTimeout(() => this.loadTasks(), 350);
      }
    } catch (e) {
      this.toast(String(e));
      await this.loadTasks();
    }
    await this.loadCounts();
  }

  private leavesView(task: Task): boolean {
    if (this.searching) return false;
    switch (this.view) {
      case "myDay":
        return task.myDayDate !== todayKey();
      case "important":
        return !task.important;
      case "planned":
        return !task.dueDate && !task.reminderAt;
      case "tasks":
        return task.listId !== this.lists.find((l) => l.isDefault)?.id;
      default:
        return task.listId !== this.view.slice(5);
    }
  }

  async toggleComplete(task: Task) {
    const completed = !task.completed;
    if (completed && this.soundEnabled) playDing();
    await this.patchTask(task.id, { completed });
  }

  toggleImportant(task: Task) {
    return this.patchTask(task.id, { important: !task.important });
  }

  toggleMyDay(task: Task) {
    return this.patchTask(task.id, { myDay: task.myDayDate !== todayKey() });
  }

  async deleteTask(task: Task) {
    this.tasks = this.tasks.filter((t) => t.id !== task.id);
    if (this.selectedId === task.id) this.selectedId = null;
    await api.deleteTask(task.id);
    await this.loadCounts();
    this.toast(t("task.deleted", { title: task.title }), {
      icon: "trash",
      action: {
        label: t("task.undo"),
        run: async () => {
          await api.restoreTask(task);
          await this.refresh();
        },
      },
    });
  }

  async reorderTasks(orderedIncomplete: Task[]) {
    const completed = this.tasks.filter((t) => t.completed);
    this.tasks = [...orderedIncomplete, ...completed];
    await api.reorderTasks(this.tasks.map((t) => t.id));
  }

  // ------------------------------------------------------------------ steps

  private async withTask(p: Promise<Task>) {
    try {
      this.replaceLocal(await p);
    } catch (e) {
      this.toast(String(e));
    }
  }

  addStep(taskId: string, title: string) {
    return this.withTask(api.addStep(taskId, title));
  }
  toggleStep(stepId: string, completed: boolean) {
    return this.withTask(api.updateStep(stepId, undefined, completed));
  }
  renameStep(stepId: string, title: string) {
    return this.withTask(api.updateStep(stepId, title));
  }
  deleteStep(stepId: string) {
    return this.withTask(api.deleteStep(stepId));
  }
  async promoteStep(stepId: string) {
    await this.withTask(api.promoteStep(stepId));
    await this.refresh();
  }

  // ------------------------------------------------------------------ lists

  async createList() {
    const list = await api.createList(t("sidebar.untitledList"), themeForIndex(this.customLists.length));
    this.lists = [...this.lists, list];
    this.setView(`list:${list.id}`);
    this.renamingListId = list.id;
    return list;
  }

  async renameList(id: string, name: string) {
    const trimmed = name.trim();
    this.renamingListId = null;
    if (!trimmed) return;
    const updated = await api.updateList(id, { name: trimmed });
    this.lists = this.lists.map((l) => (l.id === id ? updated : l));
  }

  async setTheme(key: string) {
    if (this.view.startsWith("list:")) {
      const id = this.view.slice(5);
      const updated = await api.updateList(id, { theme: key });
      this.lists = this.lists.map((l) => (l.id === id ? updated : l));
    } else {
      await this.saveSetting(`viewTheme.${this.view}`, key);
    }
  }

  async setListIcon(id: string, icon: string) {
    const updated = await api.updateList(id, { icon });
    this.lists = this.lists.map((l) => (l.id === id ? updated : l));
  }

  async deleteList(list: TaskList) {
    if (this.confirmListDelete) {
      const ok = await this.ask({
        title: t("list.deleteTitle", { name: list.name }),
        body: t("list.deleteBody", { name: list.name }),
        confirmLabel: t("common.delete"),
      });
      if (!ok) return;
    }
    await api.deleteList(list.id);
    this.lists = this.lists.filter((l) => l.id !== list.id);
    if (this.view === `list:${list.id}`) this.setView("tasks");
    await this.loadCounts();
  }

  async reorderLists(ordered: TaskList[]) {
    this.lists = [...this.lists.filter((l) => l.isDefault), ...ordered];
    await api.reorderLists(ordered.map((l) => l.id));
  }

  // --------------------------------------------------------------- settings

  async saveSetting(key: string, value: string) {
    this.settings = { ...this.settings, [key]: value };
    if (key === "language") i18n.preference = value;
    await api.setSetting(key, value);
  }

  toggleShowCompleted() {
    return this.saveSetting(`showCompleted.${this.view}`, this.showCompleted ? "off" : "on");
  }

  // ------------------------------------------------------------- feedback

  toast(message: string, opts: Omit<Toast, "id" | "message"> = {}) {
    const toast: Toast = { id: ++this.toastSeq, message, ...opts };
    this.toasts = [...this.toasts.slice(-2), toast];
    setTimeout(() => this.dismissToast(toast.id), 5000);
  }

  dismissToast(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  ask(req: Omit<ConfirmRequest, "resolve">): Promise<boolean> {
    return new Promise((resolve) => {
      this.confirm = {
        ...req,
        resolve: (ok) => {
          this.confirm = null;
          resolve(ok);
        },
      };
    });
  }

  private onReminders(due: DueReminder[]) {
    for (const r of due) this.toast(`${t("reminder.title")}: ${r.title}`, { icon: "bell" });
    if (this.soundEnabled) playDing();
    this.refresh();
  }
}

export const store = new AppStore();
