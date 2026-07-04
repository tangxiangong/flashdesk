<script lang="ts">
  import { jobs, stageLabel, stageTone } from "$lib/state/jobs.svelte";

  let { jobId }: { jobId: string | null } = $props();

  let latest = $derived(jobs.latestFor(jobId));
  let tone = $derived(latest ? stageTone(latest.stage) : "neutral");
  let progress = $derived(
    latest?.progress != null ? Math.round(latest.progress * 100) : null,
  );

  const badgeClass: Record<string, string> = {
    neutral: "ui-badge",
    progress: "ui-badge ui-badge--accent",
    success: "ui-badge ui-badge--success",
    danger: "ui-badge ui-badge--danger",
  };
</script>

{#if jobId}
  <div class="job" role="status" aria-live="polite">
    <div class="job-head">
      <span class={badgeClass[tone]}>
        <span
          class="ui-dot"
          class:ui-dot--pulse={tone === "progress"}
          style={`--dot-color: currentColor`}
        ></span>
        {latest ? stageLabel(latest.stage) : "排队中"}
      </span>
      {#if latest?.message}
        <span class="job-msg">{latest.message}</span>
      {/if}
    </div>

    {#if progress != null}
      <div
        class="job-bar"
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={progress}
      >
        <span class="job-bar-fill" data-tone={tone} style={`width:${progress}%`}
        ></span>
      </div>
    {/if}
  </div>
{/if}

<style>
  .job {
    display: grid;
    gap: var(--space-2);
  }

  .job-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .job-msg {
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .job-bar {
    height: 6px;
    overflow: hidden;
    border-radius: var(--radius-pill);
    background: var(--color-surface-inset);
  }

  .job-bar-fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--color-accent);
    transition: width var(--duration-slow) var(--ease-out);
  }

  .job-bar-fill[data-tone="success"] {
    background: var(--color-success);
  }

  .job-bar-fill[data-tone="danger"] {
    background: var(--color-danger);
  }
</style>
