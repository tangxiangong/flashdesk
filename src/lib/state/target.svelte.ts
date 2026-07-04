import {
  connectTarget,
  listProbes,
  readableError,
  searchChips,
  type ConnectionInfo,
  type ProbeSummary,
  type WireProtocol,
} from "$lib/api/tauri";

function prefersJtag(probe: ProbeSummary | null | undefined): boolean {
  if (!probe) return false;

  return [probe.product, probe.identifier]
    .filter((value): value is string => typeof value === "string")
    .some((value) => {
      const normalized = value.toLowerCase();
      return normalized.includes("jtag") || normalized.includes("esp");
    });
}

class TargetState {
  probe = $state<string | null>(null);
  chip = $state("");
  detectedChip = $state("");
  protocol = $state<WireProtocol>("swd");
  speedKhz = $state(4000);
  connectUnderReset = $state(false);

  probes = $state<ProbeSummary[]>([]);
  probesLoading = $state(false);
  probesError = $state<string | null>(null);

  connecting = $state(false);
  connectError = $state<string | null>(null);
  connection = $state<ConnectionInfo | null>(null);
  connectedSelectionKey = $state("");

  chipQuery = $state("");
  chipResults = $state<string[]>([]);
  chipSearching = $state(false);

  selectedProbeSummary = $derived(
    this.probes.find((p) => p.identifier === this.probe) ?? null,
  );

  effectiveChip = $derived(this.chip.trim() || this.detectedChip);

  connectionKey = $derived(
    selectionKey(
      this.probe ?? "auto",
      this.chip.trim(),
      this.protocol,
      this.speedKhz,
      this.connectUnderReset,
    ),
  );
  connected = $derived(
    this.connection != null &&
      this.connectionKey === this.connectedSelectionKey,
  );
  ready = $derived(this.connected);

  selection() {
    return {
      chip: this.chip.trim() || null,
      protocol: this.protocol,
      speedKhz: this.speedKhz,
      connectUnderReset: this.connectUnderReset,
    };
  }

  async connect() {
    this.connecting = true;
    this.connectError = null;
    const requestedTarget = this.selection();

    try {
      this.connection = await connectTarget({
        probe: this.probe,
        target: requestedTarget,
      });
      this.probe = this.connection.probe;
      this.detectedChip = this.connection.chip;
      this.connectedSelectionKey = selectionKey(
        this.connection.probe,
        requestedTarget.chip ?? "",
        this.connection.protocol,
        this.connection.speedKhz ?? null,
        this.connection.connectUnderReset,
      );
      this.chipResults = [];
      this.chipQuery = "";
    } catch (err) {
      this.connection = null;
      this.detectedChip = "";
      this.connectedSelectionKey = "";
      this.connectError = readableError(err);
    } finally {
      this.connecting = false;
    }
  }

  disconnect() {
    this.connection = null;
    this.detectedChip = "";
    this.connectedSelectionKey = "";
    this.connectError = null;
  }

  async refreshProbes() {
    this.probesLoading = true;
    this.probesError = null;

    try {
      this.probes = await listProbes();
      if (this.probes.length === 1) {
        this.pickProbe(this.probes[0].identifier);
      } else if (
        this.probe &&
        !this.probes.some((p) => p.identifier === this.probe)
      ) {
        this.probe = null;
        this.disconnect();
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
    this.disconnect();
  }

  pickProbe(identifier: string | null) {
    this.probe = identifier;
    const selectedProbe = this.probes.find((p) => p.identifier === identifier);
    if (prefersJtag(selectedProbe)) {
      this.protocol = "jtag";
    }
    this.disconnect();
  }
}

function selectionKey(
  probe: string,
  chip: string,
  protocol: WireProtocol,
  speedKhz: number | null | undefined,
  connectUnderReset: boolean,
): string {
  return [
    probe,
    chip.trim() || "auto",
    protocol,
    speedKhz ?? "",
    connectUnderReset ? "reset" : "normal",
  ].join("|");
}

export const target = new TargetState();
