use crate::error::{AppError, Result};
use crate::models::{JobEvent, JobId, JobKind, JobStage};
use chrono::Utc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

pub fn new_job_id() -> JobId {
    JobId(Uuid::new_v4())
}

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
