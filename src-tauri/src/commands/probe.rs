use crate::error::Result;
use crate::models::ProbeSummary;
use crate::services;

#[tauri::command]
pub fn list_probes() -> Result<Vec<ProbeSummary>> {
    services::probe::list_probes()
}

#[tauri::command]
pub fn search_chips(query: String, limit: Option<usize>) -> Result<Vec<String>> {
    services::target::search_chips(&query, limit.unwrap_or(20))
}
