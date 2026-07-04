<script lang="ts">
  import "$lib/styles/tokens.css";
  import "$lib/styles/components.css";
  import HeaderBar from "$lib/components/HeaderBar.svelte";
  import FlashView from "$lib/features/flash/FlashView.svelte";
  import EraseView from "$lib/features/erase/EraseView.svelte";
  import MemoryView from "$lib/features/memory/MemoryView.svelte";
  import { theme } from "$lib/state/theme.svelte";
  import { jobs } from "$lib/state/jobs.svelte";

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
    <div class="workbench">
      <section class="primary-pane" aria-label="固件烧录">
        <FlashView />
      </section>

      <div class="side-stack">
        <EraseView />
        <MemoryView />
      </div>
    </div>
  </main>
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
    padding: var(--space-6);
  }

  .workbench {
    display: grid;
    grid-template-columns: minmax(420px, 1.1fr) minmax(360px, 0.9fr);
    align-items: start;
    gap: var(--space-6);
    max-width: 1320px;
    min-height: 0;
  }

  .primary-pane,
  .side-stack {
    min-width: 0;
  }

  .side-stack {
    display: grid;
    gap: var(--space-6);
  }

  .workbench :global(.view) {
    max-width: none;
  }

  @media (max-width: 640px) {
    .content {
      padding: var(--space-4);
    }
  }

  @media (max-width: 1040px) {
    .workbench {
      grid-template-columns: 1fr;
    }
  }
</style>
