use crate::error::Result;
use crate::models::{FlashRequest, JobId};
use crate::services;
use tauri::AppHandle;

#[tauri::command]
pub fn flash_firmware(app: AppHandle, request: FlashRequest) -> Result<JobId> {
    services::flash::flash_firmware(&app, request)
}
