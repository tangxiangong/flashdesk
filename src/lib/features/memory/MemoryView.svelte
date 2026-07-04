<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import eyeIcon from "$lib/assets/icons/eye.svg?url";
  import HexViewer from "./HexViewer.svelte";
  import { readMemory, type MemoryReadResult } from "$lib/api/tauri";
  import { parseAddressInput } from "$lib/utils/address";
  import { target } from "$lib/state/target.svelte";

  let address = $state("0x08000000");
  let length = $state(64);

  let reading = $state(false);
  let readResult = $state<MemoryReadResult | null>(null);
  let readError = $state<string | null>(null);

  let parsedAddress = $derived(parseAddressInput(address));

  function readableError(err: unknown): string {
    if (typeof err === "string") return err;
    if (err && typeof err === "object" && "message" in err) {
      return String((err as { message: unknown }).message);
    }
    return "读取内存失败";
  }

  async function doRead() {
    reading = true;
    readError = null;
    readResult = null;

    if (parsedAddress == null) {
      readError = "起始地址必须是有效的十进制或 0x 开头十六进制地址";
      reading = false;
      return;
    }

    try {
      readResult = await readMemory({
        probe: target.probe,
        target: target.selection(),
        address: parsedAddress,
        length,
      });
    } catch (err) {
      readError = readableError(err);
    } finally {
      reading = false;
    }
  }
</script>

<section class="view" aria-labelledby="memory-title">
  <header class="view-head">
    <div>
      <h1 id="memory-title">内存地址查看</h1>
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

    {#if parsedAddress == null}
      <p class="ui-callout ui-callout--danger">
        <Icon src={alertIcon} size={14} />起始地址必须是有效的十进制或 0x
        开头十六进制地址
      </p>
    {/if}
  </div>

  <div class="ui-panel section">
    <div class="section-head">
      <h2>只读显示</h2>
      <button
        type="button"
        class="ui-btn ui-btn--primary"
        disabled={!target.ready || reading || parsedAddress == null}
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

  @media (max-width: 720px) {
    .view-head {
      flex-direction: column;
    }
  }
</style>
