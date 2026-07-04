<script lang="ts">
  import { toHexRows, formatAddress } from "./hex";

  let { dataHex, address }: { dataHex: string; address: number } = $props();

  let rows = $derived(toHexRows(dataHex, address));
</script>

<div class="hex ui-scrollbar">
  <table>
    <tbody>
      {#each rows as row (row.offset)}
        <tr>
          <td class="addr ui-mono">{formatAddress(row.offset)}</td>
          <td class="bytes ui-mono">
            {#each row.bytes as byte, i (i)}
              <span class:mid={i === 8}>{byte}</span>
            {/each}
          </td>
          <td class="ascii ui-mono">{row.ascii}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .hex {
    max-height: 320px;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-console-bg);
  }

  table {
    border-collapse: collapse;
    width: 100%;
  }

  td {
    padding: 3px 10px;
    font-size: var(--text-xs);
    white-space: nowrap;
    vertical-align: top;
  }

  .addr {
    color: var(--color-console-accent);
  }

  .bytes {
    color: var(--color-console-text);
    letter-spacing: 1px;
  }

  .bytes span {
    display: inline-block;
    width: 1.6em;
  }

  .bytes span.mid {
    margin-left: 8px;
  }

  .ascii {
    color: color-mix(in srgb, var(--color-console-text) 65%, transparent);
  }
</style>
