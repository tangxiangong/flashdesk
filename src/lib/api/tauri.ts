import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

/** 支持的固件文件格式。 */
export type FirmwareFormat = "elf" | "hex" | "bin";

/** 调试探针线协议。 */
export type WireProtocol = "swd" | "jtag";

/** 原生应用菜单可触发的前端动作。 */
export type AppMenuAction = "about" | "check-update";

/** 后台任务类型。 */
export type JobKind = "flash" | "erase";

/** 后台任务阶段。 */
export type JobStage =
  | "queued"
  | "connecting"
  | "preparing"
  | "erasing"
  | "programming"
  | "verifying"
  | "resetting"
  | "completed"
  | "failed";

/** 前端提交给后端的固件文件信息。 */
export interface FirmwareInput {
  /** 固件文件的本地路径。 */
  path: string;
  /** 用户显式选择的格式；为空时由后端根据扩展名推断。 */
  format?: FirmwareFormat | null;
  /** BIN 固件的烧录基地址。 */
  baseAddress?: number | null;
}

/** 固件文件中会被写入 Flash 的地址段。 */
export interface FirmwareUsageSegment {
  /** 起始写入地址。 */
  start: number;
  /** 结束地址，开区间。 */
  end: number;
  /** 该连续地址段的字节数。 */
  size: number;
}

/** 固件 Flash 占用分析结果。 */
export interface FirmwareUsage {
  /** 解析采用的固件格式。 */
  format: FirmwareFormat;
  /** 去重合并后的实际写入字节数。 */
  usedBytes: number;
  /** 从最小写入地址到最大写入地址的覆盖跨度，包含空洞。 */
  spanBytes: number;
  /** 最小写入地址。 */
  startAddress?: number | null;
  /** 最大写入地址，开区间。 */
  endAddress?: number | null;
  /** 合并后的连续写入地址段。 */
  segments: FirmwareUsageSegment[];
}

/** 可用调试探针的摘要信息。 */
export interface ProbeSummary {
  /** probe-rs 可解析的探针选择器字符串。 */
  identifier: string;
  /** USB vendor id。 */
  vendorId: number;
  /** USB product id。 */
  productId: number;
  /** 探针序列号。 */
  serialNumber?: string | null;
  /** 探针产品名。 */
  product?: string | null;
}

/** 一次目标连接或目标操作所需的选择信息。 */
export interface TargetSelection {
  /** 目标芯片型号；为空时尝试自动识别。 */
  chip?: string | null;
  /** 调试线协议。 */
  protocol: WireProtocol;
  /** 连接速度，单位 kHz。 */
  speedKhz?: number | null;
  /** 是否使用 connect-under-reset。 */
  connectUnderReset: boolean;
}

/** 固件烧录行为选项。 */
export interface FlashOptions {
  /** 写入后逐字节校验。 */
  verify: boolean;
  /** 只执行预检查，不修改芯片内容。 */
  dryRun: boolean;
  /** 跳过自动擦除。 */
  skipErase: boolean;
  /** 允许整片擦除。 */
  allowEraseAll: boolean;
  /** 写入完成后复位目标。 */
  resetAfter: boolean;
}

/** 前端提交的一次烧录任务请求。 */
export interface FlashRequest {
  /** 待烧录固件。 */
  firmware: FirmwareInput;
  /** 选中的探针；为空时要求当前只连接一个探针。 */
  probe?: string | null;
  /** 目标芯片和连接参数。 */
  target: TargetSelection;
  /** 烧录选项。 */
  options: FlashOptions;
}

/** 前端提交的一次整片擦除请求。 */
export interface EraseRequest {
  /** 选中的探针；为空时要求当前只连接一个探针。 */
  probe?: string | null;
  /** 目标芯片和连接参数。 */
  target: TargetSelection;
}

