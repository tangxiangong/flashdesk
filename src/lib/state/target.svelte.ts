import {
  attachTarget,
  listProbes,
  searchChips,
  type ProbeSummary,
  type TargetStatus,
  type WireProtocol,
} from "$lib/api/tauri";

export type LinkCheck =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "ok"; status: TargetStatus }
  | { state: "error"; message: string };

function readableError(err: unknown): string {
  if (typeof err === "string") return err;
  if (err && typeof err === "object" && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return "操作失败";
}

class TargetState {
  probe = $state<string | null>(null);
  chip = $state("");
  protocol = $state<WireProtocol>("swd");
  speedKhz = $state(4000);
  connectUnderReset = $state(false);

  probes = $state<ProbeSummary[]>([]);
  probesLoading = $state(false);
  probesError = $state<string | null>(null);

  chipQuery = $state("");
  chipResults = $state<string[]>([]);
  chipSearching = $state(false);

  link = $state<LinkCheck>({ state: "idle" });

  selectedProbeSummary = $derived(
    this.probes.find((p) => p.identifier === this.probe) ?? null,
  );

  ready = $derived(this.chip.trim().length > 0);

  selection() {
    return {
      chip: this.chip.trim() || null,
      protocol: this.protocol,
      speedKhz: this.speedKhz,
      connectUnderReset: this.connectUnderReset,
    };
  }

  async refreshProbes() {
    this.probesLoading = true;
    this.probesError = null;

    try {
      this.probes = await listProbes();
      if (this.probes.length === 1) {
        this.probe = this.probes[0].identifier;
      } else if (
        this.probe &&
        !this.probes.some((p) => p.identifier === this.probe)
      ) {
        this.probe = null;
      }
    } catch (err) {
      this.probesError = readableError(err);
    } finally {
      this.probesLoading = false;
    }
  }

  async searchChip(query: string) {
    this.chipQuery = query;
    if (!query.trim()) {
      this.chipResults = [];
      return;
    }

    this.chipSearching = true;
    try {
      this.chipResults = await searchChips(query.trim(), 10);
    } catch {
      this.chipResults = [];
    } finally {
      this.chipSearching = false;
    }
  }

  pickChip(chip: string) {
    this.chip = chip;
    this.chipResults = [];
    this.chipQuery = "";
    this.link = { state: "idle" };
  }

  pickProbe(identifier: string | null) {
    this.probe = identifier;
    this.link = { state: "idle" };
  }

  async testLink() {
    if (!this.ready) return;
    this.link = { state: "checking" };

    try {
      const status = await attachTarget({
        probe: this.probe,
        target: this.selection(),
        haltAfterReset: false,
      });
      this.link = { state: "ok", status };
    } catch (err) {
      this.link = { state: "error", message: readableError(err) };
    }
  }
}

export const target = new TargetState();
