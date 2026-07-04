use crate::error::Result;
use crate::models::{
    DumpMemoryRequest, EraseRequest, JobId, MemoryReadResult, MemoryRequest, TargetActionRequest,
    TargetStatus, WriteMemoryRequest,
};
use crate::services;
use tauri::AppHandle;

#[tauri::command]
pub fn read_memory(request: MemoryRequest) -> Result<MemoryReadResult> {
    services::memory::read_memory(request)
}

#[tauri::command]
pub fn write_memory(app: AppHandle, request: WriteMemoryRequest) -> Result<JobId> {
    services::memory::write_memory(&app, request)
}

#[tauri::command]
pub fn dump_memory(app: AppHandle, request: DumpMemoryRequest) -> Result<JobId> {
    services::memory::dump_memory(&app, request)
}

#[tauri::command]
pub fn erase_target(app: AppHandle, request: EraseRequest) -> Result<JobId> {
    services::memory::erase_target(&app, request)
}

#[tauri::command]
pub fn reset_target(app: AppHandle, request: TargetActionRequest) -> Result<JobId> {
    services::memory::reset_target(&app, request)
}

#[tauri::command]
pub fn attach_target(request: TargetActionRequest) -> Result<TargetStatus> {
    services::memory::attach_target(request)
}
