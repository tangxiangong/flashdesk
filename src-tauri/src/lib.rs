mod commands;
pub mod error;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::probe::list_probes,
            commands::probe::search_chips,
            commands::flash::flash_firmware,
            commands::memory::read_memory,
            commands::memory::write_memory,
            commands::memory::dump_memory,
            commands::memory::erase_target,
            commands::memory::reset_target,
            commands::memory::attach_target,
            commands::storage::load_profiles,
            commands::storage::save_profiles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
