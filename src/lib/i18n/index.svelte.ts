import { en, type TKey } from "./en";
import { id } from "./id";

export const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "id", label: "Bahasa Indonesia" },
] as const;

export type LangCode = (typeof LANGUAGES)[number]["code"];
const DICTS = { en, id } as const;

function systemLang(): LangCode {
  const nav = typeof navigator !== "undefined" ? navigator.language : "en";
  return nav.toLowerCase().startsWith("id") ? "id" : "en";
}

class I18n {
  /** User preference: a language code or "system". */
  preference = $state<string>("system");
  lang = $derived<LangCode>(
    this.preference === "en" || this.preference === "id" ? this.preference : systemLang(),
  );
  /** BCP 47 locale used for date formatting. */
  locale = $derived(this.lang === "id" ? "id-ID" : "en-US");

  t = (key: TKey, vars?: Record<string, string | number>): string => {
    let s: string = DICTS[this.lang][key] ?? en[key] ?? key;
    if (vars) for (const [k, v] of Object.entries(vars)) s = s.replaceAll(`{${k}}`, String(v));
    return s;
  };
}

export const i18n = new I18n();
export const t = i18n.t;
export type { TKey };
