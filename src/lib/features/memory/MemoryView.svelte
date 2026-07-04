<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import JobProgress from "$lib/components/JobProgress.svelte";
  import folderIcon from "$lib/assets/icons/folder.svg?url";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import downloadIcon from "$lib/assets/icons/download.svg?url";
  import eyeIcon from "$lib/assets/icons/eye.svg?url";
  import HexViewer from "./HexViewer.svelte";
  import {
    readMemory,
    writeMemory,
    dumpMemory,
    type MemoryReadResult,
  } from "$lib/api/tauri";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

  type Tab = "rw" | "dump";
  let tab = $state<Tab>("rw");

  let address = $state("0x20000000");
  let length = $state(64);
  let writeHex = $state("00 01 02 03");
  let dumpPath = $state("");

  let reading = $state(false);
  let readResult = $state<MemoryReadResult | null>(null);
  let readError = $state<string | null>(null);

  let writeJobId = $state<string | null>(null);
  let dumpJobId = $state<string | null>(null);
  let actionError = $state<string | null>(null);

  let writeLatest = $derived(jobs.latestFor(writeJobId));
  let dumpLatest = $derived(jobs.latestFor(dumpJobId));
  let writeRunning = $derived(
    writeJobId != null && !isStageTerminal(writeLatest?.stage ?? "queued"),
  );
  let dumpRunning = $derived(
    dumpJobId != null && !isStageTerminal(dumpLatest?.stage ?? "queued"),
  );

  function parseAddress(value: string): number {
    const trimmed = value.trim();
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
    return "操作失败";
  }

  async function doRead() {
    reading = true;
    readError = null;
    readResult = null;

    try {
      readResult = await readMemory({
        probe: target.probe,
        target: target.selection(),
        address: parseAddress(address),
        length,
      });
    } catch (err) {
      readError = readableError(err);
    } finally {
      reading = false;
    }
  }

  async function doWrite() {
    actionError = null;
    writeJobId = null;

    try {
      writeJobId = await writeMemory({
        probe: target.probe,
        target: target.selection(),
        address: parseAddress(address),
        dataHex: writeHex,
      });
    } catch (err) {
      actionError = readableError(err);
    }
  }

  async function doDump() {
    actionError = null;
    dumpJobId = null;

    try {
      dumpJobId = await dumpMemory({
        probe: target.probe,
        target: target.selection(),
        address: parseAddress(address),
        length,
        outputPath: dumpPath.trim(),
      });
    } catch (err) {
      actionError = readableError(err);
    }
  }
</script>

<section class="view" aria-labelledby="memory-title">
  <header class="view-head">
    <div>
      <h1 id="memory-title">内存工具</h1>
    </div>

    <div class="ui-tabs" role="tablist">
      <button
        type="button"
        class="ui-tab"
        role="tab"
        aria-selected={tab === "rw"}
        onclick={() => (tab = "rw")}>读写</button
      >
      <button
        type="button"
        class="ui-tab"
        role="tab"
        aria-selected={tab === "dump"}
        onclick={() => (tab = "dump")}>导出</button
      >
    </div>
  </header>

  {#if !target.ready}
    <p class="ui-callout ui-callout--warning">
      <Icon src={alertIcon} size={14} />请先在顶部选择目标芯片
    </p>
  {/if}

  <div class="ui-panel section">
    <h2>地址范围</h2>
    <div class="ui-grid-2">
      <label class="ui-field">
        <span>起始地址</span>
        <input
          class="ui-input ui-mono"
          bind:value={address}
          autocomplete="off"
        />
      </label>
      <label class="ui-field">
        <span>长度（字节）</span>
        <input
          class="ui-input ui-mono"
          type="number"
          min="1"
          max="1048576"
          bind:value={length}
        />
      </label>
    </div>
  </div>

  {#if tab === "rw"}
    <div class="ui-panel section">
      <div class="section-head">
        <h2>读取</h2>
        <button
          type="button"
          class="ui-btn ui-btn--primary"
          disabled={!target.ready || reading}
          onclick={() => void doRead()}
        >
          <Icon src={eyeIcon} size={14} />
          {reading ? "读取中…" : "读取内存"}
        </button>
      </div>

      {#if readError}
        <p class="ui-callout ui-callout--danger">
          <Icon src={alertIcon} size={14} />{readError}
        </p>
      {/if}

      {#if readResult}
        <HexViewer dataHex={readResult.dataHex} address={readResult.address} />
      {/if}
    </div>

    <div class="ui-panel section">
      <div class="section-head">
        <h2>写入</h2>
        <button
          type="button"
          class="ui-btn ui-btn--primary"
          disabled={!target.ready || writeRunning || !writeHex.trim()}
          onclick={() => void doWrite()}
        >
          {writeRunning ? "写入中…" : "写入内存"}
        </button>
      </div>

      <label class="ui-field">
        <span>写入数据（十六进制字节流）</span>
        <textarea class="ui-textarea ui-mono" bind:value={writeHex} rows="3"
        ></textarea>
      </label>

      <JobProgress jobId={writeJobId} />
    </div>
  {:else}
    <div class="ui-panel section">
      <div class="section-head">
        <h2>导出到文件</h2>
        <button
          type="button"
          class="ui-btn ui-btn--primary"
          disabled={!target.ready || dumpRunning || !dumpPath.trim()}
          onclick={() => void doDump()}
        >
          <Icon src={downloadIcon} size={14} />
          {dumpRunning ? "导出中…" : "导出内存"}
        </button>
      </div>

      <label class="ui-field">
        <span>输出路径</span>
        <div class="path-input">
          <Icon src={folderIcon} size={16} />
          <input
            class="ui-input ui-mono"
            bind:value={dumpPath}
            placeholder="/tmp/memory.bin"
            autocomplete="off"
          />
        </div>
      </label>

      <JobProgress jobId={dumpJobId} />
    </div>
  {/if}

  {#if actionError}
    <p class="ui-callout ui-callout--danger">
      <Icon src={alertIcon} size={14} />{actionError}
    </p>
  {/if}
</section>

<style>
  .view {
    display: grid;
    gap: var(--space-5);
    max-width: 720px;
  }

  .view-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
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

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .path-input {
    position: relative;
    display: flex;
    align-items: center;
    color: var(--color-text-faint);
  }

  .path-input :global(.icon) {
    position: absolute;
    left: 11px;
  }

  .path-input input {
    padding-left: 34px;
  }

  @media (max-width: 720px) {
    .view-head {
      flex-direction: column;
    }
  }
</style>
