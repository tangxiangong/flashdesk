use crate::{
    error::{AppError, Result},
    models::{JobEvent, JobId, JobKind, JobStage},
};
use chrono::Utc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

/// 生成新的后台任务 ID。
pub fn new_job_id() -> JobId {
    JobId(Uuid::new_v4())
}

/// 向前端广播后台任务进度事件。
pub fn emit_job_event(
    app: &AppHandle,
    id: &JobId,
    kind: JobKind,
    stage: JobStage,
    progress: Option<f32>,
    message: impl Into<String>,
) -> Result<()> {
    let event = JobEvent {
        id: id.clone(),
        kind,
        stage,
        progress,
        message: message.into(),
        at: Utc::now(),
    };

    app.emit("job_event", event)
        .map_err(|err| AppError::ProbeRsFailure {
            detail: err.to_string(),
        })
}
