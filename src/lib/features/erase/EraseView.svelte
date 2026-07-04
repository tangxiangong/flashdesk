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

<div class="erase-action">
  <ConfirmButton
    label="擦除"
    confirmLabel="确认擦除"
    disabled={!target.ready || eraseRunning}
    onconfirm={() => void doErase()}
  />

  <JobProgress jobId={eraseJobId} />

  {#if eraseError}
    <p class="erase-error">
      <Icon src={alertIcon} size={14} />{eraseError}
    </p>
  {/if}
</div>

<style>
  .erase-action {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: var(--space-2);
    width: auto;
    min-width: 0;
  }

  .erase-action :global(.confirm-btn) {
    min-width: 88px;
    min-height: 32px;
    border-radius: var(--radius-sm);
    font-weight: 900;
  }

  .erase-action :global(.job) {
    flex: 1;
    min-width: 0;
    max-width: 420px;
  }

  .erase-error {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    margin: 0;
    color: var(--color-danger);
    font-size: var(--text-xs);
  }

  @media (max-width: 640px) {
    .erase-action {
      align-items: stretch;
      flex-direction: column;
    }

    .erase-action :global(.job) {
      max-width: none;
    }
  }
</style>
