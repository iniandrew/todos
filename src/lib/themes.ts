/** Theme presets that can be applied to any list or smart view. */
export interface ListTheme {
  key: string;
  /** Accent used for checkboxes, buttons, selected states. */
  accent: string;
  /** CSS background of the main pane. */
  background: string;
}

const g = (a: string, b: string, angle = 135) => `linear-gradient(${angle}deg, ${a} 0%, ${b} 100%)`;

export const THEMES: ListTheme[] = [
  { key: "blue", accent: "#2f6fed", background: g("#3a7bfd", "#5b4bdb") },
  { key: "purple", accent: "#7c4ddb", background: g("#8e5cf2", "#c04fd8") },
  { key: "rose", accent: "#d6336c", background: g("#f05a7e", "#c2185b") },
  { key: "red", accent: "#d64545", background: g("#f0625a", "#b8322e") },
  { key: "orange", accent: "#e8590c", background: g("#ff922b", "#e8590c") },
  { key: "amber", accent: "#c27c0e", background: g("#fcc419", "#f08c00") },
  { key: "green", accent: "#2f9e44", background: g("#51cf66", "#2b8a3e") },
  { key: "teal", accent: "#0c8599", background: g("#20c997", "#1098ad") },
  { key: "slate", accent: "#5c6b7f", background: g("#748ba3", "#3d4b5c") },
  {
    key: "sunrise",
    accent: "#e8590c",
    background:
      "radial-gradient(circle at 80% 20%, rgba(255,225,140,.9) 0%, rgba(255,225,140,0) 35%), linear-gradient(160deg, #ff9a76 0%, #f76b8a 45%, #7b5cd6 100%)",
  },
  {
    key: "ocean",
    accent: "#1971c2",
    background:
      "radial-gradient(circle at 15% 85%, rgba(102,217,232,.65) 0%, rgba(102,217,232,0) 40%), linear-gradient(160deg, #1c7ed6 0%, #1864ab 50%, #0b2f5b 100%)",
  },
  {
    key: "forest",
    accent: "#2b8a3e",
    background:
      "radial-gradient(circle at 85% 15%, rgba(216,245,162,.55) 0%, rgba(216,245,162,0) 40%), linear-gradient(160deg, #40c057 0%, #2b8a3e 50%, #143d22 100%)",
  },
  {
    key: "aurora",
    accent: "#0ca678",
    background:
      "radial-gradient(ellipse at 30% 10%, rgba(99,230,190,.8) 0%, rgba(99,230,190,0) 45%), radial-gradient(ellipse at 80% 40%, rgba(177,151,252,.7) 0%, rgba(177,151,252,0) 45%), linear-gradient(180deg, #0b1d3a 0%, #13294b 100%)",
  },
  {
    key: "night",
    accent: "#5c7cfa",
    background:
      "radial-gradient(circle at 75% 20%, rgba(186,200,255,.35) 0%, rgba(186,200,255,0) 25%), linear-gradient(180deg, #1b1f3b 0%, #2d1b4e 100%)",
  },
];

export const DEFAULT_VIEW_THEMES: Record<string, string> = {
  myDay: "sunrise",
  important: "rose",
  planned: "teal",
  tasks: "blue",
  search: "slate",
};

export function getTheme(key: string | undefined | null): ListTheme {
  return THEMES.find((t) => t.key === key) ?? THEMES[0];
}

/** Rotating default theme for newly created lists. */
export function themeForIndex(i: number): string {
  const pool = ["purple", "green", "orange", "teal", "rose", "ocean", "amber", "forest"];
  return pool[i % pool.length];
}
