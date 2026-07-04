import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export type FirmwareFormat = "elf" | "hex" | "bin";
export type WireProtocol = "swd" | "jtag";
export type JobKind = "flash" | "erase";
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

export interface FirmwareInput {
  path: string;
  format?: FirmwareFormat | null;
  baseAddress?: number | null;
}

export interface ProbeSummary {
  identifier: string;
  vendorId: number;
  productId: number;
  serialNumber?: string | null;
  product?: string | null;
}

export interface TargetSelection {
  chip?: string | null;
  protocol: WireProtocol;
  speedKhz?: number | null;
  connectUnderReset: boolean;
}

export interface FlashOptions {
  verify: boolean;
  dryRun: boolean;
  skipErase: boolean;
  allowEraseAll: boolean;
  resetAfter: boolean;
}

export interface FlashRequest {
  firmware: FirmwareInput;
  probe?: string | null;
  target: TargetSelection;
  options: FlashOptions;
}

export interface MemoryRequest {
  probe?: string | null;
  target: TargetSelection;
  address: number;
  length: number;
}

export interface EraseRange {
  start: number;
  end: number;
}

export interface EraseRequest {
  probe?: string | null;
  target: TargetSelection;
  range?: EraseRange | null;
}

export interface MemoryReadResult {
  address: number;
  length: number;
  dataHex: string;
}

export interface Profile {
  id: string;
  name: string;
  probe?: string | null;
  target: TargetSelection;
  flashOptions: FlashOptions;
  binBaseAddress?: number | null;
  updatedAt: string;
}

export interface JobEvent {
  id: string;
  kind: JobKind;
  stage: JobStage;
  progress?: number | null;
  message: string;
  at: string;
}

function isTauriRuntime(): boolean {
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

export async function listProbes(): Promise<ProbeSummary[]> {
  return invokeCommand("list_probes");
}

export async function searchChips(
  query: string,
  limit = 20,
): Promise<string[]> {
  return invokeCommand("search_chips", { query, limit });
}

export async function flashFirmware(request: FlashRequest): Promise<string> {
  return invokeCommand("flash_firmware", { request });
}

export async function readMemory(
  request: MemoryRequest,
): Promise<MemoryReadResult> {
  return invokeCommand("read_memory", { request });
}

export async function eraseTarget(request: EraseRequest): Promise<string> {
  return invokeCommand("erase_target", { request });
}

export async function loadProfiles(): Promise<Profile[]> {
  return invokeCommand("load_profiles");
}

export async function saveProfiles(profiles: Profile[]): Promise<void> {
  return invokeCommand("save_profiles", { profiles });
}

export function listenToJobEvents(
  handler: (event: JobEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(() => undefined);
  }

  return listen<JobEvent>("job_event", (event) => handler(event.payload));
}
