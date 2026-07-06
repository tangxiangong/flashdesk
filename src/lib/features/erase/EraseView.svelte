<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import JobProgress from "$lib/components/JobProgress.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import { eraseTarget, readableError } from "$lib/api/tauri";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

  let eraseJobId = $state<string | null>(null);
  let eraseError = $state<string | null>(null);

  let eraseLatest = $derived(jobs.latestFor(eraseJobId));
  let eraseRunning = $derived(
    eraseJobId != null && !isStageTerminal(eraseLatest?.stage ?? "queued"),
  );

  async function doErase() {
    eraseError = null;
    eraseJobId = null;

    try {
      eraseJobId = await eraseTarget({
        probe: target.probe,
        target: target.selection(),
      });
    } catch (err) {
      eraseError = readableError(err, "擦除失败");
    }
  }
</script>

<div class="erase-tool">
  <p class="erase-warning">
    <Icon src={alertIcon} size={14} />
    此操作会清除芯片上的全部固件数据，且无法撤销。
  </p>

  <div class="erase-action">
    <ConfirmButton
      label="擦除整片芯片"
      confirmLabel="确认擦除，无法撤销"
      disabled={!target.ready || eraseRunning}
      onconfirm={() => void doErase()}
    />
  </div>

  <JobProgress jobId={eraseJobId} />

  {#if eraseError}
    <p class="erase-error">
      <Icon src={alertIcon} size={14} />{eraseError}
    </p>
  {/if}

  {#if !target.ready}
    <p class="erase-hint">先连接设备后才能执行擦除。</p>
  {/if}
</div>

<style>
  .erase-tool {
    display: grid;
    gap: var(--space-3);
  }

  .erase-warning {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin: 0;
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-md);
    background: var(--color-danger-soft);
    color: var(--color-danger);
    font-size: var(--text-sm);
    line-height: 1.5;
    padding: var(--space-3) var(--space-4);
  }

  .erase-warning :global(.icon) {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .erase-action :global(.confirm-btn) {
    min-width: 160px;
  }

  .erase-error {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    margin: 0;
    color: var(--color-danger);
    font-size: var(--text-xs);
  }

  .erase-hint {
    margin: 0;
    color: var(--color-text-faint);
    font-size: var(--text-xs);
  }
</style>
