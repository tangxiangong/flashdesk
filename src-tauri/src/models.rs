use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirmwareFormat {
    Elf,
    Hex,
    Bin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareInput {
    pub path: String,
    pub format: Option<FirmwareFormat>,
    pub base_address: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeSummary {
    pub identifier: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSelection {
    pub chip: Option<String>,
    pub protocol: WireProtocol,
    pub speed_khz: Option<u32>,
    pub connect_under_reset: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireProtocol {
    Swd,
    Jtag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashOptions {
    pub verify: bool,
    pub dry_run: bool,
    pub skip_erase: bool,
    pub allow_erase_all: bool,
    pub reset_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashRequest {
    pub firmware: FirmwareInput,
    pub probe: Option<String>,
    pub target: TargetSelection,
    pub options: FlashOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRequest {
    pub probe: Option<String>,
    pub target: TargetSelection,
    pub address: u64,
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EraseRequest {
    pub probe: Option<String>,
    pub target: TargetSelection,
    pub range: Option<EraseRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EraseRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryReadResult {
    pub address: u64,
    pub length: u32,
    pub data_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Flash,
    Erase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStage {
    Queued,
    Connecting,
    Preparing,
    Erasing,
    Programming,
    Verifying,
    Resetting,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobEvent {
    pub id: JobId,
    pub kind: JobKind,
    pub stage: JobStage,
    pub progress: Option<f32>,
    pub message: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub probe: Option<String>,
    pub target: TargetSelection,
    pub flash_options: FlashOptions,
    pub bin_base_address: Option<u64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFile {
    pub path: String,
    pub format: FirmwareFormat,
    pub base_address: Option<u64>,
    pub used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: Uuid,
    pub kind: JobKind,
    pub target: Option<String>,
    pub firmware: Option<String>,
    pub success: bool,
    pub summary: String,
    pub log_path: String,
    pub at: DateTime<Utc>,
}
