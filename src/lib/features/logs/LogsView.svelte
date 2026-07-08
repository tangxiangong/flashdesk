<script lang="ts">
  import { jobs, stageLabel, stageTone } from "$lib/state/jobs.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import xIcon from "$lib/assets/icons/x.svg?url";
  import type { JobEvent, JobKind } from "$lib/api/tauri";

  type KindFilter = "all" | JobKind;

  const KIND_LABEL: Record<JobKind, string> = {
    flash: "烧录",
    erase: "擦除",
  };

  const BADGE_CLASS: Record<string, string> = {
    neutral: "ui-badge",
    progress: "ui-badge ui-badge--accent",
    success: "ui-badge ui-badge--success",
    danger: "ui-badge ui-badge--danger",
  };

  let filter = $state<KindFilter>("all");
  let clearedAt = $state(0);
  let consoleEl = $state<HTMLDivElement | undefined>();
  let pinnedToBottom = $state(true);
  let copyFeedback = $state(false);

  let visibleEvents = $derived(
    jobs.events.filter((event) => {
      if (Date.parse(event.at) <= clearedAt) return false;
      if (filter !== "all" && event.kind !== filter) return false;
      return true;
    }),
  );

  $effect(() => {
    void visibleEvents.length;
    if (!pinnedToBottom || !consoleEl) return;
    const target = consoleEl;
    requestAnimationFrame(() => {
      target.scrollTo({ top: target.scrollHeight });
    });
  });

  function handleScroll() {
    if (!consoleEl) return;
    const distanceFromBottom =
      consoleEl.scrollHeight - consoleEl.scrollTop - consoleEl.clientHeight;
    pinnedToBottom = distanceFromBottom < 48;
  }

  function jumpToLatest() {
    pinnedToBottom = true;
    consoleEl?.scrollTo({ top: consoleEl.scrollHeight, behavior: "smooth" });
  }

  function clearLog() {
    clearedAt = Date.now();
    pinnedToBottom = true;
  }

  function formatTime(iso: string): string {
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return iso;
    const time = date.toLocaleTimeString("zh-CN", { hour12: false });
    return `${time}.${String(date.getMilliseconds()).padStart(3, "0")}`;
  }

  function formatLine(event: JobEvent): string {
    const percent =
      event.progress != null ? ` (${Math.round(event.progress * 100)}%)` : "";
    return `[${formatTime(event.at)}] ${KIND_LABEL[event.kind]}·${stageLabel(event.stage)}${percent} — ${event.message}`;
  }

  async function copyLog() {
    const text = visibleEvents.map(formatLine).join("\n");
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copyFeedback = true;
      setTimeout(() => {
        copyFeedback = false;
      }, 1600);
    } catch {
      // 剪贴板不可用时静默忽略。
    }
  }
</script>

