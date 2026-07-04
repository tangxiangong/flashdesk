<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import JobProgress from "$lib/components/JobProgress.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import eraseIcon from "$lib/assets/icons/erase.svg?url";
  import { eraseTarget } from "$lib/api/tauri";
  import { parseAddressInput } from "$lib/utils/address";
  import { target } from "$lib/state/target.svelte";
  import { jobs, isStageTerminal } from "$lib/state/jobs.svelte";

  type EraseScope = "full" | "range";

  let eraseScope = $state<EraseScope>("full");
  let eraseStart = $state("0x08000000");
  let eraseEnd = $state("0x08010000");
  let eraseJobId = $state<string | null>(null);
  let eraseError = $state<string | null>(null);

  let parsedEraseStart = $derived(parseAddressInput(eraseStart));
  let parsedEraseEnd = $derived(parseAddressInput(eraseEnd));
  let rangeInvalid = $derived(
    eraseScope === "range" &&
      (parsedEraseStart == null ||
        parsedEraseEnd == null ||
        parsedEraseStart >= parsedEraseEnd),
  );
  let eraseLatest = $derived(jobs.latestFor(eraseJobId));
  let eraseRunning = $derived(
    eraseJobId != null && !isStageTerminal(eraseLatest?.stage ?? "queued"),
  );

  function readableError(err: unknown): string {
    if (typeof err === "string") return err;
    if (err && typeof err === "object" && "message" in err) {
      return String((err as { message: unknown }).message);
    }
    return "擦除任务提交失败";
  }

  async function doErase() {
    eraseError = null;
    eraseJobId = null;

    if (rangeInvalid) {
      eraseError = "擦除范围必须使用有效地址，且结束地址必须大于起始地址";
      return;
    }

    try {
      eraseJobId = await eraseTarget({
        probe: target.probe,
        target: target.selection(),
        range:
          eraseScope === "range"
            ? { start: parsedEraseStart!, end: parsedEraseEnd! }
            : null,
      });
    } catch (err) {
      eraseError = readableError(err);
    }
  }
</script>

<section class="view" aria-labelledby="erase-title">
  <header class="view-head">
    <div>
      <h1 id="erase-title">擦除芯片</h1>
    </div>
  </header>

  {#if !target.ready}
    <p class="ui-callout ui-callout--warning">
      <Icon src={alertIcon} size={14} />请先在顶部选择目标芯片
    </p>
  {/if}

  <div class="ui-panel section danger-zone">
    <div class="danger-head">
      <h2><Icon src={eraseIcon} size={16} />擦除范围</h2>
      <span class="ui-badge ui-badge--danger">不可撤销</span>
    </div>

    <div class="scope">
      <Segmented
        value={eraseScope}
        options={[
          { value: "full" as EraseScope, label: "整片擦除" },
          { value: "range" as EraseScope, label: "指定范围" },
        ]}
        onchange={(v) => (eraseScope = v)}
      />
    </div>

    {#if eraseScope === "range"}
      <div class="ui-grid-2">
        <label class="ui-field">
          <span>起始地址</span>
          <input
            class="ui-input ui-mono"
            bind:value={eraseStart}
            autocomplete="off"
          />
        </label>
        <label class="ui-field">
          <span>结束地址</span>
          <input
            class="ui-input ui-mono"
            bind:value={eraseEnd}
            autocomplete="off"
          />
        </label>
      </div>

      {#if rangeInvalid}
        <p class="ui-callout ui-callout--danger">
          <Icon
            src={alertIcon}
            size={14}
          />擦除范围必须使用有效地址，且结束地址必须大于起始地址
        </p>
      {/if}
    {/if}

    <ConfirmButton
      label={eraseScope === "full" ? "整片擦除" : "擦除指定范围"}
      confirmLabel="再次点击以确认擦除"
      disabled={!target.ready || eraseRunning || rangeInvalid}
      onconfirm={() => void doErase()}
    />

    <JobProgress jobId={eraseJobId} />

    {#if eraseError}
      <p class="ui-callout ui-callout--danger">
        <Icon src={alertIcon} size={14} />{eraseError}
      </p>
    {/if}
  </div>
</section>

<style>
  .view {
    display: grid;
    gap: var(--space-5);
    max-width: 720px;
  }

  .view-head h1 {
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-xl);
  }

  .section {
    display: grid;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .danger-zone {
    border-color: color-mix(in srgb, var(--color-danger) 38%, transparent);
  }

  .danger-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .danger-head h2 {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    color: var(--color-text);
    font-size: var(--text-base);
    font-weight: 700;
  }

  .scope :global(.segmented) {
    width: 100%;
  }

  .scope :global(.segment) {
    flex: 1;
  }
</style>
