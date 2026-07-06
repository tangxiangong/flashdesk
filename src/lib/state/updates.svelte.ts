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
  /** 更新窗口是否打开。 */
  open = $state(false);
  /** 当前更新流程状态。 */
  status = $state<UpdateStatus>("idle");
  /** Tauri updater 返回的可用更新对象。 */
  update = $state<Update | null>(null);
  /** 当前已下载字节数。 */
  downloaded = $state(0);
  /** 当前更新包总字节数；未知时为 null。 */
  total = $state<number | null>(null);
  /** 最近一次更新检查、下载或重启失败的错误文案。 */
  error = $state("");

  /** 根据已下载字节数和总字节数计算出的下载百分比。 */
  progressPercent = $derived(
    this.total && this.total > 0
      ? Math.min(100, Math.round((this.downloaded / this.total) * 100))
      : 0,
  );

  /** 打开更新窗口；首次打开或上次失败时会立即检查更新。 */
  openPanel() {
    this.open = true;

    if (this.status === "idle" || this.status === "error") {
      void this.checkForUpdates();
    }
  }

  /** 关闭更新窗口。 */
  close() {
    this.open = false;
  }

  /** 查询发布源并更新当前更新状态。 */
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

  /** 下载可用更新并调用 Tauri updater 安装。 */
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

  /** 重启应用以完成更新切换。 */
  async restart() {
    try {
      await relaunch();
    } catch (error) {
      this.status = "error";
      this.error = readableError(error);
    }
  }
}

/** 全局应用更新状态。 */
export const updates = new UpdatesState();
