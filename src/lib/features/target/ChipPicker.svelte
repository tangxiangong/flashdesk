<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import searchIcon from "$lib/assets/icons/search.svg?url";
  import { target } from "$lib/state/target.svelte";

  /** 芯片型号选择器 props。 */
  interface Props {
    /** 关闭当前弹层的回调。 */
    close: () => void;
  }

  let { close }: Props = $props();

  let draft = $state(target.chip);
  let debounceHandle: ReturnType<typeof setTimeout> | undefined;
  let inputEl: HTMLInputElement | undefined;
  let candidateNames = $derived(
    new Set(target.targetCandidates.map((candidate) => candidate.name)),
  );

  $effect(() => {
    inputEl?.focus();
  });

  function onInput(value: string) {
    draft = value;
    clearTimeout(debounceHandle);
    debounceHandle = setTimeout(() => void target.searchChip(value), 220);
  }

  function commit(value: string) {
    const shouldReconnect = target.targetCandidates.some(
      (candidate) => candidate.name === value,
    );
    target.pickChip(value);
    draft = value;
    close();
    if (shouldReconnect) {
      void target.connect();
    }
  }
</script>

<div class="picker">
  <h3>{target.targetCandidates.length > 0 ? "选择候选目标" : "型号覆盖"}</h3>

  {#if target.targetInformation}
    <div class="target-information">
      <strong>{target.targetInformation.deviceType}</strong>
      {#if target.targetInformation.deviceId != null}
        <span class="ui-mono"
          >Device ID 0x{target.targetInformation.deviceId
            .toString(16)
            .toUpperCase()}</span
        >
      {/if}
      {#if target.targetInformation.revisionId != null}
        <span class="ui-mono"
          >Revision 0x{target.targetInformation.revisionId
            .toString(16)
            .toUpperCase()}</span
        >
      {/if}
      {#if target.targetInformation.cpu}
        <span>{target.targetInformation.cpu}</span>
      {/if}
    </div>
  {/if}

  <label class="search-field">
    <Icon src={searchIcon} size={15} />
    <input
      bind:this={inputEl}
      class="ui-input"
      value={draft}
      placeholder="输入型号"
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
        {@const candidate = target.targetCandidates.find(
          (item) => item.name === result,
        )}
        <button
          type="button"
          class="chip-row"
          class:selected={target.chip === result}
          class:candidate={candidateNames.has(result)}
          onclick={() => commit(result)}
        >
          <strong class="ui-mono">{result}</strong>
          {#if candidate}
            <span>{candidate.family}</span>
          {/if}
        </button>
      {/each}
    {:else if draft.trim()}
      <button
        type="button"
        class="chip-row"
        onclick={() => commit(draft.trim())}
      >
        <strong class="ui-mono">{draft.trim()}</strong>
        <span>使用手动型号</span>
      </button>
    {:else}
      <p class="empty">输入型号名称进行搜索，留空则自动识别</p>
    {/if}
  </div>
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
    font-weight: 800;
  }

  .search-field {
    position: relative;
    display: block;
    color: var(--color-text-faint);
  }

  .target-information {
    display: flex;
    flex-wrap: wrap;
    gap: 5px var(--space-2);
    border: 1px solid var(--color-accent-border);
    border-radius: var(--radius-sm);
    background: var(--color-accent-soft);
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    padding: var(--space-2);
  }

  .target-information strong {
    color: var(--color-accent-strong);
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
    display: grid;
    gap: 2px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-sm);
    padding: 7px var(--space-2);
    text-align: left;
  }

  .chip-row strong {
    overflow: hidden;
    color: var(--color-text);
    font-size: var(--text-sm);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip-row span {
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: var(--text-2xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip-row:hover,
  .chip-row.selected {
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
