use crate::{
    error::{AppError, Result},
    models::{HistoryEntry, Profile, RecentFile},
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// 应用本地存储文件路径集合。
#[derive(Debug, Clone)]
pub struct StoragePaths {
    root: PathBuf,
}

impl StoragePaths {
    /// 使用指定根目录创建存储路径集合。
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// 用户烧录配置 JSON 文件路径。
    pub fn profiles(&self) -> PathBuf {
        self.root.join("profiles.json")
    }

    /// 最近文件 JSON 文件路径。
    pub fn recent_files(&self) -> PathBuf {
        self.root.join("recent-files.json")
    }

    /// 历史记录 JSONL 文件路径。
    pub fn history(&self) -> PathBuf {
        self.root.join("history.jsonl")
    }

    /// 任务日志目录路径。
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }
}

/// 确保存储根目录和日志目录存在。
pub fn ensure_storage(paths: &StoragePaths) -> Result<()> {
    fs::create_dir_all(&paths.root).map_err(|err| storage_io_error("create storage root", err))?;
    fs::create_dir_all(paths.logs_dir()).map_err(|err| storage_io_error("create logs dir", err))?;
    Ok(())
}

/// 读取用户保存的烧录配置列表。
pub fn load_profiles(paths: &StoragePaths) -> Result<Vec<Profile>> {
    read_json_array(&paths.profiles())
}

/// 覆盖写入用户保存的烧录配置列表。
pub fn save_profiles(paths: &StoragePaths, profiles: &[Profile]) -> Result<()> {
    write_json_pretty(&paths.profiles(), profiles)
}

/// 读取最近使用的固件文件列表。
pub fn load_recent_files(paths: &StoragePaths) -> Result<Vec<RecentFile>> {
    read_json_array(&paths.recent_files())
}

/// 覆盖写入最近使用的固件文件列表。
pub fn save_recent_files(paths: &StoragePaths, recent_files: &[RecentFile]) -> Result<()> {
    write_json_pretty(&paths.recent_files(), recent_files)
}

/// 追加一条任务历史记录。
pub fn append_history(paths: &StoragePaths, entry: &HistoryEntry) -> Result<()> {
    ensure_storage(paths)?;
    let serialized = serde_json::to_string(entry).map_err(|err| AppError::StorageFailure {
        detail: err.to_string(),
    })?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.history())
        .map_err(|err| storage_io_error("open history file", err))?;
    writeln!(file, "{serialized}").map_err(|err| storage_io_error("write history entry", err))?;
    Ok(())
}

/// 写入任务日志并返回日志文件路径。
pub fn write_job_log(paths: &StoragePaths, job_id: &str, content: &str) -> Result<PathBuf> {
    ensure_storage(paths)?;
    let filename = sanitize_filename::sanitize(format!("{job_id}.log"));
    let path = paths.logs_dir().join(filename);
    fs::write(&path, content).map_err(|err| storage_io_error("write job log", err))?;
    Ok(path)
}

fn read_json_array<T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let exists = path
        .try_exists()
        .map_err(|err| storage_io_error("检查 JSON 文件是否存在", err))?;
    if !exists {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(path).map_err(|err| storage_io_error("read json array", err))?;
    serde_json::from_str(&content).map_err(|err| AppError::StorageFailure {
        detail: err.to_string(),
    })
}

fn write_json_pretty<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize + ?Sized,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| storage_io_error("create json parent dir", err))?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|err| AppError::StorageFailure {
        detail: err.to_string(),
    })?;
    fs::write(path, content).map_err(|err| storage_io_error("write json file", err))?;
    Ok(())
}

