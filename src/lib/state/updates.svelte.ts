import {
  check,
  type DownloadEvent,
  type Update,
} from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { isTauriRuntime, readableError } from "$lib/api/tauri";

type UpdateStatus =
  | "idle"
  | "checking"
  | "current"
  | "available"
  | "downloading"
  | "ready"
  | "error";

class UpdatesState {
  open = $state(false);
  status = $state<UpdateStatus>("idle");
  update = $state<Update | null>(null);
  downloaded = $state(0);
  total = $state<number | null>(null);
  error = $state("");

  progressPercent = $derived(
    this.total && this.total > 0
      ? Math.min(100, Math.round((this.downloaded / this.total) * 100))
      : 0,
  );

  openPanel() {
    this.open = true;

    if (this.status === "idle" || this.status === "error") {
      void this.checkForUpdates();
    }
  }

  close() {
    this.open = false;
  }

  async checkForUpdates() {
    if (!isTauriRuntime()) {
      this.status = "error";
      this.error = "当前环境不支持更新检查";
      return;
    }

    this.status = "checking";
    this.error = "";
    this.update = null;
    this.downloaded = 0;
    this.total = null;

    try {
      const next = await check();
      this.update = next;
      this.status = next ? "available" : "current";
    } catch (error) {
      this.status = "error";
      this.error = readableError(error);
    }
  }

  async downloadAndInstall() {
    if (!this.update) {
      return;
    }

    this.status = "downloading";
    this.error = "";
    this.downloaded = 0;
    this.total = null;

    try {
      await this.update.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === "Started") {
          this.downloaded = 0;
          this.total = event.data.contentLength ?? null;
        } else if (event.event === "Progress") {
          this.downloaded += event.data.chunkLength;
        } else if (event.event === "Finished") {
          if (this.total == null) {
            this.total = this.downloaded;
          }
        }
      });

      this.status = "ready";
    } catch (error) {
      this.status = "error";
      this.error = readableError(error);
    }
  }

  async restart() {
    try {
      await relaunch();
    } catch (error) {
      this.status = "error";
      this.error = readableError(error);
    }
  }
}

export const updates = new UpdatesState();
