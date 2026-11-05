export interface TaskList {
  id: string;
  name: string;
  icon: string | null;
  theme: string;
  sortOrder: number;
  isDefault: boolean;
  createdAt: string;
}

export interface Step {
  id: string;
  taskId: string;
  title: string;
  completed: boolean;
  sortOrder: number;
}

export interface Task {
  id: string;
  listId: string;
  title: string;
  notes: string;
  completed: boolean;
  completedAt: string | null;
  important: boolean;
  /** Local date `YYYY-MM-DD`. */
  dueDate: string | null;
  /** RFC 3339 timestamp. */
  reminderAt: string | null;
  /** Local date the task was added to My Day. */
  myDayDate: string | null;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
  steps: Step[];
}

export interface TaskPatch {
  title?: string;
  notes?: string;
  completed?: boolean;
  important?: boolean;
  listId?: string;
  myDay?: boolean;
  dueDate?: string | null;
  reminderAt?: string | null;
}

export interface NewTask {
  listId?: string | null;
  title: string;
  important?: boolean;
  myDay?: boolean;
  dueDate?: string | null;
}

export interface ListPatch {
  name?: string;
  theme?: string;
  icon?: string | null;
}

export interface Counts {
  myDay: number;
  important: number;
  planned: number;
  tasks: number;
  lists: Record<string, number>;
}

export interface DueReminder {
  taskId: string;
  title: string;
  listName: string;
}

export type SmartView = "myDay" | "important" | "planned" | "tasks";
/** A smart view or `list:<id>`. */
export type ViewId = SmartView | `list:${string}`;
