<script lang="ts" generics="T extends string">
  let {
    value,
    options,
    onchange,
    disabled = false,
  }: {
    value: T;
    options: Array<{ value: T; label: string }>;
    onchange: (value: T) => void;
    disabled?: boolean;
  } = $props();
</script>

<div class="segmented" role="tablist">
  {#each options as option (option.value)}
    <button
      type="button"
      role="tab"
      class="segment"
      aria-selected={value === option.value}
      {disabled}
      onclick={() => onchange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    gap: 2px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface-inset);
    padding: 3px;
  }

  .segment {
    border: 0;
    border-radius: calc(var(--radius-md) - 3px);
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-sm);
    font-weight: 600;
    min-height: 32px;
    padding: 0 var(--space-3);
    transition:
      background var(--duration-base) var(--ease-out),
      color var(--duration-base) var(--ease-out);
  }

  .segment:hover:not(:disabled) {
    color: var(--color-text);
  }

  .segment[aria-selected="true"] {
    background: var(--color-surface);
    color: var(--color-accent-strong);
    box-shadow: var(--shadow-xs);
  }

  .segment:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
