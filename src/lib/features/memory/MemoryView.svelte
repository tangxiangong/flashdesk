<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import eyeIcon from "$lib/assets/icons/eye.svg?url";
  import {
    isTauriRuntime,
    readableError,
    readMemory,
    targetMemoryMap,
    type MemoryReadResult,
    type MemoryRegionLayout,
  } from "$lib/api/tauri";
  import { jobs } from "$lib/state/jobs.svelte";
  import { appStatus } from "$lib/state/status.svelte";
  import { target } from "$lib/state/target.svelte";
  import { parseAddressInput } from "$lib/utils/address";

  let address = $state("0x00000000");
  let length = $state(128);
  let addressEdited = $state(false);
  let reading = $state(false);
  let readResult = $state<MemoryReadResult | null>(null);
  let layoutLoading = $state(false);
  let layoutChip = $state("");
  let memoryContext = $state("");
  let observedConnection = $state<unknown>(null);
  let connectionRefreshSeq = $state(0);
  let pendingConnectRefreshKey = $state<string | null>(null);
  let lastAutoRefreshKey = $state("");

  let chip = $derived(target.connected ? target.effectiveChip.trim() : "");
  let nextMemoryContext = $derived(
    target.connected && chip ? `${target.connection?.probe ?? ""}|${chip}` : "",
  );
  let parsedAddress = $derived(parseAddressInput(address));
  let latestJob = $derived(jobs.events.at(-1));
  let rows = $derived(readResult ? toRows(readResult) : []);

  $effect(() => {
    if (target.connection !== observedConnection) {
      observedConnection = target.connection;
      if (target.connected && nextMemoryContext) {
        connectionRefreshSeq += 1;
        pendingConnectRefreshKey = `connect:${connectionRefreshSeq}`;
        readResult = null;
      } else {
        pendingConnectRefreshKey = null;
      }
    }

    if (nextMemoryContext !== memoryContext) {
      memoryContext = nextMemoryContext;
      readResult = null;
    }

    if (!target.connected || !chip) {
      layoutChip = "";
      return;
    }

    if (isTauriRuntime() && chip !== layoutChip && !layoutLoading) {
      void loadDefaultAddress(chip);
    } else if (pendingConnectRefreshKey && chip === layoutChip) {
      const key = pendingConnectRefreshKey;
      pendingConnectRefreshKey = null;
      triggerAutoRefresh(key);
    }
  });

  $effect(() => {
    if (target.connected && parsedAddress == null) {
      appStatus.danger("内存读取", "地址无效");
    } else if (appStatus.current?.label === "内存读取") {
      appStatus.clear();
    }
  });

  async function loadDefaultAddress(nextChip: string) {
    layoutLoading = true;

    try {
      const regions = await targetMemoryMap(nextChip);
      const defaultAddress = pickReadableAddress(regions);
      if (
        defaultAddress != null &&
        (!addressEdited || address === "0x00000000")
      ) {
        address = hexAddress(defaultAddress);
      }
    } catch {
      // Memory layout is a convenience for choosing a sane default. Reading still
      // works if the user enters an address manually.
    } finally {
      layoutChip = nextChip;
      layoutLoading = false;
      if (pendingConnectRefreshKey) {
        const key = pendingConnectRefreshKey;
        pendingConnectRefreshKey = null;
        triggerAutoRefresh(key);
      }
    }
  }

  $effect(() => {
    if (!latestJob || latestJob.stage !== "completed") {
      return;
    }

    if (latestJob.kind === "flash") {
      triggerAutoRefresh(`flash:${latestJob.id}:completed`);
    } else if (latestJob.kind === "erase") {
      triggerAutoRefresh(`erase:${latestJob.id}:completed`);
    }
  });

  function triggerAutoRefresh(key: string) {
    if (key === lastAutoRefreshKey || reading) {
      return;
    }

    lastAutoRefreshKey = key;
    void readCurrentMemory("正在读取目标内存", false, true);
  }

  async function readCurrentMemory(
    message: string,
    clearResult: boolean,
    silent = false,
  ) {
    if (!target.connected) {
      return;
    }

    const addressToRead = parseAddressInput(address);

    if (addressToRead == null) {
      if (!silent) {
        appStatus.danger("内存读取", "地址无效");
      }
      return;
    }

    if (length <= 0) {
      if (!silent) {
        appStatus.danger("内存读取", "长度无效");
      }
      return;
    }

    if (!silent) {
      appStatus.clear();
    }
    if (clearResult) {
      readResult = null;
    }

    reading = true;
    if (!silent) {
      appStatus.progress("读取内存", message);
    }

    try {
      readResult = await readMemory({
        probe: target.probe,
        target: target.selection(),
        address: addressToRead,
        length,
      });
      if (!silent) {
        appStatus.success("读取完成", `已读取 ${readResult.length} 字节`);
      }
    } catch (err) {
      if (!silent) {
        appStatus.danger("读取失败", readableError(err, "读取失败"));
      }
    } finally {
      reading = false;
    }
  }

  async function doRead() {
    await readCurrentMemory("正在读取目标内存", true);
  }

  function markAddressEdited() {
    addressEdited = true;
  }

  function pickReadableAddress(regions: MemoryRegionLayout[]): number | null {
    const readable = regions.filter(
      (region) => region.access.read && !region.isAlias && region.size > 0,
    );
    const preferred =
      readable.find((region) => region.kind === "ram") ??
      readable.find((region) => region.kind === "nvm") ??
      readable[0];

    return preferred?.start ?? null;
  }

  function toBytes(dataHex: string): number[] {
    const bytes: number[] = [];
    for (let i = 0; i + 1 < dataHex.length; i += 2) {
      bytes.push(Number.parseInt(dataHex.slice(i, i + 2), 16));
    }
    return bytes;
  }

  function toRows(result: MemoryReadResult) {
    const bytes = toBytes(result.dataHex);
    const output: Array<{ address: number; bytes: number[]; ascii: string }> =
      [];

    for (let offset = 0; offset < bytes.length; offset += 16) {
      const chunk = bytes.slice(offset, offset + 16);
      output.push({
        address: result.address + offset,
        bytes: chunk,
        ascii: chunk
          .map((byte) =>
            byte >= 0x20 && byte <= 0x7e ? String.fromCharCode(byte) : ".",
          )
          .join(""),
      });
    }

    return output;
  }

  function hexAddress(value: number): string {
    return `0x${Math.trunc(value).toString(16).padStart(8, "0")}`;
  }

  function hexByte(value: number): string {
    return value.toString(16).padStart(2, "0").toUpperCase();
  }
