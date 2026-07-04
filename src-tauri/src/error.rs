use serde::Serialize;
use std::path::Path;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ProbeNotFound,
    TargetIdentifyFailed,
    UnsupportedFirmwareFormat,
    MissingBinBaseAddress,
    InvalidFirmwareAddress,
    ProbeRsFailure,
    IoFailure,
    InvalidUserInput,
    StorageFailure,
    JobAlreadyRunning,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    /// Frontend-safe diagnostic detail. Raw probe-rs logs and filesystem internals belong in job logs, not here.
    pub detail: Option<String>,
    pub recovery: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未找到烧录器")]
    ProbeNotFound,
    #[error("无法识别目标芯片")]
    TargetIdentifyFailed { detail: String },
    #[error("不支持的固件格式")]
    UnsupportedFirmwareFormat { path: String },
    #[error("BIN 文件必须填写基地址")]
    MissingBinBaseAddress,
    #[error("固件地址无效")]
    InvalidFirmwareAddress { detail: String },
    #[error("probe-rs 操作失败")]
    ProbeRsFailure { detail: String },
    #[error("文件读写失败")]
    IoFailure(#[from] std::io::Error),
    #[error("用户输入无效")]
    InvalidUserInput { detail: String },
    #[error("本地配置存储失败")]
    StorageFailure { detail: String },
    #[error("已有任务正在运行")]
    JobAlreadyRunning,
}

impl AppError {
    pub fn to_response(&self) -> ErrorResponse {
        match self {
            Self::ProbeNotFound => ErrorResponse {
                code: ErrorCode::ProbeNotFound,
                message: self.to_string(),
                detail: None,
                recovery: "接好烧录器后重新扫描。".to_string(),
            },
            Self::TargetIdentifyFailed { detail } => ErrorResponse {
                code: ErrorCode::TargetIdentifyFailed,
                message: self.to_string(),
                detail: Some(detail.clone()),
                recovery: "尝试降低 SWD/JTAG 速度，或在芯片搜索中手动选择目标。".to_string(),
            },
            Self::UnsupportedFirmwareFormat { path } => ErrorResponse {
                code: ErrorCode::UnsupportedFirmwareFormat,
                message: self.to_string(),
                detail: Some(format!("文件名：{}", display_file_name(path))),
                recovery: "选择 .elf、.hex 或 .bin 文件。".to_string(),
            },
            Self::MissingBinBaseAddress => ErrorResponse {
                code: ErrorCode::MissingBinBaseAddress,
                message: self.to_string(),
                detail: None,
                recovery: "为 BIN 文件填写十六进制基地址，例如 0x08000000。".to_string(),
            },
            Self::InvalidFirmwareAddress { detail } => ErrorResponse {
                code: ErrorCode::InvalidFirmwareAddress,
                message: self.to_string(),
                detail: Some(detail.clone()),
                recovery: "检查地址格式、对齐和目标芯片 Flash 映射。".to_string(),
            },
            Self::ProbeRsFailure { detail } => ErrorResponse {
                code: ErrorCode::ProbeRsFailure,
                message: self.to_string(),
                detail: Some(detail.clone()),
                recovery: "确认芯片连接、接口、速度和访问地址后重试。".to_string(),
            },
            Self::IoFailure(err) => ErrorResponse {
                code: ErrorCode::IoFailure,
                message: self.to_string(),
                detail: Some(format!("文件系统错误：{}", err.kind())),
                recovery: "确认文件存在且当前用户有读写权限。".to_string(),
            },
            Self::InvalidUserInput { detail } => ErrorResponse {
                code: ErrorCode::InvalidUserInput,
                message: self.to_string(),
                detail: Some(detail.clone()),
                recovery: "按字段提示修正输入后重试。".to_string(),
            },
            Self::StorageFailure { .. } => ErrorResponse {
                code: ErrorCode::StorageFailure,
                message: self.to_string(),
                detail: Some("本地存储操作失败，完整信息见任务日志".to_string()),
                recovery: "检查应用数据目录权限，或删除损坏的本地配置文件。".to_string(),
            },
            Self::JobAlreadyRunning => ErrorResponse {
                code: ErrorCode::JobAlreadyRunning,
                message: self.to_string(),
                detail: None,
                recovery: "等待当前任务结束，或取消后再启动新任务。".to_string(),
            },
        }
    }
}

fn display_file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "所选文件".to_string())
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_response().serialize(serializer)
    }
}
