import { diffDays, isoToDateKey, parseDateKey, todayKey } from "./dates";
import { i18n, t } from "./i18n/index.svelte";

/** "Today", "Tomorrow", "Yesterday" or e.g. "Mon, Nov 10". */
export function relativeDay(key: string): string {
  const diff = diffDays(key, todayKey());
  if (diff === 0) return t("date.today");
  if (diff === 1) return t("date.tomorrow");
  if (diff === -1) return t("date.yesterday");
  const d = parseDateKey(key);
  const sameYear = d.getFullYear() === new Date().getFullYear();
  return d.toLocaleDateString(i18n.locale, {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: sameYear ? undefined : "numeric",
  });
}

export function longDate(d = new Date()): string {
  return d.toLocaleDateString(i18n.locale, { weekday: "long", month: "long", day: "numeric" });
}

export function time(iso: string): string {
  return new Date(iso).toLocaleTimeString(i18n.locale, { hour: "numeric", minute: "2-digit" });
}

/** "Today, 9:00 AM" style label for a reminder. */
export function reminderLabel(iso: string): string {
  return `${relativeDay(isoToDateKey(iso))}, ${time(iso)}`;
}

export function dateFromIso(iso: string): string {
  return relativeDay(isoToDateKey(iso));
}
