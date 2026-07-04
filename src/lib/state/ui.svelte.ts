export type ViewKey = "flash" | "memory" | "control";

class UiState {
  view = $state<ViewKey>("flash");

  go(view: ViewKey) {
    this.view = view;
  }
}

export const ui = new UiState();
