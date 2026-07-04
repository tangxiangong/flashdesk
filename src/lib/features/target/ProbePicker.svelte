<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import refreshIcon from "$lib/assets/icons/refresh.svg?url";
  import { target } from "$lib/state/target.svelte";

  let { close }: { close: () => void } = $props();

  $effect(() => {
    if (target.probes.length === 0 && !target.probesLoading) {
      void target.refreshProbes();
    }
  });

  function hex16(value: number): string {
    return value.toString(16).padStart(4, "0").toUpperCase();
  }
</script>

<div class="picker">
  <div class="picker-head">
    <h3>烧录器</h3>
    <button
      type="button"
      class="ui-btn ui-btn--ghost ui-btn--icon"
      title="重新扫描"
      aria-label="重新扫描"
      disabled={target.probesLoading}
      onclick={() => void target.refreshProbes()}
    >
      <Icon src={refreshIcon} size={15} />
    </button>
  </div>

  <div class="picker-list ui-scrollbar">
    <button
      type="button"
      class="probe-row"
      class:selected={target.probe === null}
      onclick={() => {
        target.pickProbe(null);
        close();
      }}
    >
      <span class="ui-dot" style="--dot-color: var(--color-text-faint)"></span>
      <div>
        <strong>自动</strong>
        <span>单个设备时使用</span>
      </div>
    </button>

    {#if target.probesLoading}
      <p class="empty">正在扫描…</p>
    {:else if target.probes.length === 0}
      <p class="empty">未找到烧录器</p>
    {:else}
      {#each target.probes as probe (probe.identifier)}
        <button
          type="button"
          class="probe-row"
          class:selected={target.probe === probe.identifier}
          onclick={() => {
            target.pickProbe(probe.identifier);
            close();
          }}
        >
          <span class="ui-dot" style="--dot-color: var(--color-success)"></span>
          <div>
            <strong>{probe.product ?? "烧录器"}</strong>
            <span class="probe-meta">
              <em class="ui-mono"
                >{hex16(probe.vendorId)}:{hex16(probe.productId)}</em
              >
              {#if probe.serialNumber}
                <em class="ui-mono">{probe.serialNumber}</em>
              {/if}
            </span>
            <span class="ui-mono selector-text">{probe.identifier}</span>
          </div>
        </button>
      {/each}
    {/if}
  </div>

  {#if target.probesError}
    <p class="picker-error">{target.probesError}</p>
  {/if}
</div>

<style>
  .picker {
    display: grid;
    gap: var(--space-3);
  }

  .picker-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .picker-head h3 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 700;
  }

  .picker-list {
    display: grid;
    gap: 4px;
    max-height: 280px;
    overflow: auto;
  }

  .probe-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
    font: inherit;
    padding: var(--space-2);
    text-align: left;
  }

  .probe-row .ui-dot {
    margin-top: 6px;
  }

  .probe-row div {
    display: grid;
    gap: 1px;
    min-width: 0;
  }

  .probe-row strong {
    color: var(--color-text);
    font-size: var(--text-sm);
  }

  .probe-row span {
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .probe-row .probe-meta {
    display: flex;
    gap: 8px;
    min-width: 0;
  }

  .probe-row em {
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-style: normal;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .probe-row .selector-text {
    color: var(--color-text-faint);
  }

  .probe-row:hover {
    background: var(--color-surface-muted);
  }

  .probe-row.selected {
    border-color: var(--color-accent-border);
    background: var(--color-accent-soft);
  }

  .empty,
  .picker-error {
    margin: 0;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    padding: var(--space-2);
  }

  .picker-error {
    color: var(--color-danger);
  }
</style>
