<script lang="ts">
  import Segmented from "$lib/components/Segmented.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import checkIcon from "$lib/assets/icons/check.svg?url";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import { target } from "$lib/state/target.svelte";
  import type { WireProtocol } from "$lib/api/tauri";
</script>

<div class="picker">
  <h3>连接参数</h3>

  <div class="row">
    <span>协议</span>
    <Segmented
      value={target.protocol}
      options={[
        { value: "swd" as WireProtocol, label: "SWD" },
        { value: "jtag" as WireProtocol, label: "JTAG" },
      ]}
      onchange={(v) => (target.protocol = v)}
    />
  </div>

  <label class="ui-field">
    <span>速度 (kHz)</span>
    <input
      class="ui-input ui-mono"
      type="number"
      min="1"
      step="100"
      bind:value={target.speedKhz}
    />
  </label>

  <div class="ui-switch-row">
    <div class="ui-switch-copy">
      <strong>复位下连接</strong>
      <span>适用于无法正常附加的目标</span>
    </div>
    <button
      type="button"
      class="ui-switch"
      role="switch"
      aria-checked={target.connectUnderReset}
      aria-label="复位下连接"
      onclick={() => (target.connectUnderReset = !target.connectUnderReset)}
    ></button>
  </div>

  <div class="test-row">
    <button
      type="button"
      class="ui-btn"
      disabled={!target.ready || target.link.state === "checking"}
      onclick={() => void target.testLink()}
    >
      {target.link.state === "checking" ? "检查中…" : "测试连接"}
    </button>

    {#if target.link.state === "ok"}
      <span class="result ok">
        <Icon src={checkIcon} size={13} />
        core {target.link.status.core} · {target.link.status.halted
          ? "已暂停"
          : "运行中"}
      </span>
    {:else if target.link.state === "error"}
      <span class="result err">
        <Icon src={alertIcon} size={13} />
        {target.link.message}
      </span>
    {/if}
  </div>
</div>

<style>
  .picker {
    display: grid;
    gap: var(--space-4);
  }

  .picker h3 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 700;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .row > span {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .test-row {
    display: grid;
    gap: var(--space-2);
  }

  .result {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-xs);
    overflow-wrap: anywhere;
  }

  .result.ok {
    color: var(--color-success);
  }

  .result.err {
    color: var(--color-danger);
  }
</style>
