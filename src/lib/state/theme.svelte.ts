export type ThemeMode = "light" | "dark";

const STORAGE_KEY = "flashdesk:theme";

function readInitial(): ThemeMode {
  if (typeof window === "undefined") {
    return "dark";
  }

  const saved = window.localStorage.getItem(STORAGE_KEY);
  if (saved === "light" || saved === "dark") {
    return saved;
  }

  return window.matchMedia?.("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

class ThemeState {
  mode = $state<ThemeMode>("dark");

  constructor() {
    this.mode = readInitial();
  }

  toggle() {
    this.mode = this.mode === "dark" ? "light" : "dark";
  }

  set(mode: ThemeMode) {
    this.mode = mode;
  }

  /** Call once from the root component inside `$effect`. */
  sync() {
    if (typeof document === "undefined") {
      return;
    }

    document.documentElement.dataset.theme = this.mode;
    window.localStorage.setItem(STORAGE_KEY, this.mode);
  }
}

export const theme = new ThemeState();
