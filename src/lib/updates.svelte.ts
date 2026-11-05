/** In-app updates via tauri-plugin-updater (desktop only). */
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { version as currentVersion } from "../../package.json";
import { inTauri } from "./api";
import { store } from "./store.svelte";

export type UpdateStatus = "idle" | "checking" | "upToDate" | "available" | "downloading" | "installing" | "error";

const DAY = 24 * 60 * 60 * 1000;

class Updates {
  readonly supported = inTauri;
  /** Version of the running build (from the binary in the desktop app). */
  current = $state(currentVersion);
  status = $state<UpdateStatus>("idle");
  /** `inApp`: download & install here. `notify`: package-managed install, link to the release instead. */
  mode = $state<"inApp" | "notify">("inApp");
  available = $state<{ version: string; date?: string; notes: string } | null>(null);
  /** Download progress 0..1, or -1 when the size is unknown. */
  progress = $state(0);
  error = $state("");
  dialogOpen = $state(false);

  private update: Update | null = null;

  async init() {
    if (!inTauri) return;
    this.current = await getVersion().catch(() => currentVersion);
    this.mode = await invoke<"inApp" | "notify">("update_mode").catch(() => "notify" as const);
    setTimeout(() => this.autoCheck(), 5000);
    setInterval(() => this.autoCheck(), DAY);
  }

  get autoEnabled() {
    return store.settings.updateCheck !== "off";
  }

  private autoCheck() {
    if (this.autoEnabled) this.check(true);
  }

  /** `silent`: background check; errors are ignored and skipped versions stay hidden. */
  async check(silent = false) {
    if (!inTauri || this.status === "checking" || this.status === "downloading" || this.status === "installing") return;
    this.status = "checking";
    this.error = "";
    try {
      await this.update?.close().catch(() => {});
      this.update = null;
      const update = await check({ timeout: 20_000 });
      if (!update) {
        this.available = null;
        this.status = silent ? "idle" : "upToDate";
        return;
      }
      if (silent && store.settings.skippedUpdate === update.version) {
        await update.close().catch(() => {});
        this.status = "idle";
        return;
      }
      this.update = update;
      this.available = { version: update.version, date: update.date, notes: update.body ?? "" };
      this.status = "available";
      if (!silent) this.dialogOpen = true;
    } catch (e) {
      // Offline, or no release with update info published yet.
      this.status = silent ? "idle" : "error";
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  async install() {
    if (!this.available) return;
    if (this.mode === "notify" || !this.update) {
      await invoke("open_release_page", { version: this.available.version }).catch((e) => store.toast(String(e)));
      return;
    }
    this.status = "downloading";
    this.progress = 0;
    let total = 0;
    let received = 0;
    try {
      await this.update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? 0;
          this.progress = total ? 0 : -1;
        } else if (event.event === "Progress") {
          received += event.data.chunkLength;
          if (total) this.progress = Math.min(1, received / total);
        } else {
          this.progress = 1;
          this.status = "installing";
        }
      });
      // Windows exits during install; macOS and AppImage need a restart.
      await relaunch();
    } catch (e) {
      this.status = "error";
      this.error = e instanceof Error ? e.message : String(e);
    }
  }

  skip() {
    if (this.available) store.saveSetting("skippedUpdate", this.available.version);
    this.available = null;
    this.status = "idle";
    this.dialogOpen = false;
  }
}

export const updates = new Updates();
