<script lang="ts">
  import "$lib/styles/tokens.css";
  import "$lib/styles/components.css";
  import HeaderBar from "$lib/components/HeaderBar.svelte";
  import StatusStrip from "$lib/components/StatusStrip.svelte";
  import ConnectionPanel from "$lib/features/target/ConnectionPanel.svelte";
  import FlashView from "$lib/features/flash/FlashView.svelte";
  import ToolsPanel from "$lib/features/tools/ToolsPanel.svelte";
  import { theme } from "$lib/state/theme.svelte";
  import { jobs } from "$lib/state/jobs.svelte";

  const workspacePages = [{ label: "固件烧录" }, { label: "高级功能" }];
  const pageCount = workspacePages.length;

  let currentPage = $state(0);
  let lastWheelAt = 0;

  function setPage(index: number) {
    currentPage = Math.max(0, Math.min(pageCount - 1, index));
  }

  function shouldIgnoreWheelTarget(target: EventTarget | null) {
    return (
      target instanceof HTMLElement &&
      target.closest("input, textarea, select, [contenteditable='true']") !=
        null
    );
  }

  function handleWheel(event: WheelEvent) {
    if (shouldIgnoreWheelTarget(event.target) || Math.abs(event.deltaY) < 24) {
      return;
    }

    event.preventDefault();

    const now = performance.now();
    if (now - lastWheelAt < 420) {
      return;
    }

    lastWheelAt = now;
    setPage(currentPage + (event.deltaY > 0 ? 1 : -1));
  }

  $effect(() => {
    theme.sync();
  });

  $effect(() => {
    void jobs.start();
    return () => jobs.stop();
  });
</script>

<div class="shell">
  <HeaderBar />
  <ConnectionPanel />

  <main class="content" aria-label="烧录工作台" onwheel={handleWheel}>
    <StatusStrip />

    <div
      class="workspace"
      style={`transform: translateY(-${currentPage * 100}%);`}
    >
      <section class="workspace-page" aria-label="固件烧录">
        <div class="workspace-page-inner">
          <FlashView />
        </div>
      </section>

      <section class="workspace-page" aria-label="高级功能">
        <div class="workspace-page-inner">
          <ToolsPanel />
        </div>
      </section>
    </div>

    <nav class="page-indicator" aria-label="当前页面">
      {#each workspacePages as page, index}
        <button
          type="button"
          class="page-jump"
          class:active={currentPage === index}
          aria-current={currentPage === index ? "page" : undefined}
          aria-label={`切换到${page.label}`}
          title={page.label}
          onclick={() => setPage(index)}
        ></button>
      {/each}
    </nav>
  </main>
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: var(--header-height) auto 1fr;
    height: 100dvh;
    background:
      radial-gradient(
        circle at 50% 0%,
        color-mix(in srgb, var(--color-accent) 5%, transparent),
        transparent 60%
      ),
      var(--texture-dots) 0 0 / 18px 18px,
      var(--color-bg);
  }

  .content {
    position: relative;
    --status-strip-height: var(--space-8);
    min-height: 0;
    overflow: hidden;
  }

  .workspace {
    display: grid;
    grid-template-rows: repeat(2, 100%);
    width: 100%;
    height: 100%;
    transition: transform var(--duration-slow) var(--ease-out);
    will-change: transform;
  }

  .workspace-page {
    min-height: 0;
    overflow: auto;
    padding: var(--status-strip-height) calc(var(--space-5) + 28px)
      var(--space-8) var(--space-5);
  }

  .workspace-page-inner {
    display: grid;
    align-content: start;
    width: 100%;
    min-height: 100%;
  }

  .page-indicator {
    position: absolute;
    top: 50%;
    right: var(--space-4);
    z-index: 2;
    display: grid;
    gap: 7px;
    transform: translateY(-50%);
  }

  .page-jump {
    width: 6px;
    height: 36px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    background: var(--color-surface-muted);
    cursor: pointer;
    padding: 0;
    transition:
      background var(--duration-base) var(--ease-out),
      border-color var(--duration-base) var(--ease-out),
      height var(--duration-base) var(--ease-out);
  }

  .page-jump:hover {
    border-color: var(--color-border-strong);
    background: var(--color-border-strong);
  }

  .page-jump.active {
    border-color: var(--color-accent);
    background: var(--color-accent);
    height: 44px;
  }

  @media (max-width: 640px) {
    .workspace-page {
      padding: var(--status-strip-height) calc(var(--space-3) + 24px)
        var(--space-8) var(--space-3);
    }

    .page-indicator {
      right: var(--space-2);
    }
  }
</style>
