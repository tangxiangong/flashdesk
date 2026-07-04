use crate::error::Result;
use crate::models::{ConnectRequest, ConnectionInfo, MemoryRegionLayout, ProbeSummary};
use crate::services;

#[tauri::command]
pub fn list_probes() -> Result<Vec<ProbeSummary>> {
    services::probe::list_probes()
}

#[tauri::command]
pub fn connect_target(request: ConnectRequest) -> Result<ConnectionInfo> {
    services::probe::connect_target(request)
}

#[tauri::command]
pub fn search_chips(query: String, limit: Option<usize>) -> Result<Vec<String>> {
    services::target::search_chips(&query, limit.unwrap_or(20))
}

#[tauri::command]
pub fn target_memory_map(chip: String) -> Result<Vec<MemoryRegionLayout>> {
    services::target::target_memory_map(&chip)
}
