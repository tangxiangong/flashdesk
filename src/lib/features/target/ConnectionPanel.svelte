<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Popover from "$lib/components/Popover.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import chevronIcon from "$lib/assets/icons/chevron-down.svg?url";
  import refreshIcon from "$lib/assets/icons/refresh.svg?url";
  import targetIcon from "$lib/assets/icons/target.svg?url";
  import ProbePicker from "$lib/features/target/ProbePicker.svelte";
  import ChipPicker from "$lib/features/target/ChipPicker.svelte";
  import { target } from "$lib/state/target.svelte";
  import { isTauriRuntime } from "$lib/api/tauri";
  import type { WireProtocol } from "$lib/api/tauri";

  let probeOpen = $state(false);
  let chipOpen = $state(false);
  let didInitialProbeScan = $state(false);

  let probeLabel = $derived(
    target.selectedProbeSummary?.product ??
      (target.probe ? target.probe : "自动"),
  );
  let probeSerial = $derived(target.selectedProbeSummary?.serialNumber ?? "");
  let probeVidPid = $derived(
    target.selectedProbeSummary
      ? `${hex16(target.selectedProbeSummary.vendorId)}:${hex16(target.selectedProbeSummary.productId)}`
      : "自动",
  );
  let chipLabel = $derived(target.chip.trim() || "自动识别");
  function hex16(value: number): string {
    return value.toString(16).padStart(4, "0").toUpperCase();
  }

  $effect(() => {
    if (
      !didInitialProbeScan &&
      isTauriRuntime() &&
      target.probes.length === 0 &&
      !target.probesLoading
    ) {
      didInitialProbeScan = true;
      void target.refreshProbes();
    }
  });
</script>

