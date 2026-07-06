<script lang="ts">
  import "$lib/styles/tokens.css";
  import "$lib/styles/components.css";
  import HeaderBar from "$lib/components/HeaderBar.svelte";
  import ConnectionPanel from "$lib/features/target/ConnectionPanel.svelte";
  import FlashView from "$lib/features/flash/FlashView.svelte";
  import ToolsPanel from "$lib/features/tools/ToolsPanel.svelte";
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
  <ConnectionPanel />

  <main class="content ui-scrollbar" aria-label="烧录工作台">
    <div class="workspace">
      <FlashView />
      <ToolsPanel />
    </div>
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
    min-height: 0;
    overflow: auto;
    padding: var(--space-8) var(--space-5) var(--space-8);
  }

  .workspace {
    display: grid;
    gap: var(--space-6);
    width: 100%;
  }

  @media (max-width: 640px) {
    .content {
      padding: var(--space-5) var(--space-3) var(--space-8);
    }
  }
</style>
