<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import JobProgress from "$lib/components/JobProgress.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import folderIcon from "$lib/assets/icons/folder.svg?url";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import checkIcon from "$lib/assets/icons/check.svg?url";
  import {
    flashFirmware,
    type FlashOptions,
    type FlashRequest,
  } from "$lib/api/tauri";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

  type EraseStrategy = "auto" | "full" | "skip";
  type AfterFlash = "reset" | "halt" | "none";

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
  let latest = $derived(jobs.latestFor(jobId));
  let running = $derived(
    jobId != null && !isStageTerminal(latest?.stage ?? "queued"),
  );
  let canSubmit = $derived(
    !running && firmwarePath.trim().length > 0 && target.ready,
  );

  function optionsFromUi(): FlashOptions {
    return {
      verify,
      dryRun,
      skipErase: eraseStrategy === "skip",
      allowEraseAll: eraseStrategy === "full",
      resetAfter: afterFlash === "reset",
      haltAfter: afterFlash === "halt",
    };
  }

  function parseAddress(value: string): number | null {
    const trimmed = value.trim();
    if (!trimmed) return null;
    return Number.parseInt(
      trimmed,
      trimmed.toLowerCase().startsWith("0x") ? 16 : 10,
    );
  }

  function readableError(err: unknown): string {
    if (typeof err === "string") return err;
    if (err && typeof err === "object" && "message" in err) {
      return String((err as { message: unknown }).message);
    }
    return "烧录任务提交失败";
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
      pickerError =
        typeof err === "string"
          ? err
          : "无法打开文件选择器，请确认当前页面运行在 Tauri 应用中";
    }
  }

  async function submit() {
    submitError = null;
    jobId = null;

    const request: FlashRequest = {
      firmware: {
        path: firmwarePath.trim(),
        baseAddress: isBin ? parseAddress(binBaseAddress) : null,
      },
      probe: target.probe,
      target: target.selection(),
      options: optionsFromUi(),
    };

    try {
      jobId = await flashFirmware(request);
    } catch (err) {
      submitError = readableError(err);
    }
  }
</script>

<section class="view" aria-labelledby="flash-title">
  <header class="view-head">
    <div>
      <h1 id="flash-title">固件烧录</h1>
    </div>
  </header>

  <div class="ui-panel section">
    <h2>固件来源</h2>

    <label class="ui-field">
      <span>固件文件路径</span>
      <div class="path-input">
        <button
          type="button"
          class="path-picker"
          aria-label="选择固件文件"
          title="选择固件文件"
          onclick={() => void chooseFirmware()}
        >
          <Icon src={folderIcon} size={16} />
        </button>
        <input
          class="ui-input ui-mono"
          bind:value={firmwarePath}
          placeholder="/path/to/firmware.elf"
          autocomplete="off"
        />
      </div>
    </label>

    {#if pickerError}
      <p class="ui-callout ui-callout--danger">
        <Icon src={alertIcon} size={14} />{pickerError}
      </p>
    {/if}

    {#if isBin}
      <label class="ui-field">
        <span>BIN 基地址</span>
        <input
          class="ui-input ui-mono"
          bind:value={binBaseAddress}
          placeholder="0x08000000"
        />
      </label>
    {/if}
  </div>

  <div class="ui-panel section">
    <h2>烧录策略</h2>

    <div class="ui-grid-2">
      <div class="strategy">
        <span>擦除策略</span>
        <Segmented
          value={eraseStrategy}
          options={[
            { value: "auto" as EraseStrategy, label: "按需擦除" },
            { value: "full" as EraseStrategy, label: "整片擦除" },
            { value: "skip" as EraseStrategy, label: "跳过擦除" },
          ]}
          onchange={(v) => (eraseStrategy = v)}
        />
      </div>

      <div class="strategy">
        <span>完成后</span>
        <Segmented
          value={afterFlash}
          options={[
            { value: "reset" as AfterFlash, label: "复位运行" },
            { value: "halt" as AfterFlash, label: "复位暂停" },
            { value: "none" as AfterFlash, label: "保持连接" },
          ]}
          onchange={(v) => (afterFlash = v)}
        />
      </div>
    </div>

    <div class="ui-switch-row">
      <div class="ui-switch-copy">
        <strong>校验写入</strong>
        <span>写入后回读比对，确保数据一致</span>
      </div>
      <button
        type="button"
        class="ui-switch"
        role="switch"
        aria-checked={verify}
        aria-label="校验写入"
        onclick={() => (verify = !verify)}
      ></button>
    </div>

    <div class="ui-switch-row">
      <div class="ui-switch-copy">
        <strong>仅验证参数</strong>
        <span>不写入 Flash，仅检查固件与目标是否匹配</span>
      </div>
      <button
        type="button"
        class="ui-switch"
        role="switch"
        aria-checked={dryRun}
        aria-label="仅验证参数"
        onclick={() => (dryRun = !dryRun)}
      ></button>
    </div>
  </div>

  <div class="submit-bar">
    {#if !target.ready}
      <p class="hint">
        <Icon src={alertIcon} size={14} />请先在顶部选择目标芯片
      </p>
    {/if}

    <button
      type="button"
      class="ui-btn ui-btn--primary ui-btn--block"
      disabled={!canSubmit}
      onclick={() => void submit()}
    >
      {running ? "烧录中…" : dryRun ? "验证参数" : "开始烧录"}
    </button>

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
</section>

<style>
  .view {
    display: grid;
    gap: var(--space-5);
    max-width: 720px;
  }

  .view-head h1 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-xl);
  }

  .section {
    display: grid;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .section h2 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-base);
    font-weight: 700;
  }

  .path-input {
    position: relative;
    display: flex;
    align-items: center;
    color: var(--color-text-faint);
  }

  .path-picker {
    position: absolute;
    left: 11px;
    display: grid;
    width: 22px;
    height: 22px;
    place-items: center;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-faint);
    cursor: pointer;
    z-index: 1;
  }

  .path-picker:hover {
    color: var(--color-text);
    background: var(--color-surface-muted);
  }

  .path-input input {
    padding-left: 40px;
  }

  .strategy {
    display: grid;
    gap: var(--space-2);
  }

  .strategy > span {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .strategy :global(.segmented) {
    width: 100%;
  }

  .strategy :global(.segment) {
    flex: 1;
  }

  .submit-bar {
    display: grid;
    gap: var(--space-3);
  }

  .hint {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    color: var(--color-warning);
    font-size: var(--text-xs);
  }
</style>
