<script lang="ts">
  import { ChevronsUpDown, Globe, Trash2, Volume2 } from "@lucide/svelte";
  import { LANGUAGES, t } from "../lib/i18n/index.svelte";
  import { SHORTCUTS } from "../lib/shortcuts";
  import { store, type ColorScheme } from "../lib/store.svelte";
  import { getTheme } from "../lib/themes";
  import Modal from "./Modal.svelte";

  const schemes: { value: ColorScheme; label: () => string }[] = [
    { value: "light", label: () => t("settings.theme.light") },
    { value: "dark", label: () => t("settings.theme.dark") },
    { value: "system", label: () => t("settings.theme.system") },
  ];
  const version = "0.1.0";
  const close = () => (store.settingsOpen = false);
</script>

<Modal label={t("settings.title")} width={500} onclose={close}>
  <header>
    <span></span>
    <h2>{t("settings.title")}</h2>
    <button class="done" onclick={close}>{t("settings.done")}</button>
  </header>

  <div class="body">
    <div class="group-label">{t("settings.appearance")}</div>
    <div class="group appearance" role="radiogroup" aria-label={t("settings.theme")}>
      {#each schemes as s (s.value)}
        <button
          class="scheme"
          role="radio"
          aria-checked={store.colorScheme === s.value}
          class:on={store.colorScheme === s.value}
          onclick={() => store.saveSetting("colorScheme", s.value)}
        >
          <span class="thumb {s.value}">
            <span class="win">
              <span class="side"></span>
              <span class="main"><i></i><i></i><i></i></span>
            </span>
          </span>
          <span class="scheme-label">{s.label()}</span>
        </button>
      {/each}
    </div>

    <section class="group">
      <label class="row">
        <span class="glyph" style="background:{getTheme('blue').accent}"><Globe size={16} strokeWidth={2.2} /></span>
        <span class="row-label">{t("settings.language")}</span>
        <span class="select">
          <select value={store.settings.language ?? "system"} onchange={(e) => store.saveSetting("language", e.currentTarget.value)}>
            <option value="system">{t("settings.language.system")}</option>
            {#each LANGUAGES as l (l.code)}
              <option value={l.code}>{l.label}</option>
            {/each}
          </select>
          <ChevronsUpDown size={13} strokeWidth={2.2} />
        </span>
      </label>
    </section>

    <div class="group-label">{t("settings.general")}</div>
    <section class="group">
      <label class="row">
        <span class="glyph" style="background:{getTheme('pink').accent}"><Volume2 size={16} strokeWidth={2.2} /></span>
        <span class="row-label">{t("settings.sound")}</span>
        <span class="switch">
          <input type="checkbox" checked={store.soundEnabled} onchange={(e) => store.saveSetting("sound", e.currentTarget.checked ? "on" : "off")} />
          <span></span>
        </span>
      </label>
      <label class="row">
        <span class="glyph" style="background:{getTheme('red').accent}"><Trash2 size={15} strokeWidth={2.2} /></span>
        <span class="row-label">{t("settings.confirmDelete")}</span>
        <span class="switch">
          <input
            type="checkbox"
            checked={store.confirmListDelete}
            onchange={(e) => store.saveSetting("confirmDelete", e.currentTarget.checked ? "on" : "off")}
          />
          <span></span>
        </span>
      </label>
    </section>

    <div class="group-label">{t("settings.shortcuts")}</div>
    <section class="group">
      {#each SHORTCUTS as s (s.label)}
        <div class="row compact">
          <span class="row-label">{t(s.label)}</span>
          <span class="keys">{#each s.keys as k, i (i)}<kbd>{k}</kbd>{/each}</span>
        </div>
      {/each}
    </section>

    <div class="about">
      <img src="/favicon.svg" alt="" width="40" height="40" />
      <div>{t("settings.aboutBody", { version })}</div>
    </div>
  </div>
</Modal>

<style>
  header {
    position: sticky;
    top: 0;
    z-index: 1;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 14px 16px 10px;
    background: color-mix(in srgb, var(--grouped-bg) 85%, transparent);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
  }
  .done {
    justify-self: end;
    font-weight: 600;
    font-size: 14.5px;
    color: var(--accent);
  }
  .done:hover {
    opacity: 0.75;
  }
  .body {
    padding: 4px 18px 22px;
  }
  .group-label {
    margin: 14px 14px 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
  }
  .group {
    background: var(--cell);
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 12px;
  }
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 46px;
    padding: 8px 14px;
  }
  .row + .row::before {
    content: "";
    position: absolute;
    top: 0;
    left: 54px;
    right: 0;
    height: 1px;
    background: var(--separator);
  }
  .row.compact {
    min-height: 38px;
    padding: 6px 14px;
  }
  .row.compact + .row.compact::before {
    left: 14px;
  }
  .row-label {
    flex: 1;
    font-size: 14px;
  }
  .keys {
    display: flex;
    gap: 3px;
  }
  .select {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--text-2);
  }
  select {
    appearance: none;
    -webkit-appearance: none;
    border: none;
    background: transparent;
    font-size: 14px;
    color: var(--text-2);
    text-align: right;
    outline: none;
  }
  .appearance {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    padding: 16px 14px 12px;
  }
  .scheme {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .thumb {
    display: block;
    width: 100%;
    max-width: 112px;
    aspect-ratio: 16 / 10;
    border-radius: 9px;
    padding: 3px;
    box-shadow: 0 0 0 1px var(--separator);
    transition: box-shadow 0.2s;
  }
  .scheme.on .thumb {
    box-shadow: 0 0 0 3px var(--accent);
  }
  .win {
    display: flex;
    width: 100%;
    height: 100%;
    border-radius: 6px;
    overflow: hidden;
  }
  .side {
    width: 32%;
  }
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 9px 7px;
  }
  .main i {
    height: 5px;
    border-radius: 3px;
  }
  .main i:first-child {
    width: 55%;
    background: var(--accent) !important;
  }
  .light .win {
    background: #fff;
  }
  .light .side {
    background: #ececf0;
  }
  .light .main i {
    background: #dcdce0;
  }
  .dark .win {
    background: #1c1c1e;
  }
  .dark .side {
    background: #2c2c2e;
  }
  .dark .main i {
    background: #3a3a3c;
  }
  .system .win {
    background: linear-gradient(135deg, #fff 50%, #1c1c1e 50%);
  }
  .system .side {
    background: linear-gradient(135deg, #ececf0 50%, #2c2c2e 50%);
  }
  .system .main i {
    background: #8e8e93;
  }
  .scheme-label {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-2);
  }
  .scheme.on .scheme-label {
    color: var(--text);
    font-weight: 600;
  }
  .about {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    margin-top: 18px;
    font-size: 12px;
    color: var(--text-2);
  }
</style>
