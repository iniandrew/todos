<script lang="ts">
  import { Monitor, Moon, Sun, X } from "@lucide/svelte";
  import { LANGUAGES, t } from "../lib/i18n/index.svelte";
  import { SHORTCUTS } from "../lib/shortcuts";
  import { store, type ColorScheme } from "../lib/store.svelte";
  import Modal from "./Modal.svelte";

  const schemes: { value: ColorScheme; icon: typeof Sun; label: () => string }[] = [
    { value: "system", icon: Monitor, label: () => t("settings.theme.system") },
    { value: "light", icon: Sun, label: () => t("settings.theme.light") },
    { value: "dark", icon: Moon, label: () => t("settings.theme.dark") },
  ];
  const version = "0.1.0";
</script>

<Modal label={t("settings.title")} width={520} onclose={() => (store.settingsOpen = false)}>
  <header>
    <h2>{t("settings.title")}</h2>
    <button class="icon-btn" aria-label={t("detail.close")} onclick={() => (store.settingsOpen = false)}><X size={18} /></button>
  </header>

  <div class="body">
    <h3>{t("settings.appearance")}</h3>
    <div class="field">
      <span class="field-label">{t("settings.theme")}</span>
      <div class="segmented" role="radiogroup" aria-label={t("settings.theme")}>
        {#each schemes as s (s.value)}
          <button
            role="radio"
            aria-checked={store.colorScheme === s.value}
            class:on={store.colorScheme === s.value}
            onclick={() => store.saveSetting("colorScheme", s.value)}
          >
            <s.icon size={15} />
            {s.label()}
          </button>
        {/each}
      </div>
    </div>
    <div class="field">
      <label class="field-label" for="lang">{t("settings.language")}</label>
      <select id="lang" value={store.settings.language ?? "system"} onchange={(e) => store.saveSetting("language", e.currentTarget.value)}>
        <option value="system">{t("settings.language.system")}</option>
        {#each LANGUAGES as l (l.code)}
          <option value={l.code}>{l.label}</option>
        {/each}
      </select>
    </div>

    <h3>{t("settings.general")}</h3>
    <label class="toggle">
      <span>{t("settings.sound")}</span>
      <input
        type="checkbox"
        checked={store.soundEnabled}
        onchange={(e) => store.saveSetting("sound", e.currentTarget.checked ? "on" : "off")}
      />
      <span class="switch"></span>
    </label>
    <label class="toggle">
      <span>{t("settings.confirmDelete")}</span>
      <input
        type="checkbox"
        checked={store.confirmListDelete}
        onchange={(e) => store.saveSetting("confirmDelete", e.currentTarget.checked ? "on" : "off")}
      />
      <span class="switch"></span>
    </label>

    <h3>{t("settings.shortcuts")}</h3>
    <dl class="shortcuts">
      {#each SHORTCUTS as s (s.label)}
        <dt>{t(s.label)}</dt>
        <dd>
          {#each s.keys as k, i (i)}<kbd>{k}</kbd>{/each}
        </dd>
      {/each}
    </dl>

    <h3>{t("settings.about")}</h3>
    <p class="about">{t("settings.aboutBody", { version })}</p>
  </div>
</Modal>

<style>
  header {
    position: sticky;
    top: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 8px 22px;
    background: var(--surface);
    z-index: 1;
  }
  h2 {
    margin: 0;
    font-size: 18px;
  }
  .body {
    padding: 0 22px 22px;
  }
  h3 {
    margin: 18px 0 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-2);
  }
  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 44px;
  }
  .field-label {
    font-weight: 500;
  }
  .segmented {
    display: inline-flex;
    padding: 3px;
    border-radius: 9px;
    background: var(--surface-2);
    border: 1px solid var(--border);
  }
  .segmented button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    border-radius: 7px;
    color: var(--text-2);
    font-size: 13px;
  }
  .segmented button.on {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow-sm);
    font-weight: 600;
  }
  select {
    height: 34px;
    padding: 0 10px;
    border-radius: 8px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    min-width: 200px;
  }
  .toggle {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 44px;
    cursor: pointer;
  }
  .toggle input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  .switch {
    position: relative;
    width: 40px;
    height: 22px;
    border-radius: 11px;
    background: var(--border-strong);
    transition: background 0.2s;
    flex: none;
  }
  .switch::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    transition: transform 0.2s var(--ease);
  }
  .toggle input:checked + .switch {
    background: var(--accent);
  }
  .toggle input:checked + .switch::after {
    transform: translateX(18px);
  }
  .toggle input:focus-visible + .switch {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .shortcuts {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px 16px;
    margin: 0;
  }
  dt {
    color: var(--text);
  }
  dd {
    margin: 0;
    display: flex;
    gap: 4px;
    justify-content: flex-end;
  }
  .about {
    margin: 0;
    color: var(--text-2);
  }
</style>
