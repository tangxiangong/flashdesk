<script lang="ts">
  import Segmented from "$lib/components/Segmented.svelte";
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
</style>
