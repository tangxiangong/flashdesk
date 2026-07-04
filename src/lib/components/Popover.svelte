<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    align = "start",
    width = 320,
    trigger,
    content,
  }: {
    open?: boolean;
    align?: "start" | "end";
    width?: number;
    trigger: Snippet<[{ toggle: () => void; open: boolean }]>;
    content: Snippet<[{ close: () => void }]>;
  } = $props();

  let root: HTMLDivElement | undefined;

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function onWindowPointerDown(event: PointerEvent) {
    if (open && root && !root.contains(event.target as Node)) {
      close();
    }
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && open) {
      close();
    }
  }
</script>

<svelte:window
  onpointerdown={onWindowPointerDown}
  onkeydown={onWindowKeydown}
/>

<div class="popover-root" bind:this={root}>
  {@render trigger({ toggle, open })}

  {#if open}
    <div
      class="popover-panel"
      class:align-end={align === "end"}
      style={`width:${width}px;`}
      role="dialog"
    >
      {@render content({ close })}
    </div>
  {/if}
</div>

<style>
  .popover-root {
    position: relative;
    display: inline-flex;
  }

  .popover-panel {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    z-index: 60;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    box-shadow: var(--shadow-pop);
    padding: var(--space-4);
    animation: pop-in var(--duration-base) var(--ease-out);
  }

  .popover-panel.align-end {
    left: auto;
    right: 0;
  }

  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .popover-panel {
      animation: none;
    }
  }
</style>
