<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import { jobs, isStageTerminal, stageLabel } from "$lib/state/jobs.svelte";
  import { appStatus } from "$lib/state/status.svelte";
  import { target } from "$lib/state/target.svelte";
  import type { JobEvent } from "$lib/api/tauri";
  import alertIcon from "$lib/assets/icons/alert.svg?url";
  import checkIcon from "$lib/assets/icons/check.svg?url";
  import flashIcon from "$lib/assets/icons/flash.svg?url";
  import infoIcon from "$lib/assets/icons/info.svg?url";
  import targetIcon from "$lib/assets/icons/target.svg?url";
  import xIcon from "$lib/assets/icons/x.svg?url";

  type StripTone = "info" | "progress" | "success" | "danger";

  interface StripStatus {
    tone: StripTone;
    icon: string;
    label: string;
    message: string;
    progress: number | null;
    pulse: boolean;
    dismissible: boolean;
  }

  let latestJob = $derived(jobs.events.at(-1));
  let dismissedJobId = $state<string | null>(null);
  let status = $derived(resolveStatus(latestJob));

  function resolveStatus(latest: JobEvent | undefined): StripStatus {
    if (target.connectError) {
      return {
        tone: "danger",
        icon: alertIcon,
        label: "连接错误",
        message: target.connectError,
        progress: null,
        pulse: false,
        dismissible: false,
      };
    }

    if (target.probesError) {
      return {
        tone: "danger",
        icon: alertIcon,
        label: "探针错误",
        message: target.probesError,
        progress: null,
        pulse: false,
        dismissible: false,
      };
    }

    if (target.targetCandidates.length > 0) {
      const information = target.targetInformation;
      const identity = information
        ? [
            information.deviceType,
            information.deviceId == null
              ? null
              : `Device ID 0x${information.deviceId.toString(16).toUpperCase()}`,
            information.cpu,
          ]
            .filter(Boolean)
            .join(" · ")
        : "已读取调试接口信息";
      return {
        tone: "info",
        icon: targetIcon,
        label: "已识别芯片系列",
        message: `${identity}；检测到 ${target.targetCandidates.length} 个兼容型号，请确认具体容量/型号`,
        progress: null,
        pulse: false,
        dismissible: false,
      };
    }

    if (target.connecting) {
      return {
        tone: "progress",
        icon: targetIcon,
        label: "连接中",
        message: "正在打开探针并识别目标",
        progress: null,
        pulse: true,
        dismissible: false,
      };
    }

    if (target.probesLoading) {
      return {
        tone: "progress",
        icon: targetIcon,
        label: "扫描探针",
        message: "正在刷新本机可用调试探针",
        progress: null,
        pulse: true,
        dismissible: false,
      };
    }

    if (appStatus.current) {
      return {
        tone: appStatus.current.tone,
        icon:
          appStatus.current.tone === "danger"
            ? alertIcon
            : appStatus.current.tone === "success"
              ? checkIcon
              : appStatus.current.tone === "progress"
                ? flashIcon
                : infoIcon,
        label: appStatus.current.label,
        message: appStatus.current.message,
        progress: appStatus.current.progress,
        pulse: appStatus.current.pulse,
        dismissible: true,
      };
    }

    if (latest && latest.id !== dismissedJobId) {
      const terminal = isStageTerminal(latest.stage);
      const progress =
        latest.progress == null ? null : Math.round(latest.progress * 100);
      const failed = latest.stage === "failed";
      const completed = latest.stage === "completed";

      if (!terminal || failed || completed) {
        return {
          tone: failed ? "danger" : completed ? "success" : "progress",
          icon: failed ? alertIcon : completed ? checkIcon : flashIcon,
          label: stageLabel(latest.stage),
          message: latest.message,
          progress:
            progress ??
            (failed || completed ? 100 : latest.progress == null ? null : 0),
          pulse: !terminal,
          dismissible: terminal,
        };
      }
    }

    if (target.connected) {
      return {
        tone: "success",
        icon: checkIcon,
        label: "目标已连接",
        message: `${target.effectiveChip || "已识别目标"} · ${target.protocol.toUpperCase()} · ${target.speedKhz} kHz`,
        progress: null,
        pulse: false,
        dismissible: false,
      };
    }

    return {
      tone: "info",
      icon: infoIcon,
      label: "准备连接",
      message: "选择探针后连接；芯片会自动识别，失败时再从候选中选择",
      progress: null,
      pulse: false,
      dismissible: false,
    };
  }

  function dismissStatus() {
    if (appStatus.current) {
      appStatus.clear();
      return;
    }

    const latest = latestJob;
    if (latest && isStageTerminal(latest.stage)) {
      dismissedJobId = latest.id;
    }
  }
