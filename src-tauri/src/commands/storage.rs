use crate::error::{AppError, Result};
use crate::models::Profile;
use crate::services::storage::{
    StoragePaths, load_profiles as load_profiles_from_storage,
    save_profiles as save_profiles_to_storage,
};
use tauri::{AppHandle, Manager};

/// 从应用数据目录加载用户保存的烧录配置。
#[tauri::command]
pub fn load_profiles(app: AppHandle) -> Result<Vec<Profile>> {
    let paths = storage_paths(&app)?;
    load_profiles_from_storage(&paths)
}

/// 将用户保存的烧录配置写入应用数据目录。
#[tauri::command]
pub fn save_profiles(app: AppHandle, profiles: Vec<Profile>) -> Result<()> {
    let paths = storage_paths(&app)?;
    save_profiles_to_storage(&paths, &profiles)
}

/// 基于 Tauri 应用数据目录构造本应用的持久化路径集合。
fn storage_paths(app: &AppHandle) -> Result<StoragePaths> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| AppError::StorageFailure {
            detail: err.to_string(),
        })?;
    Ok(StoragePaths::new(dir))
}