/** 前端提交的一次内存读取请求。 */
export interface MemoryRequest {
  /** 选中的探针；为空时要求当前只连接一个探针。 */
  probe?: string | null;
  /** 目标芯片和连接参数。 */
  target: TargetSelection;
  /** 起始地址。 */
  address: number;
  /** 读取长度，单位字节。 */
  length: number;
}

/** 内存读取结果。 */
export interface MemoryReadResult {
  /** 实际读取起始地址。 */
  address: number;
  /** 实际读取长度，单位字节。 */
  length: number;
  /** 小写十六进制连续字节字符串。 */
  dataHex: string;
}

export interface TargetCandidate {
  /** probe-rs 可接收的目标名称。 */
  name: string;
  /** 候选所属芯片族。 */
  family: string;
}

interface AppErrorResponse {
  message?: unknown;
  detail?: unknown;
  recovery?: unknown;
  targetCandidates?: unknown;
}

/** 前端提交的一次连接探测请求。 */
export interface ConnectRequest {
  /** 选中的探针；为空时要求当前只连接一个探针。 */
  probe?: string | null;
  /** 目标芯片和连接参数。 */
  target: TargetSelection;
}

/** 成功连接目标后的摘要信息。 */
export interface ConnectionInfo {
  /** 实际使用的探针选择器字符串。 */
  probe: string;
  /** 实际连接的芯片型号。 */
  chip: string;
  /** 实际使用的线协议。 */
  protocol: WireProtocol;
  /** 实际连接速度，单位 kHz。 */
  speedKhz?: number | null;
  /** 是否使用 connect-under-reset。 */
  connectUnderReset: boolean;
}

/** 目标芯片内存区域类型。 */
export type MemoryRegionKind = "nvm" | "ram" | "generic";

/** 内存区域访问权限。 */
export interface MemoryAccessInfo {
  /** 是否可读。 */
  read: boolean;
  /** 是否可写。 */
  write: boolean;
  /** 是否可执行。 */
  execute: boolean;
  /** 是否可作为启动区域。 */
  boot: boolean;
}

/** 目标芯片的内存区域布局。 */
export interface MemoryRegionLayout {
  /** 区域名称。 */
  name?: string | null;
  /** 区域类型。 */
  kind: MemoryRegionKind;
  /** 起始地址。 */
  start: number;
  /** 结束地址。 */
  end: number;
  /** 区域大小，单位字节。 */
  size: number;
  /** 关联的核心名称。 */
  cores: string[];
  /** 是否是其他区域的别名。 */
  isAlias: boolean;
  /** 访问权限。 */
  access: MemoryAccessInfo;
}

/** 用户保存的烧录配置。 */
export interface Profile {
  /** 配置标识。 */
  id: string;
  /** 用户可见配置名称。 */
  name: string;
  /** 保存的探针选择器。 */
  probe?: string | null;
  /** 保存的目标选择。 */
  target: TargetSelection;
  /** 保存的烧录选项。 */
  flashOptions: FlashOptions;
  /** 保存的 BIN 基地址。 */
  binBaseAddress?: number | null;
  /** 最近更新时间。 */
  updatedAt: string;
}

/** 后台任务推送给前端的进度事件。 */
export interface JobEvent {
  /** 任务标识。 */
  id: string;
  /** 任务类型。 */
  kind: JobKind;
  /** 当前任务阶段。 */
  stage: JobStage;
  /** 任务进度，范围 0 到 1。 */
  progress?: number | null;
  /** 前端可直接展示的阶段消息。 */
  message: string;
  /** 事件产生时间。 */
  at: string;
}

/** 判断当前页面是否运行在 Tauri WebView 中。 */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && window.__TAURI_INTERNALS__ != null;
}

function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauriRuntime()) {
    return Promise.reject(new Error("当前页面不在 Tauri 运行环境中"));
  }

  return invoke<T>(command, args);
}

/** 列出当前系统可见的调试探针。 */
export async function listProbes(): Promise<ProbeSummary[]> {
  return invokeCommand("list_probes");
}

