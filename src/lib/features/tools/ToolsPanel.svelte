<script lang="ts">
  import EraseView from "$lib/features/erase/EraseView.svelte";
  import MemoryView from "$lib/features/memory/MemoryView.svelte";
</script>

<section class="tools-panel ui-panel" aria-labelledby="tools-title">
  <header class="tools-header">
    <div class="tools-title">
      <strong id="tools-title">高级功能</strong>
      <span>擦除芯片、查看内存数据</span>
    </div>

    <EraseView />
  </header>

  <div class="tools-content">
    <MemoryView />
  </div>
</section>

<style>
  .tools-panel {
    display: grid;
    gap: var(--space-4);
    width: 100%;
    padding: var(--space-5);
  }

  .tools-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: var(--space-4);
  }

  .tools-title {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .tools-title strong {
    color: var(--color-text);
    font-size: var(--text-lg);
    font-weight: 800;
    line-height: 1.2;
  }

  .tools-title span {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .tools-content {
    display: grid;
    min-width: 0;
  }

  @media (max-width: 640px) {
    .tools-panel {
      padding: var(--space-4);
    }

    .tools-header {
      display: grid;
    }
  }
</style>
