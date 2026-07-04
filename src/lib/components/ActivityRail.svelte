<script lang="ts">
  import Icon from "./Icon.svelte";
  import flashIcon from "$lib/assets/icons/flash.svg?url";
  import memoryIcon from "$lib/assets/icons/memory.svg?url";
  import controlIcon from "$lib/assets/icons/control.svg?url";
  import { ui, type ViewKey } from "$lib/state/ui.svelte";

  const items: Array<{ key: ViewKey; label: string; icon: string }> = [
    { key: "flash", label: "烧录", icon: flashIcon },
    { key: "memory", label: "内存", icon: memoryIcon },
    { key: "control", label: "控制", icon: controlIcon },
  ];
</script>

<nav class="rail" aria-label="主导航">
  {#each items as item (item.key)}
    <button
      type="button"
      class="rail-item"
      class:active={ui.view === item.key}
      aria-current={ui.view === item.key ? "page" : undefined}
      title={item.label}
      onclick={() => ui.go(item.key)}
    >
      <Icon src={item.icon} size={20} />
      <span>{item.label}</span>
    </button>
  {/each}
</nav>

<style>
  .rail {
    display: grid;
    align-content: start;
    justify-items: center;
    gap: var(--space-2);
    width: var(--activity-width);
    padding: var(--space-3) 0;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .rail-item {
    display: grid;
    justify-items: center;
    gap: 4px;
    width: 48px;
    padding: 8px 0;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--color-text-faint);
    cursor: pointer;
    transition:
      background var(--duration-base) var(--ease-out),
      color var(--duration-base) var(--ease-out);
  }

  .rail-item span {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .rail-item:hover {
    color: var(--color-text);
    background: var(--color-surface-muted);
  }

  .rail-item.active {
    color: var(--color-accent-strong);
    background: var(--color-accent-soft);
  }
</style>
