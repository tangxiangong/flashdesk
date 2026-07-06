export type ThemeMode = "light" | "dark";
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
  mode = $state<ThemePreference>("system");
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

  toggle() {
    this.set(this.resolvedMode === "dark" ? "light" : "dark");
  }

  set(mode: ThemePreference) {
    this.mode = mode;
    this.resolvedMode = mode === "system" ? systemMode() : mode;
  }

  /** Call once from the root component inside `$effect`. */
  sync() {
    if (typeof document === "undefined") {
      return;
    }

    document.documentElement.dataset.theme = this.resolvedMode;
    window.localStorage.setItem(STORAGE_KEY, this.mode);
  }
}

export const theme = new ThemeState();
