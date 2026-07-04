<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import searchIcon from "$lib/assets/icons/search.svg?url";
  import { target } from "$lib/state/target.svelte";

  let { close }: { close: () => void } = $props();

  let draft = $state(target.chip);
  let debounceHandle: ReturnType<typeof setTimeout> | undefined;
  let inputEl: HTMLInputElement | undefined;

  $effect(() => {
    inputEl?.focus();
  });

  function onInput(value: string) {
    draft = value;
    clearTimeout(debounceHandle);
    debounceHandle = setTimeout(() => void target.searchChip(value), 220);
  }

  function commit(value: string) {
    target.pickChip(value);
    draft = value;
    close();
  }
</script>

<div class="picker">
  <h3>目标芯片</h3>

  <label class="search-field">
    <Icon src={searchIcon} size={15} />
    <input
      bind:this={inputEl}
      class="ui-input"
      value={draft}
      placeholder="输入或搜索型号，例如 STM32F103"
      autocomplete="off"
      oninput={(e) => onInput(e.currentTarget.value)}
      onkeydown={(e) => {
        if (e.key === "Enter") commit(draft.trim());
      }}
    />
  </label>

  <div class="picker-list ui-scrollbar">
    {#if target.chipSearching}
      <p class="empty">搜索中…</p>
    {:else if target.chipResults.length > 0}
      {#each target.chipResults as result (result)}
        <button type="button" class="chip-row" onclick={() => commit(result)}>
          {result}
        </button>
      {/each}
    {:else if draft.trim()}
      <p class="empty">未匹配到内置芯片，可直接确认手动输入</p>
    {:else}
      <p class="empty">支持模糊搜索 probe-rs 内置芯片数据库</p>
    {/if}
  </div>

  <button
    type="button"
    class="ui-btn ui-btn--primary ui-btn--block"
    disabled={!draft.trim()}
    onclick={() => commit(draft.trim())}
  >
    使用 “{draft.trim() || "…"}”
  </button>
</div>

<style>
  .picker {
    display: grid;
    gap: var(--space-3);
  }

  .picker h3 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-sm);
    font-weight: 700;
  }

  .search-field {
    position: relative;
    display: block;
    color: var(--color-text-faint);
  }

  .search-field input {
    padding-left: 34px;
  }

  .search-field :global(.icon) {
    position: absolute;
    top: 50%;
    left: 10px;
    transform: translateY(-50%);
  }

  .picker-list {
    display: grid;
    gap: 2px;
    max-height: 220px;
    overflow: auto;
  }

  .chip-row {
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    padding: 7px var(--space-2);
    text-align: left;
  }

  .chip-row:hover {
    border-color: var(--color-accent-border);
    background: var(--color-accent-soft);
  }

  .empty {
    margin: 0;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    padding: var(--space-2);
  }
</style>
