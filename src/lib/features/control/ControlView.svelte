<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import JobProgress from "$lib/components/JobProgress.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import checkIcon from "$lib/assets/icons/check.svg?url";
  import resetIcon from "$lib/assets/icons/reset.svg?url";
  import eraseIcon from "$lib/assets/icons/erase.svg?url";
  import {
    attachTarget,
    resetTarget,
    eraseTarget,
    type TargetStatus,
  } from "$lib/api/tauri";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

  type Tab = "reset" | "erase";
  type EraseScope = "full" | "range";

  let tab = $state<Tab>("reset");
  let haltAfterReset = $state(false);

  let checking = $state(false);
  let checkResult = $state<TargetStatus | null>(null);
  let checkError = $state<string | null>(null);

  let resetJobId = $state<string | null>(null);
  let resetError = $state<string | null>(null);
  let resetLatest = $derived(jobs.latestFor(resetJobId));
  let resetRunning = $derived(
    resetJobId != null && !isStageTerminal(resetLatest?.stage ?? "queued"),
  );

  let eraseScope = $state<EraseScope>("full");
  let eraseStart = $state("0x08000000");
  let eraseEnd = $state("0x08010000");
  let eraseJobId = $state<string | null>(null);
  let eraseError = $state<string | null>(null);
  let eraseLatest = $derived(jobs.latestFor(eraseJobId));
  let eraseRunning = $derived(
    eraseJobId != null && !isStageTerminal(eraseLatest?.stage ?? "queued"),
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

  async function checkConnection() {
    checking = true;
    checkError = null;
    checkResult = null;

    try {
      checkResult = await attachTarget({
        probe: target.probe,
        target: target.selection(),
        haltAfterReset,
      });
    } catch (err) {
      checkError = readableError(err);
    } finally {
      checking = false;
    }
  }

  async function doReset() {
    resetError = null;
    resetJobId = null;

    try {
      resetJobId = await resetTarget({
        probe: target.probe,
        target: target.selection(),
        haltAfterReset,
      });
    } catch (err) {
      resetError = readableError(err);
    }
  }

  async function doErase() {
    eraseError = null;
    eraseJobId = null;

    try {
      eraseJobId = await eraseTarget({
        probe: target.probe,
        target: target.selection(),
        range:
          eraseScope === "range"
            ? { start: parseAddress(eraseStart), end: parseAddress(eraseEnd) }
            : null,
      });
    } catch (err) {
      eraseError = readableError(err);
    }
  }
</script>

<section class="view" aria-labelledby="control-title">
  <header class="view-head">
    <div>
      <h1 id="control-title">目标控制</h1>
    </div>

    <div class="ui-tabs" role="tablist">
      <button
        type="button"
        class="ui-tab"
        role="tab"
        aria-selected={tab === "reset"}
        onclick={() => (tab = "reset")}>复位</button
      >
      <button
        type="button"
        class="ui-tab ui-tab--danger"
        role="tab"
        aria-selected={tab === "erase"}
        onclick={() => (tab = "erase")}>擦除</button
      >
    </div>
  </header>

  {#if !target.ready}
    <p class="ui-callout ui-callout--warning">
      <Icon src={alertIcon} size={14} />请先在顶部选择目标芯片
    </p>
  {/if}

  {#if tab === "reset"}
    <div class="ui-panel section">
      <h2>复位选项</h2>

      <div class="ui-switch-row">
        <div class="ui-switch-copy">
          <strong>复位后暂停核心</strong>
          <span>复位并立即 halt，便于调试而非直接运行</span>
        </div>
        <button
          type="button"
          class="ui-switch"
          role="switch"
          aria-checked={haltAfterReset}
          aria-label="复位后暂停核心"
          onclick={() => (haltAfterReset = !haltAfterReset)}
        ></button>
      </div>

      <div class="actions">
        <button
          type="button"
          class="ui-btn"
          disabled={!target.ready || checking}
          onclick={() => void checkConnection()}
        >
          {checking ? "检查中…" : "连接检查"}
        </button>

        <button
          type="button"
          class="ui-btn ui-btn--primary"
          disabled={!target.ready || resetRunning}
          onclick={() => void doReset()}
        >
          <Icon src={resetIcon} size={14} />
          {resetRunning ? "复位中…" : "执行复位"}
        </button>
      </div>

      {#if checkResult}
        <p class="ui-callout ui-callout--success">
          <Icon src={checkIcon} size={14} />
          芯片 {checkResult.chip} · core {checkResult.core} ·
          {checkResult.halted ? "已暂停" : "运行中"}
        </p>
      {/if}

      {#if checkError}
        <p class="ui-callout ui-callout--danger">
          <Icon src={alertIcon} size={14} />{checkError}
        </p>
      {/if}

      <JobProgress jobId={resetJobId} />

      {#if resetError}
        <p class="ui-callout ui-callout--danger">
          <Icon src={alertIcon} size={14} />{resetError}
        </p>
      {/if}
    </div>
  {:else}
    <div class="ui-panel section danger-zone">
      <div class="danger-head">
        <h2><Icon src={eraseIcon} size={16} />危险操作</h2>
        <span class="ui-badge ui-badge--danger">不可撤销</span>
      </div>
      <p class="danger-copy">
        擦除会永久清除 Flash
        内容。整片擦除将移除全部数据，请确认目标芯片正确无误后再执行。
      </p>

      <div class="scope">
        <span>擦除范围</span>
        <Segmented
          value={eraseScope}
          options={[
            { value: "full" as EraseScope, label: "整片擦除" },
            { value: "range" as EraseScope, label: "指定范围" },
          ]}
          onchange={(v) => (eraseScope = v)}
        />
      </div>

      {#if eraseScope === "range"}
        <div class="ui-grid-2">
          <label class="ui-field">
            <span>起始地址</span>
            <input
              class="ui-input ui-mono"
              bind:value={eraseStart}
              autocomplete="off"
            />
          </label>
          <label class="ui-field">
            <span>结束地址</span>
            <input
              class="ui-input ui-mono"
              bind:value={eraseEnd}
              autocomplete="off"
            />
          </label>
        </div>
      {/if}

      <ConfirmButton
        label={eraseScope === "full" ? "整片擦除" : "擦除指定范围"}
        confirmLabel="再次点击以确认擦除"
        disabled={!target.ready || eraseRunning}
        onconfirm={() => void doErase()}
      />

      <JobProgress jobId={eraseJobId} />

      {#if eraseError}
        <p class="ui-callout ui-callout--danger">
          <Icon src={alertIcon} size={14} />{eraseError}
        </p>
      {/if}
    </div>
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

  .actions {
    display: flex;
    gap: var(--space-3);
  }

  .actions .ui-btn {
    flex: 1;
  }

  .danger-zone {
    border-color: var(--color-danger-border);
  }

  .danger-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .danger-head h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--color-danger);
  }

  .danger-copy {
    margin: 0;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .scope {
    display: grid;
    gap: var(--space-2);
  }

  .scope > span {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  @media (max-width: 720px) {
    .view-head {
      flex-direction: column;
    }

    .actions {
      flex-direction: column;
    }
  }
</style>
