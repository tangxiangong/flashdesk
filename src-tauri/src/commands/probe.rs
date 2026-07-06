use crate::error::Result;
use crate::models::{ConnectRequest, ConnectionInfo, MemoryRegionLayout, ProbeSummary};
use crate::services;

/// 列出当前系统可见的调试探针。
#[tauri::command]
pub fn list_probes() -> Result<Vec<ProbeSummary>> {
    services::probe::list_probes()
}

/// 连接目标芯片并返回实际连接信息。
#[tauri::command]
pub fn connect_target(request: ConnectRequest) -> Result<ConnectionInfo> {
    services::probe::connect_target(request)
}

/// 根据关键字搜索 probe-rs 内置芯片型号。
#[tauri::command]
pub fn search_chips(query: String, limit: Option<usize>) -> Result<Vec<String>> {
    services::target::search_chips(&query, limit.unwrap_or(20))
}

/// 读取指定芯片型号的内存布局。
#[tauri::command]
pub fn target_memory_map(chip: String) -> Result<Vec<MemoryRegionLayout>> {
    services::target::target_memory_map(&chip)
}
