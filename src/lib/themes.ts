/** Accent colors that can be applied to any list or smart view (Apple-style system colors). */
export interface ListTheme {
  key: string;
  accent: string;
}

export const THEMES: ListTheme[] = [
  { key: "red", accent: "#ff3b30" },
  { key: "orange", accent: "#ff9500" },
  { key: "yellow", accent: "#f5b800" },
  { key: "green", accent: "#30b350" },
  { key: "mint", accent: "#00b8b0" },
  { key: "teal", accent: "#30a7c2" },
  { key: "blue", accent: "#0a84ff" },
  { key: "indigo", accent: "#5e5ce6" },
  { key: "purple", accent: "#af52de" },
  { key: "pink", accent: "#ff2d55" },
  { key: "brown", accent: "#a2845e" },
  { key: "gray", accent: "#8e8e93" },
];

/** Keys used by earlier versions of the app. */
const LEGACY: Record<string, string> = {
  rose: "pink",
  amber: "yellow",
  slate: "gray",
  sunrise: "orange",
  ocean: "blue",
  forest: "green",
  aurora: "mint",
  night: "indigo",
  cyan: "teal",
};

export const DEFAULT_VIEW_THEMES: Record<string, string> = {
  myDay: "orange",
  important: "pink",
  planned: "red",
  tasks: "blue",
  search: "gray",
};

export function getTheme(key: string | undefined | null): ListTheme {
  const k = key ? (LEGACY[key] ?? key) : "blue";
  return THEMES.find((t) => t.key === k) ?? THEMES[6];
}

/** Rotating default color for newly created lists. */
export function themeForIndex(i: number): string {
  const pool = ["blue", "orange", "green", "purple", "pink", "teal", "indigo", "yellow", "mint", "red"];
  return pool[i % pool.length];
}
