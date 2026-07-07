use crate::error::Result;
use crate::models::{FirmwareInput, FirmwareUsage, FlashRequest, JobId};
use crate::services;
use tauri::AppHandle;

/// 启动固件烧录任务并立即返回任务 ID。
#[tauri::command]
pub fn flash_firmware(app: AppHandle, request: FlashRequest) -> Result<JobId> {
    services::flash::flash_firmware(&app, request)
}

/// 分析固件文件中会写入 Flash 的地址段。
#[tauri::command]
pub fn firmware_usage(input: FirmwareInput) -> Result<FirmwareUsage> {
    services::firmware::analyze_firmware_usage(&input)
}