<section class="logs-panel ui-panel" aria-labelledby="logs-title">
  <header class="logs-header">
    <div class="logs-title">
      <strong id="logs-title">运行日志</strong>
      <span>仅在当前应用运行期间保留，不写入磁盘，关闭应用后自动清空</span>
    </div>

    <div class="logs-actions">
      <Segmented
        value={filter}
        options={[
          { value: "all", label: "全部" },
          { value: "flash", label: "烧录" },
          { value: "erase", label: "擦除" },
        ]}
        onchange={(value) => (filter = value)}
      />
      <button
        type="button"
        class="ui-btn ui-btn--ghost"
        onclick={copyLog}
        disabled={visibleEvents.length === 0}
      >
        {copyFeedback ? "已复制" : "复制"}
      </button>
      <button
        type="button"
        class="ui-btn ui-btn--ghost ui-btn--icon"
        onclick={clearLog}
        disabled={visibleEvents.length === 0}
        aria-label="清空日志"
        title="清空日志"
      >
        <Icon src={xIcon} size={16} />
      </button>
    </div>
  </header>

  <div class="logs-console-wrap">
    <div
      class="logs-console ui-scrollbar"
      role="log"
      aria-live="polite"
      bind:this={consoleEl}
      onscroll={handleScroll}
    >
      {#if visibleEvents.length === 0}
        <p class="logs-empty">
          暂无任务事件。开始一次烧录或擦除后，进度会实时显示在这里。
        </p>
      {:else}
        {#each visibleEvents as event, index (event.id + event.at + index)}
          <div class="logs-line" data-tone={stageTone(event.stage)}>
            <span class="logs-time ui-mono">{formatTime(event.at)}</span>
            <span class="logs-job ui-mono" title={`任务 ${event.id}`}
              >{event.id.slice(0, 8)}</span
            >
            <span class={BADGE_CLASS[stageTone(event.stage)]}>
              <span
                class="ui-dot"
                class:ui-dot--pulse={stageTone(event.stage) === "progress"}
                style="--dot-color: currentColor"
              ></span>
              {KIND_LABEL[event.kind]}·{stageLabel(event.stage)}
            </span>
            {#if event.progress != null}
              <span class="logs-percent ui-mono"
                >{Math.round(event.progress * 100)}%</span
              >
            {/if}
            <span class="logs-message">{event.message}</span>
          </div>
        {/each}
      {/if}
    </div>

    {#if !pinnedToBottom && visibleEvents.length > 0}
      <button type="button" class="logs-jump ui-mono" onclick={jumpToLatest}>
        ↓ 回到最新
      </button>
    {/if}
  </div>
</section>

<style>
  .logs-panel {
    display: grid;
    gap: var(--space-4);
    width: 100%;
    padding: var(--space-5);
  }

  .logs-header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: var(--space-4);
  }

  .logs-title {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .logs-title strong {
    color: var(--color-text);
    font-size: var(--text-lg);
    font-weight: 800;
    line-height: 1.2;
  }

  .logs-title span {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .logs-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .logs-console-wrap {
    position: relative;
    min-width: 0;
  }

  .logs-console {
    display: grid;
    align-content: start;
    gap: 2px;
    height: min(56vh, 560px);
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-console-bg);
    padding: var(--space-3);
  }

  .logs-empty {
    margin: auto;
    color: var(--color-text-faint);
    font-size: var(--text-sm);
    text-align: center;
  }

  .logs-line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    border-radius: var(--radius-sm);
    padding: 5px var(--space-2);
    color: var(--color-console-text);
    font-size: var(--text-xs);
    line-height: 1.5;
  }

  .logs-line:hover {
    background: color-mix(in srgb, var(--color-console-text) 6%, transparent);
  }

  .logs-line[data-tone="danger"] {
    background: color-mix(in srgb, var(--color-danger) 14%, transparent);
  }

  .logs-time {
    flex-shrink: 0;
    color: color-mix(in srgb, var(--color-console-text) 55%, transparent);
    font-size: var(--text-2xs);
  }

  .logs-job {
    flex-shrink: 0;
    color: var(--color-console-accent);
    font-size: var(--text-2xs);
  }

  .logs-percent {
    flex-shrink: 0;
    color: color-mix(in srgb, var(--color-console-text) 70%, transparent);
  }

  .logs-message {
    flex: 1 1 240px;
    min-width: 0;
    color: var(--color-console-text);
    word-break: break-word;
  }

  .logs-jump {
    position: absolute;
    bottom: var(--space-3);
    left: 50%;
    transform: translateX(-50%);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    box-shadow: var(--shadow-sm);
    color: var(--color-text);
    cursor: pointer;
    font-size: var(--text-xs);
    font-weight: 600;
    padding: 6px var(--space-3);
    transition:
      background var(--duration-base) var(--ease-out),
      border-color var(--duration-base) var(--ease-out);
  }

  .logs-jump:hover {
    border-color: var(--color-accent);
    color: var(--color-accent-strong);
  }

  @media (max-width: 640px) {
    .logs-console {
      height: min(50vh, 440px);
    }
  }
</style>
