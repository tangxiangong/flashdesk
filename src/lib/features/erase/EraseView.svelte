<script lang="ts">
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import { eraseTarget, readableError } from "$lib/api/tauri";
  import { appStatus } from "$lib/state/status.svelte";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

  let eraseJobId = $state<string | null>(null);

  let eraseLatest = $derived(jobs.latestFor(eraseJobId));
  let eraseRunning = $derived(
    eraseJobId != null && !isStageTerminal(eraseLatest?.stage ?? "queued"),
  );

  async function doErase() {
    appStatus.clear();
    eraseJobId = null;

    try {
      eraseJobId = await eraseTarget({
        probe: target.probe,
        target: target.selection(),
      });
    } catch (err) {
      appStatus.danger("擦除失败", readableError(err, "擦除失败"));
    }
  }
</script>

<div class="erase-tool">
  <ConfirmButton
    label="擦除整片芯片"
    confirmLabel="确认擦除"
    disabled={!target.ready || eraseRunning}
    onconfirm={() => void doErase()}
  />
</div>

<style>
  .erase-tool {
    display: inline-grid;
    align-self: start;
    justify-items: end;
    gap: var(--space-2);
  }
</style>