fn storage_io_error(operation: &str, err: std::io::Error) -> AppError {
    AppError::StorageFailure {
        detail: format!("{operation}: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FirmwareFormat, FlashOptions, JobKind, TargetSelection, WireProtocol};
    use chrono::Utc;
    use std::fs;
    use uuid::Uuid;

    fn paths() -> StoragePaths {
        StoragePaths::new(
            std::env::temp_dir().join(format!("flashdesk-storage-test-{}", Uuid::new_v4())),
        )
    }

    fn profile() -> Profile {
        Profile {
            id: Uuid::new_v4(),
            name: "STM32 default".to_string(),
            probe: None,
            target: TargetSelection {
                chip: Some("STM32F103C8".to_string()),
                protocol: WireProtocol::Swd,
                speed_khz: Some(4000),
                connect_under_reset: false,
            },
            flash_options: FlashOptions {
                verify: true,
                dry_run: false,
                skip_erase: false,
                allow_erase_all: false,
                reset_after: true,
            },
            bin_base_address: Some(0x0800_0000),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn save_profiles_should_round_trip_profiles_json() {
        let paths = paths();
        let expected = vec![profile()];
        save_profiles(&paths, &expected).unwrap();
        let actual = load_profiles(&paths).unwrap();
        assert_eq!(actual[0].name, expected[0].name);
    }

    #[test]
    fn append_history_should_write_one_json_line() {
        let paths = paths();
        let entry = HistoryEntry {
            id: Uuid::new_v4(),
            kind: JobKind::Flash,
            target: Some("STM32F103C8".to_string()),
            firmware: Some("/tmp/app.elf".to_string()),
            success: true,
            summary: "烧录成功".to_string(),
            log_path: "logs/job.log".to_string(),
            at: Utc::now(),
        };
        append_history(&paths, &entry).unwrap();
        let content = fs::read_to_string(paths.history()).unwrap();
        assert_eq!(content.lines().count(), 1);
        let actual: HistoryEntry = serde_json::from_str(content.lines().next().unwrap()).unwrap();
        assert_eq!(actual.summary, "烧录成功");
    }

    #[test]
    fn load_recent_files_should_return_empty_when_file_missing() {
        let paths = paths();
        let actual = load_recent_files(&paths).unwrap();
        assert!(actual.is_empty());
    }

    #[test]
    fn save_recent_files_should_round_trip_recent_files_json() {
        let paths = paths();
        let expected = vec![RecentFile {
            path: "/tmp/app.bin".to_string(),
            format: FirmwareFormat::Bin,
            base_address: Some(0),
            used_at: Utc::now(),
        }];
        save_recent_files(&paths, &expected).unwrap();
        let actual = load_recent_files(&paths).unwrap();
        assert_eq!(actual[0].path, expected[0].path);
    }

    #[test]
    fn load_profiles_should_return_storage_failure_for_invalid_json() {
        let paths = paths();
        fs::create_dir_all(paths.profiles().parent().unwrap()).unwrap();
        fs::write(paths.profiles(), "{invalid json").unwrap();
        let err = load_profiles(&paths).unwrap_err();
        assert!(matches!(err, AppError::StorageFailure { .. }));
    }

    #[test]
    fn ensure_storage_should_return_storage_failure_when_root_is_file() {
        let paths = StoragePaths::new(
            std::env::temp_dir().join(format!("flashdesk-storage-file-root-{}", Uuid::new_v4())),
        );
        fs::write(&paths.root, "not a directory").unwrap();
        let err = ensure_storage(&paths).unwrap_err();
        assert!(matches!(err, AppError::StorageFailure { .. }));
    }

    #[test]
    fn write_job_log_should_sanitize_filename_and_write_content() {
        let paths = paths();
        let logs_dir = paths.logs_dir();
        let log_path = write_job_log(&paths, "../job:1", "hello log").unwrap();
        let file_name = log_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap();
        assert_eq!(log_path.parent(), Some(logs_dir.as_path()));
        assert_ne!(file_name, "../job:1.log");
        assert!(!file_name.contains('/'));
        assert!(!file_name.contains('\\'));
        assert!(file_name.ends_with(".log"));
        assert_eq!(fs::read_to_string(log_path).unwrap(), "hello log");
    }
}
