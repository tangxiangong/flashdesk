<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import refreshIcon from "$lib/assets/icons/refresh.svg?url";
  import {
    isTauriRuntime,
    readableError,
    targetMemoryMap,
    type MemoryRegionKind,
    type MemoryRegionLayout,
  } from "$lib/api/tauri";
  import { target } from "$lib/state/target.svelte";

  let loading = $state(false);
  let regions = $state<MemoryRegionLayout[]>([]);
  let loadError = $state<string | null>(null);
  let loadedChip = $state("");

  let chip = $derived(target.effectiveChip.trim());
  let flashBytes = $derived(totalSize("nvm"));
  let ramBytes = $derived(totalSize("ram"));
  let connection = $derived(target.connection);

  $effect(() => {
    if (!chip || !target.connected) {
      regions = [];
      loadError = null;
      loadedChip = "";
      return;
    }

    if (isTauriRuntime() && chip !== loadedChip && !loading) {
      void loadLayout(chip);
    }
  });

  async function loadLayout(nextChip = chip) {
    if (!nextChip) return;

    loading = true;
    loadError = null;

    try {
      regions = await targetMemoryMap(nextChip);
      loadedChip = nextChip;
    } catch (err) {
      regions = [];
      loadedChip = nextChip;
      loadError = readableError(err);
    } finally {
      loading = false;
    }
  }

  function sizeLabel(bytes: number): string {
    if (bytes <= 0) return "--";
    if (bytes >= 1024 * 1024 && bytes % (1024 * 1024) === 0) {
      return `${bytes / (1024 * 1024)} MiB`;
    }
    if (bytes >= 1024 && bytes % 1024 === 0) {
      return `${bytes / 1024} KiB`;
    }
    return `${bytes} B`;
  }

  function totalSize(kind: MemoryRegionKind): number {
    return regions
      .filter((region) => region.kind === kind && !region.isAlias)
      .reduce((sum, region) => sum + region.size, 0);
  }
</script>

<section class="chip-info" aria-labelledby="chip-info-title">
  <header class="chip-head">
    <h2 id="chip-info-title">芯片</h2>
    <button
      type="button"
      class="refresh-button"
      title="刷新"
      aria-label="刷新"
      disabled={!chip || !target.connected || loading}
      onclick={() => void loadLayout()}
    >
      <Icon src={refreshIcon} size={14} />
    </button>
  </header>

  <dl class="info-list">
    <div>
      <dt>型号</dt>
      <dd class="ui-mono">{chip || "--"}</dd>
    </div>
    <div>
      <dt>烧录器</dt>
      <dd class="ui-mono">{connection?.probe ?? "--"}</dd>
    </div>
    <div>
      <dt>接口</dt>
      <dd>{connection?.protocol.toUpperCase() ?? "--"}</dd>
    </div>
    <div>
      <dt>速度</dt>
      <dd>{connection?.speedKhz ? `${connection.speedKhz} kHz` : "--"}</dd>
    </div>
  </dl>

  <div class="summary-grid">
    <div>
      <span>Flash</span>
      <strong>{loading ? "读取中" : sizeLabel(flashBytes)}</strong>
    </div>
    <div>
      <span>RAM</span>
      <strong>{loading ? "读取中" : sizeLabel(ramBytes)}</strong>
    </div>
  </div>

  {#if loadError}
    <p class="ui-callout ui-callout--danger">
      <Icon src={alertIcon} size={14} />{loadError}
    </p>
  {/if}
</section>

<style>
  .chip-info {
    display: grid;
    gap: 10px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    padding: 12px;
    box-shadow: var(--shadow-pop);
  }

  .chip-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .chip-head h2 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-lg);
    line-height: 1.1;
  }

  .refresh-button {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-inset);
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .refresh-button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .info-list {
    display: grid;
    gap: 1px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-border);
    margin: 0;
  }

  .info-list div {
    display: grid;
    grid-template-columns: 64px minmax(0, 1fr);
    gap: 8px;
    background: var(--color-surface-inset);
    padding: 7px 8px;
  }

  .info-list dt {
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 800;
  }

  .info-list dd {
    overflow: hidden;
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
  }

  .summary-grid div {
    display: grid;
    gap: 3px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-inset);
    padding: 8px;
  }

  .summary-grid span {
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 800;
    text-transform: uppercase;
  }

  .summary-grid strong {
    color: var(--color-text);
    font-size: var(--text-md);
  }
</style>
