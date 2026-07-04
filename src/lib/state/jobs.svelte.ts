import {
  listenToJobEvents,
  type JobEvent,
  type JobStage,
} from "$lib/api/tauri";

const MAX_EVENTS = 400;

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

export function stageLabel(stage: JobStage): string {
  return STAGE_LABEL[stage] ?? stage;
}

export function stageTone(stage: JobStage): StageTone {
  return STAGE_TONE[stage] ?? "neutral";
}

export function isStageTerminal(stage: JobStage): boolean {
  return stage === "completed" || stage === "failed";
}

class JobsState {
  events = $state<JobEvent[]>([]);
  private unlisten: (() => void) | undefined;

  async start() {
    if (this.unlisten) return;
    this.unlisten = await listenToJobEvents((event) => {
      const next = [...this.events, event];
      this.events =
        next.length > MAX_EVENTS ? next.slice(next.length - MAX_EVENTS) : next;
    });
  }

  stop() {
    this.unlisten?.();
    this.unlisten = undefined;
  }

  eventsFor(jobId: string | null | undefined): JobEvent[] {
    if (!jobId) return [];
    return this.events.filter((event) => event.id === jobId);
  }

  latestFor(jobId: string | null | undefined): JobEvent | undefined {
    const scoped = this.eventsFor(jobId);
    return scoped.at(-1);
  }
}

export const jobs = new JobsState();
