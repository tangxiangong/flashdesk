<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import chevronDownIcon from "$lib/assets/icons/chevron-down.svg?url";
  import eyeIcon from "$lib/assets/icons/eye.svg?url";
  import {
    isTauriRuntime,
    readableError,
    readMemory,
    targetMemoryMap,
    type MemoryReadResult,
    type MemoryRegionLayout,
  } from "$lib/api/tauri";
  import { target } from "$lib/state/target.svelte";
  import { parseAddressInput } from "$lib/utils/address";

  let address = $state("0x00000000");
  let length = $state(256);
  let expanded = $state(false);
  let addressEdited = $state(false);
  let reading = $state(false);
  let readError = $state<string | null>(null);
  let readResult = $state<MemoryReadResult | null>(null);
  let layoutLoading = $state(false);
  let layoutChip = $state("");
  let memoryContext = $state("");

  let chip = $derived(target.connected ? target.effectiveChip.trim() : "");
  let nextMemoryContext = $derived(
    target.connected && chip ? `${target.connection?.probe ?? ""}|${chip}` : "",
  );
  let parsedAddress = $derived(parseAddressInput(address));
  let rows = $derived(readResult ? toRows(readResult) : []);

  $effect(() => {
    if (nextMemoryContext !== memoryContext) {
      memoryContext = nextMemoryContext;
      readError = null;
      readResult = null;
    }

    if (!target.connected || !chip) {
      layoutChip = "";
      return;
    }

    if (isTauriRuntime() && chip !== layoutChip && !layoutLoading) {
      void loadDefaultAddress(chip);
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
        readError = null;
      }
    } catch {
      // Memory layout is a convenience for choosing a sane default. Reading still
      // works if the user enters an address manually.
    } finally {
      layoutChip = nextChip;
      layoutLoading = false;
    }
  }

  async function doRead() {
    readError = null;
    readResult = null;

    if (parsedAddress == null) {
      readError = "地址无效";
      return;
    }

    reading = true;

    try {
      readResult = await readMemory({
        probe: target.probe,
        target: target.selection(),
        address: parsedAddress,
        length,
      });
    } catch (err) {
      readError = readableError(err, "读取失败");
    } finally {
      reading = false;
    }
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

<section class="memory-tool" aria-labelledby="memory-title">
  <header class="memory-head">
    <h2 id="memory-title">内存</h2>
    <button
      type="button"
      class="memory-toggle"
      aria-expanded={expanded}
      aria-controls="memory-body"
      title={expanded ? "收起" : "展开"}
      onclick={() => (expanded = !expanded)}
    >
      <Icon src={chevronDownIcon} size={14} />
    </button>
  </header>

  {#if expanded}
    <div id="memory-body" class="memory-body">
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
          <span>长度</span>
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
          class="read-button"
          disabled={!target.connected ||
            reading ||
            parsedAddress == null ||
            length <= 0}
          onclick={() => void doRead()}
        >
          <Icon src={eyeIcon} size={14} />
          {reading ? "读取中" : "读取"}
        </button>
      </div>

      {#if readError}
        <p class="ui-callout ui-callout--danger">
          <Icon src={alertIcon} size={14} />{readError}
        </p>
      {/if}

      <div class="memory-table ui-scrollbar" aria-label="内存数据">
        <div class="table-head">
          <span>Address</span>
          <span class="byte-head"
            >00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F</span
          >
          <span>ASCII</span>
        </div>

        {#if rows.length === 0}
          <p class="empty">{target.connected ? "未读取" : "未连接"}</p>
        {:else}
          {#each rows as row (row.address)}
            <div class="table-row">
              <span class="ui-mono address-cell">{hexAddress(row.address)}</span
              >
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
  {/if}
</section>

<style>
  .memory-tool {
    display: grid;
    gap: 10px;
    border-top: 1px solid var(--color-border);
    padding-top: 10px;
  }

  .memory-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .memory-head h2 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-lg);
    line-height: 1.1;
  }

  .memory-toggle {
    display: grid;
    width: 28px;
    height: 28px;
    place-items: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-inset);
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .memory-toggle :global(.icon) {
    transition: transform var(--duration-base) var(--ease-out);
  }

  .memory-toggle[aria-expanded="true"] :global(.icon) {
    transform: rotate(180deg);
  }

  .memory-body {
    display: grid;
    gap: 10px;
  }

  .read-controls {
    display: grid;
    grid-template-columns: minmax(160px, 220px) 112px 82px;
    gap: 8px;
    align-items: end;
    width: max-content;
    max-width: 100%;
  }

  .read-controls label {
    display: grid;
    gap: 4px;
  }

  .read-controls span {
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 800;
  }

  .read-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 30px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-muted);
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 900;
    padding: 0 10px;
  }

  .read-button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .memory-table {
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-inset);
  }

  .table-head,
  .table-row {
    display: grid;
    grid-template-columns: 112px minmax(520px, 1fr) 140px;
    gap: 10px;
    align-items: center;
    min-width: 800px;
    padding: 6px 8px;
  }

  .table-head {
    position: sticky;
    top: 0;
    z-index: 1;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-muted);
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    font-weight: 800;
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
    padding: 10px;
  }

  @media (max-width: 720px) {
    .read-controls {
      grid-template-columns: 1fr;
      width: 100%;
    }
  }
</style>