/** 连接目标芯片并返回实际连接信息。 */
export async function connectTarget(
  request: ConnectRequest,
): Promise<ConnectionInfo> {
  return invokeCommand("connect_target", { request });
}

/** 根据关键字搜索 probe-rs 内置芯片型号。 */
export async function searchChips(
  query: string,
  limit = 20,
): Promise<string[]> {
  return invokeCommand("search_chips", { query, limit });
}

/** 读取指定芯片型号的内存布局。 */
export async function targetMemoryMap(
  chip: string,
): Promise<MemoryRegionLayout[]> {
  return invokeCommand("target_memory_map", { chip });
}

/** 启动固件烧录任务并返回任务 ID。 */
export async function flashFirmware(request: FlashRequest): Promise<string> {
  return invokeCommand("flash_firmware", { request });
}

/** 分析固件会写入的 Flash 字节数和地址范围。 */
export async function firmwareUsage(
  input: FirmwareInput,
): Promise<FirmwareUsage> {
  return invokeCommand("firmware_usage", { input });
}

/** 同步读取目标内存并返回十六进制字符串。 */
export async function readMemory(
  request: MemoryRequest,
): Promise<MemoryReadResult> {
  return invokeCommand("read_memory", { request });
}

/** 启动整片擦除任务并返回任务 ID。 */
export async function eraseTarget(request: EraseRequest): Promise<string> {
  return invokeCommand("erase_target", { request });
}

/** 将未知错误对象转换为前端可展示的字符串。 */
export function readableError(err: unknown, fallback = "操作失败"): string {
  if (typeof err === "string") return err;
  if (!err || typeof err !== "object") return fallback;

  const response = err as AppErrorResponse;
  const message =
    typeof response.message === "string" && response.message.trim()
      ? response.message.trim()
      : null;
  const detail =
    typeof response.detail === "string" && response.detail.trim()
      ? response.detail.trim()
      : null;

  if (message && detail) return `${message}：${detail}`;
  if (message) return message;
  if (detail) return detail;

  if ("toString" in response && typeof response.toString === "function") {
    const rendered = response.toString();
    if (rendered && rendered !== "[object Object]") return rendered;
  }

  return fallback;
}

/** 从后端错误响应中提取自动识别候选目标。 */
export function targetCandidatesFromError(err: unknown): TargetCandidate[] {
  if (!err || typeof err !== "object") return [];

  const candidates = (err as AppErrorResponse).targetCandidates;
  if (!Array.isArray(candidates)) return [];

  return candidates.filter((candidate): candidate is TargetCandidate => {
    return (
      candidate != null &&
      typeof candidate === "object" &&
      typeof (candidate as TargetCandidate).name === "string" &&
      typeof (candidate as TargetCandidate).family === "string"
    );
  });
}

/** 从应用数据目录加载用户保存的烧录配置。 */
export async function loadProfiles(): Promise<Profile[]> {
  return invokeCommand("load_profiles");
}

/** 将用户保存的烧录配置写入应用数据目录。 */
export async function saveProfiles(profiles: Profile[]): Promise<void> {
  return invokeCommand("save_profiles", { profiles });
}

/** 监听后端后台任务进度事件。 */
export function listenToJobEvents(
  handler: (event: JobEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(() => undefined);
  }

  return listen<JobEvent>("job_event", (event) => handler(event.payload));
}

/** 监听原生应用菜单动作并映射为前端动作枚举。 */
export async function listenToAppMenuEvents(
  handler: (action: AppMenuAction) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return () => undefined;
  }

  const unlistenAbout = await listen("flashdesk://menu/about", () =>
    handler("about"),
  );
  const unlistenCheckUpdate = await listen(
    "flashdesk://menu/check-update",
    () => handler("check-update"),
  );

  return () => {
    void unlistenAbout();
    void unlistenCheckUpdate();
  };
}
