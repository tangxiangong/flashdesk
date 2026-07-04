use crate::error::Result;
use crate::models::{EraseRequest, JobId, MemoryReadResult, MemoryRequest};
use crate::services;
use tauri::AppHandle;

#[tauri::command]
pub fn read_memory(request: MemoryRequest) -> Result<MemoryReadResult> {
    services::memory::read_memory(request)
}

#[tauri::command]
pub fn erase_target(app: AppHandle, request: EraseRequest) -> Result<JobId> {
    services::memory::erase_target(&app, request)
}
