import {
  listenToJobEvents,
  type JobEvent,
  type JobStage,
} from "$lib/api/tauri";

const MAX_EVENTS = 400;

/** 任务阶段在 UI 中对应的视觉语义。 */
export type StageTone = "neutral" | "progress" | "success" | "danger";

const STAGE_LABEL: Record<JobStage, string> = {
  queued: "排队中",
  connecting: "连接中",
  preparing: "准备中",
  erasing: "擦除中",
  programming: "写入中",
  verifying: "校验中",
  resetting: "复位中",
  completed: "已完成",
  failed: "失败",
};

const STAGE_TONE: Record<JobStage, StageTone> = {
  queued: "neutral",
  connecting: "progress",
  preparing: "progress",
  erasing: "progress",
  programming: "progress",
  verifying: "progress",
  resetting: "progress",
  completed: "success",
  failed: "danger",
};

/** 返回任务阶段的中文显示文案。 */
export function stageLabel(stage: JobStage): string {
  return STAGE_LABEL[stage] ?? stage;
}

/** 返回任务阶段对应的 UI 颜色语义。 */
export function stageTone(stage: JobStage): StageTone {
  return STAGE_TONE[stage] ?? "neutral";
}

/** 判断任务阶段是否已经结束。 */
export function isStageTerminal(stage: JobStage): boolean {
  return stage === "completed" || stage === "failed";
}

class JobsState {
  /** 当前保留在前端内存中的任务事件列表。 */
  events = $state<JobEvent[]>([]);
  private unlisten: (() => void) | undefined;

  /** 开始监听后端任务事件；重复调用不会注册多个监听器。 */
  async start() {
    if (this.unlisten) return;
    this.unlisten = await listenToJobEvents((event) => {
      const next = [...this.events, event];
      this.events =
        next.length > MAX_EVENTS ? next.slice(next.length - MAX_EVENTS) : next;
    });
  }

  /** 停止监听后端任务事件。 */
  stop() {
    this.unlisten?.();
    this.unlisten = undefined;
  }

  /** 返回指定任务 ID 对应的全部事件。 */
  eventsFor(jobId: string | null | undefined): JobEvent[] {
    if (!jobId) return [];
    return this.events.filter((event) => event.id === jobId);
  }

  /** 返回指定任务 ID 的最新事件。 */
  latestFor(jobId: string | null | undefined): JobEvent | undefined {
    const scoped = this.eventsFor(jobId);
    return scoped.at(-1);
  }
}

/** 全局任务事件状态。 */
export const jobs = new JobsState();
