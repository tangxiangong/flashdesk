use crate::error::Result;
use crate::models::{EraseRequest, JobId, MemoryReadResult, MemoryRequest};
use crate::services;
use tauri::AppHandle;

/// 同步读取目标内存并返回十六进制字符串。
#[tauri::command]
pub fn read_memory(request: MemoryRequest) -> Result<MemoryReadResult> {
    services::memory::read_memory(request)
}

/// 启动整片擦除任务并立即返回任务 ID。
#[tauri::command]
pub fn erase_target(app: AppHandle, request: EraseRequest) -> Result<JobId> {
    services::memory::erase_target(&app, request)
}
