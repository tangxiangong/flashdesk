use crate::error::Result;
use crate::models::{FlashRequest, JobId};
use crate::services;
use tauri::AppHandle;

/// 启动固件烧录任务并立即返回任务 ID。
#[tauri::command]
pub fn flash_firmware(app: AppHandle, request: FlashRequest) -> Result<JobId> {
    services::flash::flash_firmware(&app, request)
}
