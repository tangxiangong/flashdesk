<script lang="ts">
  import type { Snippet } from "svelte";

  /** Popover 组件 props。 */
  interface Props {
    /** 是否打开弹层，支持双向绑定。 */
    open?: boolean;
    /** 弹层相对触发器的水平对齐方式。 */
    align?: "start" | "end";
    /** 弹层宽度，单位 px。 */
    width?: number;
    /** 触发器片段，接收 toggle 和 open 状态。 */
    trigger: Snippet<[{ toggle: () => void; open: boolean }]>;
    /** 弹层内容片段，接收 close 回调。 */
    content: Snippet<[{ close: () => void }]>;
  }

  let {
    open = $bindable(false),
    align = "start",
    width = 320,
    trigger,
    content,
  }: Props = $props();

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
    border-radius: var(--radius-md);
    background: var(--color-surface);
    box-shadow: var(--shadow-pop);
    padding: 10px;
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
