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
  <div class="job" data-tone={tone} role="status" aria-live="polite">
    <div class="job-head">
      <span class={badgeClass[tone]}>
        <span
          class="ui-dot"
          class:ui-dot--pulse={tone === "progress"}
          style={`--dot-color: currentColor`}
        ></span>
        {latest ? stageLabel(latest.stage) : "排队中"}
      </span>
      {#if progress != null}
        <span class="job-percent ui-mono">{progress}%</span>
      {/if}
    </div>

    <div
      class="job-bar"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={progress ?? 0}
    >
      <span
        class="job-bar-fill"
        data-tone={tone}
        style={`width:${progress ?? (tone === "success" || tone === "danger" ? 100 : 0)}%`}
      ></span>
    </div>

    {#if latest?.message}
      <span class="job-msg">{latest.message}</span>
    {/if}
  </div>
{/if}

<style>
  .job {
    display: grid;
    gap: 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface-inset);
    padding: var(--space-3) var(--space-4);
  }

  .job[data-tone="success"] {
    border-color: var(--color-success-border);
    background: var(--color-success-soft);
  }

  .job[data-tone="danger"] {
    border-color: var(--color-danger-border);
    background: var(--color-danger-soft);
  }

  .job-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    min-width: 0;
  }

  .job-percent {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
  }

  .job-msg {
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .job-bar {
    height: 6px;
    overflow: hidden;
    border-radius: var(--radius-pill);
    background: var(--color-surface);
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

  .job-bar-fill[data-tone="neutral"] {
    background: var(--color-text-faint);
  }
</style>
