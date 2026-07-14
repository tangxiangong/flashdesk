const MAX_EVENTS = 200;

/** 运行日志中的操作类型。 */
export type ActivityKind =
  "connect" | "probe" | "target" | "flash" | "erase" | "memory" | "storage";

/** 与后台任务阶段兼容的运行日志阶段。 */
export type ActivityStage =
  | "queued"
  | "connecting"
  | "preparing"
  | "erasing"
  | "programming"
  | "verifying"
  | "resetting"
  | "completed"
  | "failed";

/** 日志页统一展示的运行事件。 */
export interface ActivityEvent {
  id: string;
  kind: ActivityKind;
  stage: ActivityStage;
  progress?: number | null;
  message: string;
  at: string;
}

class ActivityLogState {
  /** 当前运行期间由前端产生的操作事件。 */
  events = $state<ActivityEvent[]>([]);

  /** 追加一条事件，并限制内存中保留的事件数量。 */
  append(event: ActivityEvent) {
    const next = [...this.events, event];
    this.events =
      next.length > MAX_EVENTS ? next.slice(next.length - MAX_EVENTS) : next;
  }

  /** 记录一次前台操作的阶段。 */
  operation(
    id: string,
    kind: ActivityKind,
    stage: ActivityStage,
    message: string,
  ) {
    this.append({
      id,
      kind,
      stage,
      progress: stage === "completed" ? 1 : null,
      message,
      at: new Date().toISOString(),
    });
  }
}

/** 全局前台操作日志。 */
export const activityLog = new ActivityLogState();
