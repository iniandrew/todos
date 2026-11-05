mod backup;
mod commands;
mod db;
mod error;
mod models;
mod reminders;

use std::sync::Mutex;

use tauri::Manager;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            let conn = db::open(&dir.join("todos.db"))?;
            app.manage(AppState {
                db: Mutex::new(conn),
                data_dir: dir,
            });
            reminders::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_lists,
            commands::create_list,
            commands::update_list,
            commands::delete_list,
            commands::reorder_lists,
            commands::get_tasks,
            commands::search_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::restore_task,
            commands::reorder_tasks,
            commands::get_counts,
            commands::add_step,
            commands::update_step,
            commands::delete_step,
            commands::promote_step,
            commands::get_settings,
            commands::set_setting,
            commands::export_backup,
            commands::export_csv,
            commands::preview_backup,
            commands::import_backup,
            commands::list_auto_backups,
            commands::open_data_folder,
            commands::update_mode,
            commands::open_release_page,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Todos");
}
