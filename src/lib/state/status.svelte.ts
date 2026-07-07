export type AppStatusTone = "info" | "progress" | "success" | "danger";

export interface AppStatusNotice {
  tone: AppStatusTone;
  label: string;
  message: string;
  progress: number | null;
  pulse: boolean;
}

class AppStatusState {
  current = $state<AppStatusNotice | null>(null);

  set(notice: AppStatusNotice) {
    this.current = notice;
  }

  info(label: string, message: string) {
    this.set({ tone: "info", label, message, progress: null, pulse: false });
  }

  progress(label: string, message: string) {
    this.set({ tone: "progress", label, message, progress: null, pulse: true });
  }

  success(label: string, message: string) {
    this.set({ tone: "success", label, message, progress: 100, pulse: false });
  }

  danger(label: string, message: string) {
    this.set({ tone: "danger", label, message, progress: null, pulse: false });
  }

  clear() {
    this.current = null;
  }
}

export const appStatus = new AppStatusState();
