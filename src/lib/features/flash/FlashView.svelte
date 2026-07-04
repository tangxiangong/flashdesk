<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import JobProgress from "$lib/components/JobProgress.svelte";
  import EraseView from "$lib/features/erase/EraseView.svelte";
  import ConnectionPanel from "$lib/features/target/ConnectionPanel.svelte";
  import MemoryView from "$lib/features/memory/MemoryView.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import folderIcon from "$lib/assets/icons/folder.svg?url";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import checkIcon from "$lib/assets/icons/check.svg?url";
  import {
    flashFirmware,
    readableError,
    type FlashOptions,
    type FlashRequest,
  } from "$lib/api/tauri";
  import { parseAddressInput } from "$lib/utils/address";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

  type EraseStrategy = "auto" | "full" | "skip";
  type AfterFlash = "reset" | "none";

  let firmwarePath = $state("");
  let binBaseAddress = $state("0x08000000");
  let verify = $state(true);
  let dryRun = $state(false);
  let eraseStrategy = $state<EraseStrategy>("auto");
  let afterFlash = $state<AfterFlash>("reset");

  let jobId = $state<string | null>(null);
  let submitError = $state<string | null>(null);
  let pickerError = $state<string | null>(null);

  let isBin = $derived(firmwarePath.trim().toLowerCase().endsWith(".bin"));
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
  function optionsFromUi(): FlashOptions {
    return {
      verify,
      dryRun,
      skipErase: eraseStrategy === "skip",
      allowEraseAll: eraseStrategy === "full",
      resetAfter: afterFlash === "reset",
    };
  }

  async function chooseFirmware() {
    pickerError = null;

    try {
      const selected = await open({
        title: "选择固件",
        multiple: false,
        directory: false,
        filters: [{ name: "Firmware", extensions: ["elf", "hex", "bin"] }],
      });

      if (typeof selected === "string") {
        firmwarePath = selected;
      }
    } catch (err) {
      pickerError = typeof err === "string" ? err : "无法选择文件";
    }
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
</script>

<section class="flash-console" aria-labelledby="flash-title">
  <ConnectionPanel />

  <div class="console-head">
    <h1 id="flash-title">固件</h1>
  </div>

  <div class="firmware-loader">
    <input
      class="path-field ui-mono"
      bind:value={firmwarePath}
      placeholder="选择 ELF / HEX / BIN"
      autocomplete="off"
      aria-label="固件文件路径"
    />

    <button
      type="button"
      class="file-button"
      aria-label="选择固件文件"
      onclick={() => void chooseFirmware()}
    >
      <Icon src={folderIcon} size={16} />
      选择
    </button>
  </div>

  {#if pickerError}
    <p class="ui-callout ui-callout--danger">
      <Icon src={alertIcon} size={14} />{pickerError}
    </p>
  {/if}

  {#if isBin}
    <label class="bin-address">
      <span>BIN 基地址</span>
      <input
        class="ui-input ui-mono"
        bind:value={binBaseAddress}
        placeholder="0x08000000"
      />
    </label>
  {/if}

  {#if isBin && parsedBinBaseAddress == null}
    <p class="ui-callout ui-callout--danger">
      <Icon src={alertIcon} size={14} />BIN 地址无效
    </p>
  {/if}

  <div class="option-deck" aria-label="烧录选项">
    <div class="option-group">
      <span>擦除</span>
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
      <span>烧录后</span>
      <Segmented
        value={afterFlash}
        options={[
          { value: "reset" as AfterFlash, label: "复位" },
          { value: "none" as AfterFlash, label: "不复位" },
        ]}
        onchange={(v) => (afterFlash = v)}
      />
    </div>

    <div class="toggle-group">
      <button
        type="button"
        class="toggle-chip"
        aria-pressed={verify}
        onclick={() => (verify = !verify)}
      >
        校验
      </button>
      <button
        type="button"
        class="toggle-chip"
        aria-pressed={dryRun}
        onclick={() => (dryRun = !dryRun)}
      >
        不写入
      </button>
    </div>
  </div>

  <div class="action-deck">
    <button
      type="button"
      class="flash-button"
      disabled={!canSubmit}
      onclick={() => void submit()}
    >
      {running ? "烧录中" : dryRun ? "检查" : "烧录"}
    </button>

    <EraseView />
  </div>

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

  <MemoryView />
</section>

<style>
  .flash-console {
    display: grid;
    align-content: start;
    gap: 8px;
    width: 100%;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    padding: 10px;
  }

  .console-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .console-head h1 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-lg);
    line-height: 1.1;
  }

  .firmware-loader {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 8px;
    align-items: center;
  }

  .file-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 32px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-muted);
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 800;
    padding: 0 10px;
    transition:
      border-color var(--duration-base) var(--ease-out),
      background var(--duration-base) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  .file-button:hover {
    border-color: var(--color-accent);
  }

  .file-button:active {
    transform: scale(0.995);
  }

  .path-field {
    width: 100%;
    min-height: 32px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-inset);
    color: var(--color-text);
    font: inherit;
    font-size: var(--text-xs);
    padding: 0 10px;
  }

  .bin-address {
    display: grid;
    gap: 6px;
    max-width: 240px;
  }

  .bin-address > span,
  .option-group > span {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    font-weight: 800;
  }

  .option-deck {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
    gap: 8px;
    align-items: end;
    border-block: 1px solid var(--color-border);
    padding: 8px 0;
  }

  .option-group {
    display: grid;
    gap: 6px;
    min-width: 0;
  }

  .option-group :global(.segmented) {
    width: 100%;
  }

  .option-group :global(.segment) {
    flex: 1;
  }

  .toggle-group {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .toggle-chip {
    min-height: 28px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    background: var(--color-surface-inset);
    color: var(--color-text-muted);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-sm);
    font-weight: 800;
    padding: 0 10px;
  }

  .toggle-chip[aria-pressed="true"] {
    border-color: var(--color-accent-border);
    background: var(--color-accent-soft);
    color: var(--color-accent-strong);
  }

  .action-deck {
    display: flex;
    gap: 8px;
    align-items: center;
    justify-content: flex-start;
  }

  .flash-button {
    min-width: 96px;
    min-height: 32px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--color-accent);
    color: var(--color-text-inverse);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-sm);
    font-weight: 900;
    transition:
      background var(--duration-base) var(--ease-out),
      transform var(--duration-fast) var(--ease-out),
      opacity var(--duration-base) var(--ease-out);
  }

  .flash-button:hover:not(:disabled) {
    background: var(--color-accent-strong);
  }

  .flash-button:active:not(:disabled) {
    transform: scale(0.985);
  }

  .flash-button:disabled {
    cursor: not-allowed;
    opacity: 0.42;
  }

  .result-zone {
    display: grid;
    gap: var(--space-2);
  }

  @media (max-width: 720px) {
    .console-head,
    .option-deck,
    .action-deck {
      grid-template-columns: 1fr;
    }

    .console-head {
      display: grid;
    }

    .toggle-group {
      justify-content: flex-start;
      flex-wrap: wrap;
    }

    .file-button {
      grid-template-columns: 1fr;
      min-height: auto;
    }
  }
</style>
