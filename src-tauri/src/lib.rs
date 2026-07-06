mod commands;
/// 应用级错误类型和前端可序列化错误响应。
pub mod error;
/// Rust 与前端共享的请求、响应和状态模型。
pub mod models;
/// probe-rs、存储、任务和固件处理等后端服务层。
pub mod services;

#[cfg(target_os = "macos")]
const MENU_ABOUT_ID: &str = "app-about";
#[cfg(target_os = "macos")]
const MENU_CHECK_UPDATE_ID: &str = "app-check-update";
#[cfg(target_os = "macos")]
const MENU_EVENT_ABOUT: &str = "flashdesk://menu/about";
#[cfg(target_os = "macos")]
const MENU_EVENT_CHECK_UPDATE: &str = "flashdesk://menu/check-update";

/// 构建桌面端使用的最小 macOS 应用菜单。
///
/// 默认 Tauri 菜单已关闭，这里只保留关于、检查更新和系统窗口操作，
/// 同时继续支持 Cmd+Q、Cmd+M 等原生快捷键。
#[cfg(target_os = "macos")]
fn minimal_macos_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<tauri::menu::Menu<R>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

    let about = MenuItem::with_id(app, MENU_ABOUT_ID, "关于", true, None::<&str>)?;
    let check_update =
        MenuItem::with_id(app, MENU_CHECK_UPDATE_ID, "检查更新", true, None::<&str>)?;

    let app_menu = Submenu::with_items(
        app,
        app.package_info().name.clone(),
        true,
        &[
            &about,
            &check_update,
            &PredefinedMenuItem::separator(app)?,
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

/// 将原生 macOS 菜单选择桥接到前端窗口状态。
///
/// 更新和关于窗口仍由前端维护，Rust 只把稳定菜单 ID 映射为 Tauri 事件。
#[cfg(target_os = "macos")]
fn handle_macos_menu_event<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    event: tauri::menu::MenuEvent,
) {
    use tauri::Emitter;

    let event_name = match event.id() {
        id if id == MENU_ABOUT_ID => MENU_EVENT_ABOUT,
        id if id == MENU_CHECK_UPDATE_ID => MENU_EVENT_CHECK_UPDATE,
        _ => return,
    };

    let _ = app.emit(event_name, ());
}

/// 启动 Tauri 桌面应用。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .enable_macos_default_menu(false)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    #[cfg(target_os = "macos")]
    let builder = builder.menu(minimal_macos_menu);

    #[cfg(target_os = "macos")]
    let builder = builder.on_menu_event(handle_macos_menu_event);

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