</script>

<div class="memory-tool">
  <div class="read-controls">
    <label>
      <span>地址</span>
      <input
        class="ui-input ui-mono"
        bind:value={address}
        autocomplete="off"
        oninput={markAddressEdited}
        placeholder="0x00000000"
      />
    </label>

    <label>
      <span>长度（字节）</span>
      <input
        class="ui-input ui-mono"
        type="number"
        min="1"
        max="4096"
        step="16"
        bind:value={length}
      />
    </label>

    <button
      type="button"
      class="ui-btn ui-btn--primary read-button"
      disabled={!target.connected ||
        reading ||
        parsedAddress == null ||
        length <= 0}
      onclick={() => void doRead()}
    >
      <Icon src={eyeIcon} size={14} />
      {reading ? "读取中…" : "读取"}
    </button>
  </div>

  <div class="memory-table ui-scrollbar" aria-label="内存数据">
    <div class="table-head">
      <span>Address</span>
      <span class="byte-head"
        >00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F</span
      >
      <span>ASCII</span>
    </div>

    {#if rows.length === 0}
      <p class="empty">{target.connected ? "尚未读取数据" : "请先连接设备"}</p>
    {:else}
      {#each rows as row (row.address)}
        <div class="table-row">
          <span class="ui-mono address-cell">{hexAddress(row.address)}</span>
          <span class="ui-mono bytes-cell">
            {#each Array.from({ length: 16 }) as _, index}
              <span
                >{row.bytes[index] == null
                  ? "--"
                  : hexByte(row.bytes[index])}</span
              >
            {/each}
          </span>
          <span class="ui-mono ascii-cell">{row.ascii}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .memory-tool {
    display: grid;
    gap: var(--space-3);
  }

  .read-controls {
    display: grid;
    grid-template-columns: minmax(160px, 220px) 140px auto;
    gap: var(--space-2);
    align-items: end;
  }

  .read-controls label {
    display: grid;
    gap: 4px;
  }

  .read-controls span {
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 700;
  }

  .read-button {
    white-space: nowrap;
  }

  .memory-table {
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface-inset);
  }

  .table-head,
  .table-row {
    display: grid;
    grid-template-columns: 100px minmax(430px, 1fr) 130px;
    gap: 10px;
    align-items: center;
    min-width: 680px;
    padding: 8px 12px;
  }

  .table-head {
    position: sticky;
    top: 0;
    z-index: 1;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-muted);
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 700;
  }

  .table-row {
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text);
    font-size: var(--text-xs);
  }

  .table-row:last-child {
    border-bottom: 0;
  }

  .bytes-cell {
    display: grid;
    grid-template-columns: repeat(16, 1fr);
    gap: 6px;
  }

  .bytes-cell span {
    text-align: center;
  }

  .ascii-cell {
    overflow: hidden;
    color: var(--color-text-muted);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    margin: 0;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    padding: var(--space-4);
  }

  @media (max-width: 640px) {
    .read-controls {
      grid-template-columns: 1fr;
    }
  }
</style>
