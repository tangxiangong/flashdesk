<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import JobProgress from "$lib/components/JobProgress.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import folderIcon from "$lib/assets/icons/folder.svg?url";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import checkIcon from "$lib/assets/icons/check.svg?url";
  import flashIcon from "$lib/assets/icons/flash.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";
  import {
    flashFirmware,
    isTauriRuntime,
    readableError,
    type FlashOptions,
    type FlashRequest,
  } from "$lib/api/tauri";
  import { parseAddressInput } from "$lib/utils/address";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

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

  let jobId = $state<string | null>(null);
  let submitError = $state<string | null>(null);
  let pickerError = $state<string | null>(null);

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

  function acceptPath(path: string) {
    pickerError = null;
    firmwarePath = path;
  }

  async function chooseFirmware() {
    pickerError = null;

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
      pickerError = typeof err === "string" ? err : "无法选择文件";
    }
  }

  function clearFirmware() {
    firmwarePath = "";
    pickerError = null;
  }

  async function submit() {
    submitError = null;
    jobId = null;

    if (isBin && parsedBinBaseAddress == null) {
      submitError = "BIN 地址无效";
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
      submitError = readableError(err, "烧录失败");
    }
  }

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
            pickerError = "仅支持 ELF / HEX / BIN 固件文件";
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

    {#if pickerError}
      <p class="ui-callout ui-callout--danger">
        <Icon src={alertIcon} size={14} />{pickerError}
      </p>
    {/if}

    {#if isBin}
      <label class="bin-address ui-field">
        <span>BIN 基地址</span>
        <input
          class="ui-input ui-mono"
          bind:value={binBaseAddress}
          placeholder="0x08000000"
        />
        {#if parsedBinBaseAddress == null}
          <span class="field-error">
            <Icon src={alertIcon} size={12} />地址格式无效
          </span>
        {/if}
      </label>
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

    <div class="result-zone">
      <JobProgress {jobId} />

      {#if submitError}
        <p class="ui-callout ui-callout--danger">
          <Icon src={alertIcon} size={14} />{submitError}
        </p>
      {/if}

      {#if latest?.stage === "completed"}
        <p class="ui-callout ui-callout--success">
          <Icon src={checkIcon} size={14} />{latest.message}
        </p>
      {/if}
    </div>
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

  .field-error {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--color-danger);
    font-size: var(--text-2xs);
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

  .result-zone {
    display: grid;
    gap: var(--space-2);
  }

  @media (max-width: 560px) {
    .options-grid {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .switch-list {
      grid-template-columns: 1fr;
    }
  }
</style>
