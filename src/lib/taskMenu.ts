import {
  Calendar,
  CalendarArrowUp,
  CalendarClock,
  CalendarX,
  CircleCheck,
  Circle,
  FolderInput,
  List,
  Star,
  Sun,
  SunDim,
  Trash2,
} from "@lucide/svelte";
import { addDays, nextWeekKey, todayKey } from "./dates";
import { t } from "./i18n/index.svelte";
import type { MenuEntry } from "./menu.svelte";
import { store } from "./store.svelte";
import type { Task } from "./types";

export function taskMenu(task: Task): MenuEntry[] {
  const inMyDay = task.myDayDate === todayKey();
  const entries: MenuEntry[] = [
    {
      label: inMyDay ? t("task.removeFromMyDay") : t("task.addToMyDay"),
      icon: inMyDay ? SunDim : Sun,
      run: () => store.toggleMyDay(task),
    },
    {
      label: task.important ? t("task.removeImportant") : t("task.markImportant"),
      icon: Star,
      run: () => store.toggleImportant(task),
    },
    {
      label: task.completed ? t("task.markIncomplete") : t("task.markComplete"),
      icon: task.completed ? Circle : CircleCheck,
      run: () => store.toggleComplete(task),
    },
    "divider",
    { label: t("task.dueToday"), icon: Calendar, run: () => store.patchTask(task.id, { dueDate: todayKey() }) },
    {
      label: t("task.dueTomorrow"),
      icon: CalendarArrowUp,
      run: () => store.patchTask(task.id, { dueDate: addDays(todayKey(), 1) }),
    },
    {
      label: t("task.dueNextWeek"),
      icon: CalendarClock,
      run: () => store.patchTask(task.id, { dueDate: nextWeekKey() }),
    },
  ];
  if (task.dueDate)
    entries.push({
      label: t("task.removeDueDate"),
      icon: CalendarX,
      danger: true,
      run: () => store.patchTask(task.id, { dueDate: null }),
    });
  entries.push("divider", {
    label: t("task.moveTo"),
    icon: FolderInput,
    children: store.lists.map((l) => ({
      label: l.isDefault ? t("view.tasks") : l.name,
      icon: List,
      checked: l.id === task.listId,
      run: () => store.patchTask(task.id, { listId: l.id }),
    })),
  });
  entries.push("divider", {
    label: t("task.delete"),
    icon: Trash2,
    danger: true,
    shortcut: "Del",
    run: () => store.deleteTask(task),
  });
  return entries;
}
