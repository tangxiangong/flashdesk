<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import chevronDownIcon from "$lib/assets/icons/chevron-down.svg?url";
  import eraseIcon from "$lib/assets/icons/erase.svg?url";
  import eyeIcon from "$lib/assets/icons/eye.svg?url";
  import EraseView from "$lib/features/erase/EraseView.svelte";
  import MemoryView from "$lib/features/memory/MemoryView.svelte";

  type Tab = "erase" | "memory";

  let expanded = $state(false);
  let tab = $state<Tab>("erase");
</script>

<section class="tools-panel ui-panel" aria-labelledby="tools-title">
  <button
    type="button"
    class="tools-toggle"
    aria-expanded={expanded}
    aria-controls="tools-body"
    onclick={() => (expanded = !expanded)}
  >
    <span class="tools-toggle-copy">
      <strong id="tools-title">高级工具</strong>
      <span>擦除芯片、查看内存数据</span>
    </span>
    <span class="chevron" class:open={expanded}>
      <Icon src={chevronDownIcon} size={16} />
    </span>
  </button>

  {#if expanded}
    <div id="tools-body" class="tools-body">
      <div class="ui-tabs" role="tablist">
        <button
          type="button"
          role="tab"
          class="ui-tab"
          aria-selected={tab === "erase"}
          onclick={() => (tab = "erase")}
        >
          <Icon src={eraseIcon} size={13} /> 擦除
        </button>
        <button
          type="button"
          role="tab"
          class="ui-tab"
          aria-selected={tab === "memory"}
          onclick={() => (tab = "memory")}
        >
          <Icon src={eyeIcon} size={13} /> 内存
        </button>
      </div>

      <div class="tools-content">
        {#if tab === "erase"}
          <EraseView />
        {:else}
          <MemoryView />
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .tools-panel {
    padding: var(--space-2) var(--space-2);
  }

  .tools-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    width: 100%;
    border: 0;
    background: transparent;
    cursor: pointer;
    font: inherit;
    padding: var(--space-3) var(--space-3);
    text-align: left;
  }

  .tools-toggle-copy {
    display: grid;
    gap: 2px;
  }

  .tools-toggle-copy strong {
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 700;
  }

  .tools-toggle-copy span {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
  }

  .chevron {
    display: grid;
    place-items: center;
    color: var(--color-text-faint);
    transition: transform var(--duration-base) var(--ease-out);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .tools-body {
    display: grid;
    gap: var(--space-4);
    border-top: 1px solid var(--color-border);
    padding: var(--space-4) var(--space-3) var(--space-3);
  }

  .tools-content {
    display: grid;
  }
</style>
