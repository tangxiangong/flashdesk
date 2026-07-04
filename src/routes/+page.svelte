<script lang="ts">
  import "$lib/styles/tokens.css";
  import "$lib/styles/components.css";
  import HeaderBar from "$lib/components/HeaderBar.svelte";
  import FlashView from "$lib/features/flash/FlashView.svelte";
  import ChipInfo from "$lib/features/target/ChipInfo.svelte";
  import { theme } from "$lib/state/theme.svelte";
  import { jobs } from "$lib/state/jobs.svelte";
  import { target } from "$lib/state/target.svelte";

  let chipPanelOpen = $state(false);

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

  <main class="content ui-scrollbar" aria-label="烧录工作台">
    <div class="workspace">
      <div class="programmer-console">
        <section class="flash-pane" aria-label="固件烧录">
          <FlashView />
        </section>
      </div>
    </div>
  </main>

  {#if target.connected}
    <aside
      class="chip-float"
      class:collapsed={!chipPanelOpen}
      aria-label="芯片信息"
    >
      <button
        type="button"
        class="chip-toggle"
        aria-expanded={chipPanelOpen}
        onclick={() => (chipPanelOpen = !chipPanelOpen)}
      >
        芯片
      </button>

      {#if chipPanelOpen}
        <ChipInfo />
      {/if}
    </aside>
  {/if}
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: var(--header-height) 1fr;
    height: 100dvh;
    background: var(--color-bg);
  }

  .content {
    min-height: 0;
    overflow: auto;
    padding: var(--space-2);
  }

  .workspace {
    display: grid;
    justify-items: stretch;
    gap: var(--space-3);
    width: 100%;
    margin: 0;
  }

  .programmer-console {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-3);
    align-items: stretch;
    min-width: 0;
  }

  .flash-pane {
    min-width: 0;
  }

  .chip-float {
    position: fixed;
    top: calc(var(--header-height) + var(--space-3));
    right: var(--space-3);
    z-index: 30;
    display: grid;
    gap: 8px;
    width: min(300px, calc(100vw - 24px));
    max-height: calc(100dvh - var(--header-height) - 24px);
  }

  .chip-float.collapsed {
    width: auto;
  }

  .chip-toggle {
    justify-self: end;
    min-height: 32px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    color: var(--color-text);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 900;
    padding: 0 12px;
    box-shadow: var(--shadow-pop);
  }

  .chip-toggle:hover {
    border-color: var(--color-border-strong);
    background: var(--color-surface-muted);
  }

  @media (max-width: 360px) {
    .chip-float {
      left: var(--space-3);
      right: var(--space-3);
      width: auto;
    }
  }

  @media (max-width: 640px) {
    .content {
      padding: var(--space-3);
    }
  }
</style>
