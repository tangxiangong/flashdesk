<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import folderIcon from "$lib/assets/icons/folder.svg?url";
  import flashIcon from "$lib/assets/icons/flash.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";
  import {
    firmwareUsage,
    flashFirmware,
    isTauriRuntime,
    readableError,
    targetMemoryMap,
    type FirmwareUsage,
    type FlashOptions,
    type FlashRequest,
    type MemoryRegionLayout,
  } from "$lib/api/tauri";
  import { parseAddressInput } from "$lib/utils/address";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";
  import { appStatus } from "$lib/state/status.svelte";

  type EraseStrategy = "auto" | "full" | "skip";
  type AfterFlash = "reset" | "none";

  const FIRMWARE_EXTENSIONS = ["elf", "hex", "bin"];

  let firmwarePath = $state("");
  let binBaseAddress = $state("0x08000000");
  let verify = $state(true);
  let dryRun = $state(false);
  let eraseStrategy = $state<EraseStrategy>("auto");
  let afterFlash = $state<AfterFlash>("reset");
  let dragActive = $state(false);
  let usageLoading = $state(false);
  let usage = $state<FirmwareUsage | null>(null);
  let usageError = $state<string | null>(null);
  let memoryRegions = $state<MemoryRegionLayout[]>([]);
  let memoryLayoutChip = $state("");

  let jobId = $state<string | null>(null);

  let extension = $derived(fileExtension(firmwarePath));
  let fileName = $derived(baseName(firmwarePath));
  let isBin = $derived(extension === "bin");
  let parsedBinBaseAddress = $derived(parseAddressInput(binBaseAddress));
  let latest = $derived(jobs.latestFor(jobId));
  let running = $derived(
    jobId != null && !isStageTerminal(latest?.stage ?? "queued"),
  );
  let canSubmit = $derived(
    !running &&
      firmwarePath.trim().length > 0 &&
      target.ready &&
      (!isBin || parsedBinBaseAddress != null),
  );
  let flashTotalBytes = $derived(
    memoryRegions
      .filter((region) => region.kind === "nvm" && !region.isAlias)
      .reduce((sum, region) => sum + region.size, 0),
  );
  let usagePercent = $derived(
    usage && flashTotalBytes > 0
      ? Math.min(999.9, (usage.usedBytes / flashTotalBytes) * 100)
      : null,
  );

  function fileExtension(path: string): string {
    const trimmed = path.trim().toLowerCase();
    const dot = trimmed.lastIndexOf(".");
    return dot === -1 ? "" : trimmed.slice(dot + 1);
  }

  function baseName(path: string): string {
    const trimmed = path.trim();
    const parts = trimmed.split(/[\\/]/);
    return parts.at(-1) ?? trimmed;
  }

  function optionsFromUi(): FlashOptions {
    return {
      verify,
      dryRun,
      skipErase: eraseStrategy === "skip",
      allowEraseAll: eraseStrategy === "full",
      resetAfter: afterFlash === "reset",
    };
  }

  function sizeLabel(bytes: number): string {
    if (!Number.isFinite(bytes) || bytes <= 0) return "--";
    if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(bytes % (1024 * 1024) === 0 ? 0 : 1)} MiB`;
    }
    if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(bytes % 1024 === 0 ? 0 : 1)} KiB`;
    }
    return `${bytes} B`;
  }

  function addressLabel(address: number | null | undefined): string {
    if (address == null) return "--";
    return `0x${Math.trunc(address).toString(16).toUpperCase().padStart(8, "0")}`;
  }

  function lastAddressLabel(endAddress: number | null | undefined): string {
    if (endAddress == null) return "--";
    return addressLabel(Math.max(0, endAddress - 1));
  }

  function percentLabel(value: number | null): string {
    if (value == null) return "--";
    return `${value.toFixed(value >= 10 ? 1 : 2)}%`;
  }

  function acceptPath(path: string) {
    appStatus.clear();
    firmwarePath = path;
  }

  async function chooseFirmware() {
    appStatus.clear();

    try {
      const selected = await open({
        title: "选择固件",
        multiple: false,
        directory: false,
        filters: [{ name: "Firmware", extensions: FIRMWARE_EXTENSIONS }],
      });

      if (typeof selected === "string") {
        acceptPath(selected);
      }
    } catch (err) {
      appStatus.danger(
        "固件选择",
        typeof err === "string" ? err : "无法选择文件",
      );
    }
  }

  function clearFirmware() {
    firmwarePath = "";
    appStatus.clear();
  }

  async function submit() {
    appStatus.clear();
    jobId = null;

    if (isBin && parsedBinBaseAddress == null) {
      appStatus.danger("固件地址", "BIN 地址无效");
      return;
    }

    const request: FlashRequest = {
      firmware: {
        path: firmwarePath.trim(),
        baseAddress: isBin ? parsedBinBaseAddress : null,
      },
      probe: target.probe,
      target: target.selection(),
      options: optionsFromUi(),
    };

    try {
      jobId = await flashFirmware(request);
    } catch (err) {
      appStatus.danger("烧录失败", readableError(err, "烧录失败"));
    }
  }

  $effect(() => {
    if (isBin && parsedBinBaseAddress == null) {
      appStatus.danger("固件地址", "BIN 地址格式无效");
    } else if (appStatus.current?.label === "固件地址") {
      appStatus.clear();
    }
  });

  $effect(() => {
    if (!isTauriRuntime()) return;

    const chip = target.connected ? target.effectiveChip.trim() : "";
    if (!chip) {
      memoryRegions = [];
      memoryLayoutChip = "";
      return;
    }

    if (chip === memoryLayoutChip) return;

    let cancelled = false;
    void (async () => {
      try {
        const regions = await targetMemoryMap(chip);
        if (!cancelled) {
          memoryRegions = regions;
          memoryLayoutChip = chip;
        }
      } catch {
        if (!cancelled) {
          memoryRegions = [];
          memoryLayoutChip = chip;
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (!isTauriRuntime()) return;

    const path = firmwarePath.trim();
    const baseAddress = isBin ? parsedBinBaseAddress : null;
    if (!path || (isBin && baseAddress == null)) {
      usage = null;
      usageError = null;
      usageLoading = false;
      return;
    }

    let cancelled = false;
    usageLoading = true;
    usageError = null;

    void (async () => {
      try {
        const nextUsage = await firmwareUsage({
          path,
          baseAddress,
        });
        if (!cancelled) {
          usage = nextUsage;
        }
      } catch (err) {
        if (!cancelled) {
          usage = null;
          usageError = readableError(err, "无法分析 Flash 占用");
        }
      } finally {
        if (!cancelled) {
          usageLoading = false;
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (!isTauriRuntime()) return;

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void (async () => {
      const { getCurrentWebview } = await import("@tauri-apps/api/webview");
      const stop = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          dragActive = true;
        } else if (event.payload.type === "leave") {
          dragActive = false;
        } else if (event.payload.type === "drop") {
          dragActive = false;
          const match = event.payload.paths.find((path) =>
            FIRMWARE_EXTENSIONS.includes(fileExtension(path)),
          );
          if (match) {
            acceptPath(match);
          } else if (event.payload.paths.length > 0) {
            appStatus.danger("固件类型", "仅支持 ELF / HEX / BIN 固件文件");
          }
        }
      });
      if (cancelled) {
        stop();
      } else {
        unlisten = stop;
      }
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  });
</script>

<section class="flash-panel ui-panel" aria-labelledby="flash-title">
  <div class="ui-panel-pad panel-body">
    <h2 id="flash-title">固件烧录</h2>

    <div class="dropzone-wrap">
      <button
        type="button"
        class="dropzone"
        class:has-file={firmwarePath.length > 0}
        class:drag-active={dragActive}
        title={firmwarePath ? "点击重新选择固件" : undefined}
        onclick={() => void chooseFirmware()}
      >
        {#if firmwarePath}
          <span class="dz-icon dz-icon--file"
            ><Icon src={flashIcon} size={17} /></span
          >
          <span class="dz-copy">
            <strong>{fileName}</strong>
            <span class="ui-mono">{firmwarePath}</span>
          </span>
          {#if extension}
            <span class="ui-badge ui-badge--accent"
              >{extension.toUpperCase()}</span
            >
          {/if}
        {:else}
          <span class="dz-icon"><Icon src={folderIcon} size={20} /></span>
          <strong>点击选择固件文件</strong>
          <span>或将文件拖放到此处 · 支持 ELF / HEX / BIN</span>
        {/if}
      </button>

      {#if firmwarePath}
        <button
          type="button"
          class="dz-clear"
          aria-label="清除已选固件"
          title="清除"
          onclick={clearFirmware}
        >
          <Icon src={xIcon} size={13} />
        </button>
      {/if}
    </div>

    {#if isBin}
      <label class="bin-address ui-field">
        <span>BIN 基地址</span>
        <input
          class="ui-input ui-mono"
          bind:value={binBaseAddress}
          placeholder="0x08000000"
        />
      </label>
    {/if}

    {#if firmwarePath}
      <div class="usage-strip" aria-live="polite">
        <span class="ui-label">Flash 占用</span>
        {#if usageLoading}
          <strong>分析中…</strong>
          <span>正在读取固件写入范围</span>
        {:else if usage}
          <strong>
            {sizeLabel(usage.usedBytes)}
            {#if usagePercent != null}
              <em>{percentLabel(usagePercent)}</em>
            {/if}
          </strong>
          <span class="ui-mono">
            {addressLabel(usage.startAddress)} - {lastAddressLabel(usage.endAddress)}
            {#if usage.segments.length > 1}
              · {usage.segments.length} 段
            {/if}
          </span>
          {#if usagePercent == null}
            <span>连接目标后显示占比</span>
          {/if}
        {:else if usageError}
          <strong>无法分析</strong>
          <span>{usageError}</span>
        {:else}
          <strong>--</strong>
          <span>选择有效固件后显示</span>
        {/if}
      </div>
    {/if}

    <div class="options-block">
      <span class="ui-label">烧录选项</span>

      <div class="options-grid">
        <div class="option-group">
          <span>擦除策略</span>
          <Segmented
            value={eraseStrategy}
            options={[
              { value: "auto" as EraseStrategy, label: "自动" },
              { value: "full" as EraseStrategy, label: "全片" },
              { value: "skip" as EraseStrategy, label: "跳过" },
            ]}
            onchange={(v) => (eraseStrategy = v)}
          />
        </div>

        <div class="option-group">
          <span>完成后</span>
          <Segmented
            value={afterFlash}
            options={[
              { value: "reset" as AfterFlash, label: "复位运行" },
              { value: "none" as AfterFlash, label: "保持暂停" },
            ]}
            onchange={(v) => (afterFlash = v)}
          />
        </div>
      </div>

      <div class="switch-list">
        <div class="ui-switch-row">
          <span class="ui-switch-copy">
            <strong>写入后校验</strong>
            <span>逐字节比对确保烧录内容正确</span>
          </span>
          <button
            type="button"
            class="ui-switch"
            role="switch"
            aria-checked={verify}
            aria-label="写入后校验"
            onclick={() => (verify = !verify)}
          ></button>
        </div>

        <div class="ui-switch-row">
          <span class="ui-switch-copy">
            <strong>仅检查，不写入</strong>
            <span>用于预演，不会修改芯片内容</span>
          </span>
          <button
            type="button"
            class="ui-switch"
            role="switch"
            aria-checked={dryRun}
            aria-label="仅检查，不写入"
            onclick={() => (dryRun = !dryRun)}
          ></button>
        </div>
      </div>
    </div>

    <button
      type="button"
      class="ui-btn ui-btn--primary ui-btn--block cta-button"
      disabled={!canSubmit}
      onclick={() => void submit()}
    >
      <Icon src={flashIcon} size={16} />
      {running ? "烧录中…" : dryRun ? "开始检查" : "开始烧录"}
    </button>

    {#if !target.ready}
      <p class="cta-hint">先在上方连接设备，才能开始烧录</p>
    {/if}
  </div>
</section>

<style>
  .flash-panel {
    width: 100%;
  }

  .panel-body {
    display: grid;
    gap: var(--space-4);
  }

  .panel-body h2 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-lg);
    font-weight: 800;
    line-height: 1.2;
  }

  .dropzone-wrap {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .dropzone {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    flex: 1;
    gap: var(--space-3);
    min-width: 0;
    min-height: 84px;
    border: 1.5px dashed var(--color-border-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-inset);
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    padding: var(--space-4);
    text-align: left;
    transition:
      border-color var(--duration-base) var(--ease-out),
      background var(--duration-base) var(--ease-out);
  }

  .dropzone:not(.has-file) {
    grid-template-columns: 1fr;
    justify-items: center;
    align-content: center;
    justify-content: center;
    min-height: 168px;
    gap: 8px;
    text-align: center;
  }

  .dropzone:not(.has-file) strong {
    color: var(--color-text);
    font-size: var(--text-base);
    font-weight: 700;
  }

  .dropzone:not(.has-file) span {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
  }

  .dropzone:hover,
  .dropzone.drag-active {
    border-color: var(--color-accent);
    background: var(--color-accent-soft);
  }

  .dropzone.has-file {
    border-style: solid;
    border-color: var(--color-border);
    cursor: default;
  }

  .dz-icon {
    display: grid;
    flex-shrink: 0;
    width: 40px;
    height: 40px;
    place-items: center;
    border-radius: var(--radius-sm);
    background: var(--color-surface-muted);
    color: var(--color-text-muted);
  }

  .dropzone:not(.has-file) .dz-icon {
    width: 52px;
    height: 52px;
    border-radius: var(--radius-md);
  }

  .dz-icon--file {
    background: var(--color-accent);
    color: var(--color-text-inverse);
  }

  .dz-copy {
    display: grid;
    gap: 2px;
    min-width: 0;
    overflow: hidden;
  }

  .dz-copy strong {
    overflow: hidden;
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dz-copy span {
    overflow: hidden;
    color: var(--color-text-faint);
    font-size: var(--text-2xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dz-clear {
    display: grid;
    flex-shrink: 0;
    width: 38px;
    height: 38px;
    place-items: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    color: var(--color-text-faint);
    cursor: pointer;
  }

  .dz-clear:hover {
    border-color: var(--color-danger-border);
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .bin-address {
    max-width: 260px;
  }

  .usage-strip {
    display: grid;
    grid-template-columns: minmax(88px, auto) minmax(120px, auto) minmax(0, 1fr);
    align-items: center;
    column-gap: var(--space-3);
    row-gap: 4px;
    border-top: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-3) 0;
  }

  .usage-strip strong {
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 800;
    white-space: nowrap;
  }

  .usage-strip em {
    color: var(--color-accent);
    font-style: normal;
    font-size: var(--text-xs);
    font-weight: 800;
  }

  .usage-strip span:not(.ui-label) {
    min-width: 0;
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .options-block {
    display: grid;
    gap: var(--space-3);
    border-top: 1px solid var(--color-border);
    padding-top: var(--space-4);
  }

  .options-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .option-group {
    display: grid;
    gap: 6px;
    min-width: 0;
  }

  .option-group > span {
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 700;
  }

  .option-group :global(.segmented) {
    width: 100%;
  }

  .option-group :global(.segment) {
    flex: 1;
  }

  .switch-list {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--space-6);
    row-gap: var(--space-2);
    border-top: 1px solid var(--color-border);
    padding-top: var(--space-3);
  }

  .switch-list :global(.ui-switch-row) {
    justify-content: flex-start;
    gap: var(--space-3);
    min-width: 0;
  }

  .switch-list :global(.ui-switch-copy) {
    flex: 0 1 auto;
  }

  .cta-button {
    margin-top: var(--space-1);
  }

  .cta-hint {
    margin: -6px 0 0;
    color: var(--color-text-faint);
    font-size: var(--text-xs);
    text-align: center;
  }

  @media (max-width: 560px) {
    .options-grid {
      grid-template-columns: 1fr;
    }

    .usage-strip {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .switch-list {
      grid-template-columns: 1fr;
    }
  }
</style>
