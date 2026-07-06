mod commands;
pub mod error;
pub mod models;
pub mod services;

#[cfg(target_os = "macos")]
fn minimal_macos_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<tauri::menu::Menu<R>> {
    use tauri::menu::{Menu, PredefinedMenuItem, Submenu};

    let app_menu = Submenu::with_items(
        app,
        app.package_info().name.clone(),
        true,
        &[
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;

    Menu::with_items(app, &[&app_menu, &window_menu])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .enable_macos_default_menu(false)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    #[cfg(target_os = "macos")]
    let builder = builder.menu(minimal_macos_menu);

    builder
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
