mod commands;
pub mod error;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .enable_macos_default_menu(false)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::probe::list_probes,
            commands::probe::connect_target,
            commands::probe::search_chips,
            commands::probe::target_memory_map,
            commands::flash::flash_firmware,
            commands::memory::read_memory,
            commands::memory::erase_target,
            commands::storage::load_profiles,
            commands::storage::save_profiles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
