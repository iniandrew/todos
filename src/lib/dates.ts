/** Date helpers. All "dates" are local calendar dates in `YYYY-MM-DD`. */

const pad = (n: number) => String(n).padStart(2, "0");

export function toDateKey(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function todayKey(): string {
  return toDateKey(new Date());
}

export function parseDateKey(key: string): Date {
  const [y, m, d] = key.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export function addDays(key: string, days: number): string {
  const d = parseDateKey(key);
  d.setDate(d.getDate() + days);
  return toDateKey(d);
}

/** Next Monday (or the Monday after if today is Monday). */
export function nextWeekKey(from = todayKey()): string {
  const d = parseDateKey(from);
  const delta = ((8 - d.getDay()) % 7) || 7;
  return addDays(from, delta);
}

export function diffDays(a: string, b: string): number {
  return Math.round((parseDateKey(a).getTime() - parseDateKey(b).getTime()) / 86_400_000);
}

/** Converts an RFC 3339 timestamp into the local date key. */
export function isoToDateKey(iso: string): string {
  return toDateKey(new Date(iso));
}

/** `<input type="datetime-local">` value -> ISO string (UTC). */
export function localInputToIso(value: string): string {
  return new Date(value).toISOString();
}

export function isoToLocalInput(iso: string): string {
  const d = new Date(iso);
  return `${toDateKey(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** A reminder at `hour:00` on the given local date. */
export function reminderAt(dateKey: string, hour: number, minute = 0): string {
  const d = parseDateKey(dateKey);
  d.setHours(hour, minute, 0, 0);
  return d.toISOString();
}

/** "Later today": three hours from now, rounded to the hour. */
export function laterToday(): string {
  const d = new Date();
  d.setHours(d.getHours() + 3, 0, 0, 0);
  return d.toISOString();
}

export type DueState = "overdue" | "today" | "future";

export function dueState(dueDate: string, today = todayKey()): DueState {
  const diff = diffDays(dueDate, today);
  if (diff < 0) return "overdue";
  if (diff === 0) return "today";
  return "future";
}
