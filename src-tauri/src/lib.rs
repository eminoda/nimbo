mod commands;
mod editors;
mod git;
mod models;
mod process;
mod storage;

use models::NimboSettings;
use process::AppState;
use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示 Nimbo Terminal", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "停止所有服务", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出 Nimbo Terminal", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &stop, &quit])?;
    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Nimbo Terminal")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "stop" => process::stop_all(app, &app.state::<AppState>()),
            "quit" => {
                process::stop_all(app, &app.state::<AppState>());
                app.exit(0);
            }
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_settings,
            commands::settings_file_path,
            commands::list_package_scripts,
            commands::list_project_branches,
            commands::get_current_branch,
            commands::switch_project_branch,
            commands::list_installed_editors,
            commands::open_project_in_editor,
            commands::save_project,
            commands::delete_project,
            commands::select_project,
            commands::reorder_projects,
            commands::select_command,
            commands::set_close_to_tray,
            commands::list_sessions,
            commands::start_command,
            commands::open_terminal,
            commands::write_terminal,
            commands::resize_terminal,
            commands::stop_session,
            commands::reload_session,
        ])
        .setup(|app| {
            let settings = storage::load(app.handle()).unwrap_or_else(|error| {
                eprintln!("Failed to load Nimbo settings: {error}");
                NimboSettings::default()
            });
            *app.state::<AppState>()
                .settings
                .lock()
                .expect("settings lock") = settings;
            build_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                let close_to_tray = state
                    .settings
                    .lock()
                    .map(|settings| settings.close_to_tray)
                    .unwrap_or(true);
                if close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    process::stop_all(&window.app_handle(), &state);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Nimbo Terminal");
}