<section class="connection-panel" aria-labelledby="connection-title">
  <div class="connection-head">
    <h2 id="connection-title">连接</h2>
    <button type="button" class="status-wrap" aria-label="连接状态">
      <span
        class="status-dot"
        class:ready={target.connected}
        class:loading={target.connecting || target.probesLoading}
        title={target.probesLoading
          ? "扫描中"
          : target.connecting
            ? "连接中"
            : target.connected
              ? "已连接"
              : "未连接"}
        aria-hidden="true"
      ></span>

      {#if target.connected && target.connection}
        <span class="hardware-popover">
          <span>
            <b>烧录器</b>
            <em class="ui-mono">{target.connection.probe}</em>
          </span>
          <span>
            <b>型号</b>
            <em class="ui-mono">{target.connection.chip}</em>
          </span>
          <span>
            <b>接口</b>
            <em>{target.connection.protocol.toUpperCase()}</em>
          </span>
          <span>
            <b>速度</b>
            <em>{target.connection.speedKhz ?? "-"} kHz</em>
          </span>
          <span>
            <b>VID:PID</b>
            <em class="ui-mono">{probeVidPid}</em>
          </span>
        </span>
      {/if}
    </button>
  </div>

  <div class="connection-fields">
    <div class="field-group probe-field">
      <span>烧录器</span>
      <Popover bind:open={probeOpen} width={360}>
        {#snippet trigger({ toggle })}
          <button
            type="button"
            class="select-control"
            aria-expanded={probeOpen}
            onclick={toggle}
          >
            <strong>{probeLabel}</strong>
            {#if probeSerial}
              <small class="ui-mono">{probeSerial}</small>
            {/if}
            <Icon src={chevronIcon} size={12} />
          </button>
        {/snippet}
        {#snippet content({ close })}
          <ProbePicker {close} />
        {/snippet}
      </Popover>
    </div>

    <div class="field-group protocol-field">
      <span>接口</span>
      <Segmented
        value={target.protocol}
        options={[
          { value: "swd" as WireProtocol, label: "SWD" },
          { value: "jtag" as WireProtocol, label: "JTAG" },
        ]}
        onchange={(v) => (target.protocol = v)}
      />
    </div>

    <label class="field-group speed">
      <span>速度</span>
      <div class="speed-input">
        <input
          class="ui-input ui-mono"
          type="number"
          min="1"
          step="100"
          bind:value={target.speedKhz}
        />
        <em>kHz</em>
      </div>
    </label>

    <label class="field-group reset-mode" title="复位下连接">
      <span>复位下连接</span>
      <button
        type="button"
        class="reset-toggle"
        role="switch"
        aria-checked={target.connectUnderReset}
        aria-label="复位下连接"
        onclick={() => (target.connectUnderReset = !target.connectUnderReset)}
      >
        {target.connectUnderReset ? "开" : "关"}
      </button>
    </label>

    <Popover bind:open={chipOpen} width={300}>
      {#snippet trigger({ toggle })}
        <button
          type="button"
          class="target-button"
          title={`型号覆盖：${chipLabel}`}
          aria-label={`型号覆盖：${chipLabel}`}
          aria-expanded={chipOpen}
          onclick={toggle}
        >
          <Icon src={targetIcon} size={14} />
        </button>
      {/snippet}
      {#snippet content({ close })}
        <ChipPicker {close} />
      {/snippet}
    </Popover>

    <button
      type="button"
      class="scan-button"
      title="重新扫描烧录器"
      aria-label="重新扫描烧录器"
      disabled={target.probesLoading}
      onclick={() => void target.refreshProbes()}
    >
      <Icon src={refreshIcon} size={14} />
    </button>

    <div class="connection-actions">
      <button
        type="button"
        class="connect-button"
        disabled={target.connecting || target.connected}
        onclick={() => void target.connect()}
      >
        {target.connecting ? "连接中" : target.connected ? "已连接" : "连接"}
      </button>

      {#if target.connected}
        <button
          type="button"
          class="disconnect-button"
          onclick={() => target.disconnect()}
        >
          断开
        </button>
      {/if}
    </div>
  </div>

  {#if target.probesError}
    <p class="connection-error">{target.probesError}</p>
  {/if}

  {#if target.connectError}
    <p class="connection-error">{target.connectError}</p>
  {/if}
</section>

<style>
  .connection-panel {
    display: grid;
    gap: 8px;
    width: 100%;
    max-width: 100%;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--color-border);
  }

  .connection-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .connection-head {
    position: relative;
  }

  .status-wrap {
    position: relative;
    display: inline-grid;
    place-items: center;
    border: 0;
    background: transparent;
    cursor: default;
    padding: 0;
    outline: none;
  }

  .connection-head h2 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-lg);
    line-height: 1.1;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-warning);
  }

  .status-dot.ready {
    background: var(--color-success);
  }

  .status-dot.loading {
    background: var(--color-accent);
  }

  .hardware-popover {
    position: absolute;
    top: 18px;
    left: -8px;
    z-index: 20;
    display: grid;
    gap: 6px;
    width: max-content;
    max-width: 360px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    box-shadow: var(--shadow-pop);
    opacity: 0;
    padding: 8px;
    pointer-events: none;
    transform: translateY(-2px);
    transition:
      opacity var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  .status-wrap:hover .hardware-popover,
  .status-wrap:focus .hardware-popover,
  .status-wrap:focus-within .hardware-popover {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0);
  }

  .hardware-popover span {
    display: grid;
    grid-template-columns: 52px minmax(0, 1fr);
    gap: 8px;
    align-items: baseline;
  }

  .hardware-popover b {
    color: var(--color-text-muted);
    font-size: 10px;
    font-weight: 800;
  }

  .hardware-popover em {
    overflow: hidden;
    color: var(--color-text);
    font-size: var(--text-xs);
    font-style: normal;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .connection-fields {
    display: grid;
    grid-template-columns: 220px 132px 112px 116px 32px 32px auto;
    align-items: end;
    gap: 8px;
    width: max-content;
    max-width: 100%;
  }

  .field-group {
    display: grid;
    gap: 4px;
    min-width: 0;
  }

  .probe-field {
    width: 220px;
  }

  .protocol-field {
    width: 132px;
  }

  .speed {
    width: 112px;
  }

  .reset-mode {
    width: 116px;
  }

  .field-group > span {
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 800;
  }

  .field-group :global(.popover-root) {
    width: 100%;
    min-width: 0;
  }

  .connection-fields > :global(.popover-root) {
    width: 32px;
    flex: 0 0 32px;
  }

  .select-control,
  .speed-input,
  .reset-toggle,
  .target-button,
  .scan-button,
  .connect-button,
  .disconnect-button {
    min-height: 32px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-inset);
  }

  .select-control {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    padding: 0 8px;
    min-width: 0;
  }

  .select-control:hover,
  .connect-button:hover:not(:disabled),
  .scan-button:hover:not(:disabled),
  .disconnect-button:hover {
    border-color: var(--color-border-strong);
    background: var(--color-surface-muted);
  }

  .select-control strong {
    min-width: 0;
    flex: 1 1 auto;
    overflow: hidden;
    color: var(--color-text);
    font-size: var(--text-xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .select-control small {
    min-width: 0;
    flex: 0 1 80px;
    overflow: hidden;
    color: var(--color-text-faint);
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .field-group :global(.segmented) {
    width: 100%;
  }

  .field-group :global(.segment) {
    flex: 1;
    min-height: 30px;
    padding: 0 6px;
    font-size: var(--text-xs);
  }

  .speed-input {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
  }

  .speed input {
    min-height: 24px;
    border: 0;
    background: transparent;
    padding: 0;
    font-size: var(--text-xs);
    text-align: left;
    appearance: textfield;
    -moz-appearance: textfield;
  }

  .speed input::-webkit-outer-spin-button,
  .speed input::-webkit-inner-spin-button {
    margin: 0;
  }

  .speed em {
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-style: normal;
  }

  .reset-toggle {
    width: 100%;
    color: var(--color-text-muted);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 800;
  }

  .reset-toggle[aria-checked="true"] {
    border-color: var(--color-accent-border);
    background: var(--color-accent-soft);
    color: var(--color-accent-strong);
  }

  .scan-button {
    display: grid;
    width: 32px;
    align-self: end;
    place-items: center;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .target-button {
    display: grid;
    width: 32px;
    align-self: end;
    place-items: center;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .connection-actions {
    display: flex;
    align-items: end;
    gap: 6px;
    align-self: end;
  }

  .connect-button {
    width: 76px;
    min-height: 32px;
    padding: 0 12px;
    background: var(--color-accent);
    color: var(--color-text-inverse);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 900;
  }

  .connect-button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .disconnect-button {
    min-height: 32px;
    padding: 0 10px;
    color: var(--color-danger);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 900;
  }

  .scan-button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .connection-error {
    margin: 0;
    color: var(--color-danger);
    font-size: var(--text-xs);
  }

  @media (max-width: 640px) {
    .connection-panel {
      width: 100%;
    }

    .connection-fields {
      display: flex;
      align-items: stretch;
      flex-direction: column;
      width: 100%;
    }

    .field-group,
    .probe-field,
    .protocol-field,
    .speed,
    .reset-mode {
      width: 100%;
    }
  }
</style>
