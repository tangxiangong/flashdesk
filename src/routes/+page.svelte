<script lang="ts">
  import "$lib/styles/tokens.css";
  import "$lib/styles/components.css";
  import HeaderBar from "$lib/components/HeaderBar.svelte";
  import ActivityRail from "$lib/components/ActivityRail.svelte";
  import FlashView from "$lib/features/flash/FlashView.svelte";
  import MemoryView from "$lib/features/memory/MemoryView.svelte";
  import ControlView from "$lib/features/control/ControlView.svelte";
  import { theme } from "$lib/state/theme.svelte";
  import { jobs } from "$lib/state/jobs.svelte";
  import { ui } from "$lib/state/ui.svelte";

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

  <div class="body">
    <ActivityRail />

    <main class="content ui-scrollbar" aria-label="工作区">
      {#if ui.view === "flash"}
        <FlashView />
      {:else if ui.view === "memory"}
        <MemoryView />
      {:else}
        <ControlView />
      {/if}
    </main>
  </div>
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: var(--header-height) 1fr;
    height: 100dvh;
    background: var(--color-bg);
  }

  .body {
    display: flex;
    min-height: 0;
  }

  .content {
    flex: 1;
    min-width: 0;
    overflow: auto;
    padding: var(--space-6);
  }

  @media (max-width: 640px) {
    .content {
      padding: var(--space-4);
    }
  }
</style>
