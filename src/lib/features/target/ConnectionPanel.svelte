<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Popover from "$lib/components/Popover.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import chevronIcon from "$lib/assets/icons/chevron-down.svg?url";
  import cpuIcon from "$lib/assets/icons/cpu.svg?url";
  import targetIcon from "$lib/assets/icons/target.svg?url";
  import slidersIcon from "$lib/assets/icons/sliders.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";
  import ProbePicker from "$lib/features/target/ProbePicker.svelte";
  import ChipPicker from "$lib/features/target/ChipPicker.svelte";
  import { appStatus } from "$lib/state/status.svelte";
  import { target } from "$lib/state/target.svelte";
  import {
    isTauriRuntime,
    readableError,
    targetMemoryMap,
    type MemoryRegionKind,
    type MemoryRegionLayout,
    type WireProtocol,
  } from "$lib/api/tauri";

  let probeOpen = $state(false);
  let chipOpen = $state(false);
  let gearOpen = $state(false);
  let didInitialProbeScan = $state(false);

  let layoutLoading = $state(false);
  let regions = $state<MemoryRegionLayout[]>([]);
  let loadedChip = $state("");

  let hasManualProbe = $derived(!target.connected && target.probe != null);
  let hasManualChip = $derived(!target.connected && target.chip.trim() !== "");

  let probeName = $derived(
    target.connected
      ? (target.connection?.probe ?? target.probe ?? "")
      : (target.selectedProbeSummary?.product ?? target.probe ?? ""),
  );
  let probeSub = $derived(
    target.connected
      ? (target.selectedProbeSummary?.serialNumber ?? "已连接")
      : target.probesLoading
        ? "扫描中…"
        : target.probes.length > 0
          ? `检测到 ${target.probes.length} 个设备`
          : "未检测到设备",
  );

  let chipName = $derived(
    target.connected ? (target.connection?.chip ?? "") : target.chip.trim(),
  );

  let sizeSuffix = $derived.by(() => {
    if (!target.connected || regions.length === 0) return "";
    const flash = totalSize("nvm");
    const ram = totalSize("ram");
    const parts: string[] = [];
    if (flash > 0) parts.push(`Flash ${sizeLabel(flash)}`);
    if (ram > 0) parts.push(`RAM ${sizeLabel(ram)}`);
    return parts.length > 0 ? ` · ${parts.join(" · ")}` : "";
  });

  let chipSub = $derived(
    target.connected && target.connection
      ? `${target.connection.protocol.toUpperCase()} · ${target.connection.speedKhz ?? target.speedKhz} kHz${sizeSuffix}`
      : `${target.protocol.toUpperCase()} · ${target.speedKhz} kHz`,
  );

  let statusState = $derived(
    target.connecting || target.probesLoading
      ? "busy"
      : target.connected
        ? "connected"
        : "idle",
  );
  let statusText = $derived(
    target.probesLoading
      ? "扫描中"
      : target.connecting
        ? "连接中"
        : target.connected
          ? "已连接"
          : "未连接",
  );
  function sizeLabel(bytes: number): string {
    if (bytes <= 0) return "--";
    if (bytes >= 1024 * 1024 && bytes % (1024 * 1024) === 0) {
      return `${bytes / (1024 * 1024)}M`;
    }
    if (bytes >= 1024 && bytes % 1024 === 0) {
      return `${bytes / 1024}K`;
    }
    return `${bytes}B`;
  }

  function totalSize(kind: MemoryRegionKind): number {
    return regions
      .filter((region) => region.kind === kind && !region.isAlias)
      .reduce((sum, region) => sum + region.size, 0);
  }

  async function loadLayout(nextChip: string) {
    if (!nextChip) return;

    layoutLoading = true;
    if (appStatus.current?.label === "内存布局") {
      appStatus.clear();
    }

    try {
      regions = await targetMemoryMap(nextChip);
      loadedChip = nextChip;
      if (appStatus.current?.label === "内存布局") {
        appStatus.clear();
      }
    } catch (err) {
      regions = [];
      loadedChip = nextChip;
      appStatus.danger("内存布局", readableError(err));
    } finally {
      layoutLoading = false;
    }
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

  $effect(() => {
    const chip = target.connected ? target.effectiveChip.trim() : "";

    if (!chip) {
      regions = [];
      loadedChip = "";
      return;
    }

    if (isTauriRuntime() && chip !== loadedChip && !layoutLoading) {
      void loadLayout(chip);
    }
  });
</script>

<div class="device-bar">
  <div class="device-bar-row">
    <div class="device-field">
      <Popover bind:open={probeOpen} width={340}>
        {#snippet trigger({ toggle, open })}
          <button
            type="button"
            class="field-trigger"
            aria-expanded={probeOpen}
            onclick={toggle}
          >
            <Icon src={cpuIcon} size={15} />
            <span class="field-copy">
              <strong class:is-placeholder={!probeName}
                >{probeName || "自动检测"}</strong
              >
              <small>{probeSub}</small>
            </span>
            <span class="field-chevron" class:open>
              <Icon src={chevronIcon} size={12} />
            </span>
          </button>
        {/snippet}
        {#snippet content({ close })}
          <ProbePicker {close} />
        {/snippet}
      </Popover>

      {#if hasManualProbe}
        <button
          type="button"
          class="field-clear"
          title="重置为自动检测"
          aria-label="重置为自动检测"
          onclick={() => target.pickProbe(null)}
        >
          <Icon src={xIcon} size={11} />
        </button>
      {/if}
    </div>

    <span class="bar-divider" aria-hidden="true"></span>

    <div class="device-field">
      <Popover bind:open={chipOpen} width={300}>
        {#snippet trigger({ toggle, open })}
          <button
            type="button"
            class="field-trigger"
            aria-expanded={chipOpen}
            onclick={toggle}
          >
            <Icon src={targetIcon} size={14} />
            <span class="field-copy">
              <strong class="ui-mono" class:is-placeholder={!chipName}
                >{chipName || "自动识别"}</strong
              >
              <small>{chipSub}</small>
            </span>
            <span class="field-chevron" class:open>
              <Icon src={chevronIcon} size={12} />
            </span>
          </button>
        {/snippet}
        {#snippet content({ close })}
          <ChipPicker {close} />
        {/snippet}
      </Popover>

      {#if hasManualChip}
        <button
          type="button"
          class="field-clear"
          title="清除型号覆盖"
          aria-label="清除型号覆盖"
          onclick={() => target.pickChip("")}
        >
          <Icon src={xIcon} size={11} />
        </button>
      {/if}
    </div>

    <Popover bind:open={gearOpen} align="end" width={260}>
      {#snippet trigger({ toggle })}
        <button
          type="button"
          class="ui-btn ui-btn--ghost ui-btn--icon"
          title="接口与速率"
          aria-label="接口与速率"
          aria-expanded={gearOpen}
          disabled={target.connected}
          onclick={toggle}
        >
          <Icon src={slidersIcon} size={15} />
        </button>
      {/snippet}
      {#snippet content()}
        <div class="gear-panel">
          <div class="gear-row">
            <span class="ui-label">接口</span>
            <Segmented
              value={target.protocol}
              options={[
                { value: "swd" as WireProtocol, label: "SWD" },
                { value: "jtag" as WireProtocol, label: "JTAG" },
              ]}
              onchange={(v) => (target.protocol = v)}
            />
          </div>

          <label class="gear-row">
            <span class="ui-label">速率 (kHz)</span>
            <input
              class="ui-input ui-mono"
              type="number"
              min="1"
              step="100"
              bind:value={target.speedKhz}
            />
          </label>

          <div class="ui-switch-row">
            <span class="ui-switch-copy">
              <strong>复位下连接</strong>
              <span>适用于低功耗或异常状态的芯片</span>
            </span>
            <button
              type="button"
              class="ui-switch"
              role="switch"
              aria-checked={target.connectUnderReset}
              aria-label="复位下连接"
              onclick={() =>
                (target.connectUnderReset = !target.connectUnderReset)}
            ></button>
          </div>
        </div>
      {/snippet}
    </Popover>

    <span class="bar-spacer"></span>

    <span class="bar-status" data-state={statusState}>
      <span
        class="ui-dot"
        class:ui-dot--pulse={statusState === "busy"}
        style="--dot-color: currentColor"
      ></span>
      {statusText}
    </span>

    {#if target.connected}
      <button
        type="button"
        class="ui-btn ui-btn--ghost bar-action bar-action--danger"
        onclick={() => target.disconnect()}
      >
        断开
      </button>
    {:else}
      <button
        type="button"
        class="ui-btn ui-btn--primary bar-action"
        disabled={target.connecting}
        onclick={() => void target.connect()}
      >
        {target.connecting ? "连接中…" : "连接"}
      </button>
    {/if}
  </div>
</div>

<style>
  .device-bar {
    display: grid;
    gap: 6px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    padding: 10px var(--space-6);
  }

  .device-bar-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
  }

  .device-field {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    flex: 1 1 0;
  }

  .field-trigger {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    font: inherit;
    padding: 6px 8px;
    text-align: left;
    transition: background var(--duration-fast) var(--ease-out);
  }

  .field-trigger:hover,
  .field-trigger[aria-expanded="true"] {
    background: var(--color-surface-muted);
  }

  .device-bar-row :global(.ui-btn--icon[aria-expanded="true"]) {
    background: var(--color-surface-muted);
    color: var(--color-text);
  }

  .field-copy {
    display: grid;
    gap: 0;
    min-width: 0;
    overflow: hidden;
    line-height: 1.25;
  }

  .field-copy strong {
    overflow: hidden;
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .field-copy strong.is-placeholder {
    color: var(--color-text-faint);
    font-weight: 600;
  }

  .field-copy small {
    overflow: hidden;
    color: var(--color-text-faint);
    font-size: var(--text-2xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .field-chevron {
    display: grid;
    flex-shrink: 0;
    place-items: center;
    color: var(--color-text-faint);
    transition: transform var(--duration-base) var(--ease-out);
  }

  .field-chevron.open {
    transform: rotate(180deg);
  }

  .field-clear {
    display: grid;
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    place-items: center;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--color-text-faint);
    cursor: pointer;
  }

  .field-clear:hover {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .bar-divider {
    flex-shrink: 0;
    width: 1px;
    height: 24px;
    background: var(--color-border);
  }

  .bar-spacer {
    flex: 1 1 auto;
    min-width: var(--space-2);
  }

  .bar-status {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    gap: 6px;
    color: var(--color-text-faint);
    font-size: var(--text-2xs);
    font-weight: 700;
  }

  .bar-status[data-state="connected"] {
    color: var(--color-success);
  }

  .bar-status[data-state="busy"] {
    color: var(--color-accent-strong);
  }

  .bar-action {
    flex-shrink: 0;
    min-height: 32px;
    padding: 0 var(--space-4);
  }

  .bar-action--danger {
    color: var(--color-danger);
  }

  .bar-action--danger:hover {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }

  .gear-panel {
    display: grid;
    gap: var(--space-3);
    min-width: 220px;
  }

  .gear-row {
    display: grid;
    gap: 6px;
  }

  .gear-row :global(.segmented) {
    width: 100%;
  }

  .gear-row :global(.segment) {
    flex: 1;
  }

  @media (max-width: 720px) {
    .device-bar {
      padding: 10px var(--space-3);
    }

    .device-bar-row {
      flex-wrap: wrap;
    }

    .bar-divider {
      display: none;
    }

    .device-field {
      flex: 1 1 100%;
    }

    .bar-spacer {
      display: none;
    }

    .bar-status {
      flex: 1 1 auto;
    }
  }
</style>
