/**
 * Tiny, safe Markdown renderer for release notes (headings, bullet lists,
 * bold/italic/code, links). Everything is HTML-escaped first.
 */
const escape = (s: string) =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

function inline(s: string): string {
  return escape(s)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/(^|[^*])\*([^*\s][^*]*)\*/g, "$1<em>$2</em>")
    .replace(/\[([^\]]+)\]\((https:\/\/[^\s)]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
}

interface Item {
  text: string;
  children: string[];
}

/** Renders release notes, dropping everything from the "## Download" section on. */
export function renderNotes(md: string): string {
  const cut = md.search(/^## Download/m);
  const lines = (cut >= 0 ? md.slice(0, cut) : md).split(/\r?\n/);
  const out: string[] = [];
  let list: Item[] | null = null;
  let para: string[] = [];

  const flushPara = () => {
    if (para.length) out.push(`<p>${inline(para.join(" "))}</p>`);
    para = [];
  };
  const flushList = () => {
    if (list) {
      const items = list.map((it) => {
        const sub = it.children.length ? `<ul>${it.children.map((c) => `<li>${inline(c)}</li>`).join("")}</ul>` : "";
        return `<li>${inline(it.text)}${sub}</li>`;
      });
      out.push(`<ul>${items.join("")}</ul>`);
    }
    list = null;
  };

  for (const raw of lines) {
    const line = raw.trimEnd();
    const heading = line.match(/^#{1,6}\s+(.*)$/);
    const item = line.match(/^(\s*)[-*]\s+(.*)$/);
    if (heading) {
      flushPara();
      flushList();
      out.push(`<h4>${inline(heading[1])}</h4>`);
    } else if (item) {
      flushPara();
      const nested = item[1].length >= 2 && list && list.length > 0;
      if (nested) list![list!.length - 1].children.push(item[2]);
      else (list ??= []).push({ text: item[2], children: [] });
    } else if (!line.trim()) {
      flushPara();
      flushList();
    } else if (list && /^\s+/.test(line)) {
      // Continuation of the previous (possibly nested) item.
      const last = list[list.length - 1];
      if (last.children.length) last.children[last.children.length - 1] += ` ${line.trim()}`;
      else last.text += ` ${line.trim()}`;
    } else {
      flushList();
      para.push(line.trim());
    }
  }
  flushPara();
  flushList();
  return out.join("");
}
