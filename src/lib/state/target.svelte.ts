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
  /** 当前选中的调试探针标识；为 null 时由后端自动选择。 */
  probe = $state<string | null>(null);
  /** 用户手动指定的目标芯片名；为空时允许后端自动识别。 */
  chip = $state("");
  /** 最近一次连接后由 probe-rs 识别出的芯片名。 */
  detectedChip = $state("");
  /** 当前连接使用的线协议。 */
  protocol = $state<WireProtocol>("swd");
  /** 当前连接使用的调试时钟频率，单位为 kHz。 */
  speedKhz = $state(4000);
  /** 是否在复位保持状态下连接目标芯片。 */
  connectUnderReset = $state(false);

  /** 最近一次枚举到的可用探针列表。 */
  probes = $state<ProbeSummary[]>([]);
  /** 探针列表是否正在刷新。 */
  probesLoading = $state(false);
  /** 探针刷新失败时展示给用户的错误信息。 */
  probesError = $state<string | null>(null);

  /** 当前是否正在建立目标连接。 */
  connecting = $state(false);
  /** 最近一次连接失败时展示给用户的错误信息。 */
  connectError = $state<string | null>(null);
  /** 当前已建立连接的目标信息。 */
  connection = $state<ConnectionInfo | null>(null);
  /** 已连接目标对应的选择快照，用于判断当前 UI 选择是否仍然匹配连接。 */
  connectedSelectionKey = $state("");

  /** 芯片搜索框中的当前查询文本。 */
  chipQuery = $state("");
  /** 当前芯片搜索结果。 */
  chipResults = $state<string[]>([]);
  /** 芯片搜索请求是否仍在进行。 */
  chipSearching = $state(false);

  /** 当前探针标识对应的探针摘要。 */
  selectedProbeSummary = $derived(
    this.probes.find((p) => p.identifier === this.probe) ?? null,
  );

  /** 实际用于展示和高级功能请求的芯片名。 */
  effectiveChip = $derived(this.chip.trim() || this.detectedChip);

  /** 当前 UI 选择生成的连接键。 */
  connectionKey = $derived(
    selectionKey(
      this.probe ?? "auto",
      this.chip.trim(),
      this.protocol,
      this.speedKhz,
      this.connectUnderReset,
    ),
  );
  /** 当前连接信息是否仍然匹配页面上的探针、芯片和连接参数。 */
  connected = $derived(
    this.connection != null &&
      this.connectionKey === this.connectedSelectionKey,
  );
  /** 当前目标连接是否可用于烧录、擦除和内存读取。 */
  ready = $derived(this.connected);

  /** 生成传给后端的目标选择对象。 */
  selection() {
    return {
      chip: this.chip.trim() || null,
      protocol: this.protocol,
      speedKhz: this.speedKhz,
      connectUnderReset: this.connectUnderReset,
    };
  }

  /** 按当前探针和目标选择建立 probe-rs 会话。 */
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

  /** 清空当前连接状态，但保留用户的探针和芯片选择。 */
  disconnect() {
    this.connection = null;
    this.detectedChip = "";
    this.connectedSelectionKey = "";
    this.connectError = null;
  }

  /** 重新枚举本机可用调试探针，并处理自动选择和失效选择。 */
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

  /** 按关键字搜索 probe-rs 内置芯片注册表。 */
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

  /** 选择一个芯片名并使现有连接失效。 */
  pickChip(chip: string) {
    this.chip = chip;
    this.chipResults = [];
    this.chipQuery = "";
    this.disconnect();
  }

  /** 选择一个探针，并根据探针名称对常见 JTAG 设备做协议预选。 */
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

/** 全局目标连接和芯片选择状态。 */
export const target = new TargetState();
