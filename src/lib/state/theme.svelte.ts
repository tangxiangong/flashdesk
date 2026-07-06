/** 已解析出的实际主题模式。 */
export type ThemeMode = "light" | "dark";

/** 用户选择的主题偏好。 */
export type ThemePreference = ThemeMode | "system";

const STORAGE_KEY = "flashdesk:theme";
const SYSTEM_QUERY = "(prefers-color-scheme: light)";

function systemMode(): ThemeMode {
  if (typeof window === "undefined") {
    return "dark";
  }

  return window.matchMedia?.(SYSTEM_QUERY).matches ? "light" : "dark";
}

function readInitial(): ThemePreference {
  if (typeof window === "undefined") {
    return "system";
  }

  const saved = window.localStorage.getItem(STORAGE_KEY);
  if (saved === "light" || saved === "dark" || saved === "system") {
    return saved;
  }

  return "system";
}

class ThemeState {
  /** 用户当前保存的主题偏好。 */
  mode = $state<ThemePreference>("system");
  /** 已解析后的实际主题，用于写入 DOM。 */
  resolvedMode = $state<ThemeMode>("dark");

  private mediaQuery: MediaQueryList | undefined;

  constructor() {
    this.mode = readInitial();
    this.resolvedMode = this.mode === "system" ? systemMode() : this.mode;

    if (typeof window !== "undefined") {
      this.mediaQuery = window.matchMedia?.(SYSTEM_QUERY);
      this.mediaQuery?.addEventListener("change", () => {
        if (this.mode === "system") {
          this.resolvedMode = systemMode();
        }
      });
    }
  }

  /** 在当前实际亮暗主题之间快速切换。 */
  toggle() {
    this.set(this.resolvedMode === "dark" ? "light" : "dark");
  }

  /** 设置主题偏好；system 会继续跟随系统配色变化。 */
  set(mode: ThemePreference) {
    this.mode = mode;
    this.resolvedMode = mode === "system" ? systemMode() : mode;
  }

  /** 从根组件的 `$effect` 中调用，将主题同步到 DOM 和本地存储。 */
  sync() {
    if (typeof document === "undefined") {
      return;
    }

    document.documentElement.dataset.theme = this.resolvedMode;
    window.localStorage.setItem(STORAGE_KEY, this.mode);
  }
}

/** 全局主题状态。 */
export const theme = new ThemeState();