</script>

<div
  class="status-strip"
  data-tone={status.tone}
  role="status"
  aria-live="polite"
  title={status.message}
>
  <div class="status-main">
    <span class="status-icon" class:pulse={status.pulse}>
      <Icon src={status.icon} size={14} />
    </span>
    <strong>{status.label}</strong>
    <span class="status-message">{status.message}</span>
  </div>

  {#if status.progress != null}
    <span class="status-percent ui-mono">{status.progress}%</span>
  {/if}

  {#if status.dismissible}
    <button
      type="button"
      class="status-close"
      aria-label="关闭提示"
      title="关闭提示"
      onclick={dismissStatus}
    >
      <Icon src={xIcon} size={12} />
    </button>
  {/if}

  {#if status.progress != null || status.pulse}
    <div class="status-progress" aria-hidden="true">
      <span
        class="status-progress-fill"
        style={`width:${status.progress ?? 32}%`}
      ></span>
    </div>
  {/if}
</div>

<style>
  .status-strip {
    position: absolute;
    top: 0;
    right: calc(var(--space-5) + 28px);
    left: var(--space-5);
    z-index: 2;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    height: var(--status-strip-height, 40px);
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    min-width: 0;
  }

  .status-main {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .status-icon {
    display: grid;
    flex-shrink: 0;
    place-items: center;
    color: var(--color-text-faint);
  }

  .status-icon.pulse {
    animation: status-pulse 1.2s var(--ease-out) infinite;
  }

  .status-strip strong {
    flex-shrink: 0;
    color: var(--color-text);
    font-weight: 800;
  }

  .status-message {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-percent {
    flex-shrink: 0;
    color: var(--color-text-faint);
    font-size: var(--text-2xs);
    font-weight: 600;
  }

  .status-close {
    display: grid;
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    place-items: center;
    border: 0;
    border-radius: var(--radius-pill);
    background: transparent;
    color: currentColor;
    cursor: pointer;
    opacity: 0.62;
    padding: 0;
    transition:
      background var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }

  .status-close:hover {
    background: color-mix(in srgb, currentColor 10%, transparent);
    opacity: 1;
  }

  .status-progress {
    position: absolute;
    right: 0;
    bottom: 4px;
    left: 0;
    height: 2px;
    overflow: hidden;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--color-border) 58%, transparent);
  }

  .status-progress-fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: currentColor;
    transition: width var(--duration-slow) var(--ease-out);
  }

  .status-strip[data-tone="progress"] {
    color: var(--color-accent-strong);
  }

  .status-strip[data-tone="success"] {
    color: var(--color-success);
  }

  .status-strip[data-tone="danger"] {
    top: 4px;
    height: calc(var(--status-strip-height, 40px) - 8px);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-sm);
    background: var(--color-danger-soft);
    box-shadow:
      inset 3px 0 0 var(--color-danger),
      0 4px 14px color-mix(in srgb, var(--color-danger) 12%, transparent);
    color: var(--color-danger);
    padding: 0 var(--space-3);
  }

  .status-strip[data-tone="danger"] strong {
    color: var(--color-danger);
  }

  .status-strip[data-tone="progress"] .status-icon,
  .status-strip[data-tone="success"] .status-icon,
  .status-strip[data-tone="danger"] .status-icon {
    color: currentColor;
  }

  @keyframes status-pulse {
    0%,
    100% {
      opacity: 0.55;
      transform: scale(0.94);
    }

    50% {
      opacity: 1;
      transform: scale(1);
    }
  }

  @media (max-width: 640px) {
    .status-strip {
      right: calc(var(--space-3) + 24px);
      left: var(--space-3);
    }
  }
</style>
