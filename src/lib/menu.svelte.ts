import type { Component } from "svelte";

export interface MenuItem {
  label: string;
  icon?: Component<any>;
  danger?: boolean;
  shortcut?: string;
  checked?: boolean;
  /** Items shown in an inline sub-section. */
  children?: MenuItem[];
  run?: () => void;
}
export type MenuEntry = MenuItem | "divider";

class MenuState {
  current = $state<{ x: number; y: number; items: MenuEntry[] } | null>(null);

  open(e: MouseEvent | { x: number; y: number }, items: MenuEntry[]) {
    if ("preventDefault" in e) e.preventDefault();
    const x = "clientX" in e ? e.clientX : e.x;
    const y = "clientY" in e ? e.clientY : e.y;
    this.current = { x, y, items };
  }

  /** Open below an anchor element (for "…" buttons). */
  openAt(anchor: HTMLElement, items: MenuEntry[], align: "left" | "right" = "left") {
    const r = anchor.getBoundingClientRect();
    this.current = { x: align === "left" ? r.left : r.right - 240, y: r.bottom + 6, items };
  }

  close() {
    this.current = null;
  }
}

export const menu = new MenuState();
