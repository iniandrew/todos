//! Thin `#[tauri::command]` wrappers around [`crate::db`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::backup::{self, BackupFile, BackupSummary, ImportMode};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{Counts, ListPatch, NewTask, Task, TaskList, TaskPatch, View};

pub struct AppState {
    pub db: Mutex<Connection>,
    /// Folder holding `todos.db` and `backups/`.
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        // A poisoned mutex only means another command panicked mid-way;
        // SQLite itself is still consistent, so keep going.
        self.db.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// The user's local calendar date. The frontend sends its own idea of
/// "today" so that My Day always matches what the user sees on screen.
fn today(today: Option<String>) -> String {
    today.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string())
}

#[tauri::command]
pub fn get_lists(state: State<AppState>) -> AppResult<Vec<TaskList>> {
    db::get_lists(&state.conn())
}

#[tauri::command]
pub fn create_list(
    state: State<AppState>,
    name: String,
    theme: Option<String>,
) -> AppResult<TaskList> {
    db::create_list(&state.conn(), &name, theme.as_deref())
}

#[tauri::command]
pub fn update_list(state: State<AppState>, id: String, patch: ListPatch) -> AppResult<TaskList> {
    db::update_list(&state.conn(), &id, &patch)
}

#[tauri::command]
pub fn delete_list(state: State<AppState>, id: String) -> AppResult<()> {
    db::delete_list(&state.conn(), &id)
}

#[tauri::command]
pub fn reorder_lists(state: State<AppState>, ids: Vec<String>) -> AppResult<()> {
    db::reorder_lists(&mut state.conn(), &ids)
}

#[tauri::command]
pub fn get_tasks(
    state: State<AppState>,
    view: String,
    today: Option<String>,
) -> AppResult<Vec<Task>> {
    let view = View::from_str(&view).map_err(AppError::Invalid)?;
    db::get_tasks(&state.conn(), &view, &self::today(today))
}

#[tauri::command]
pub fn search_tasks(state: State<AppState>, query: String) -> AppResult<Vec<Task>> {
    db::search_tasks(&state.conn(), &query)
}

#[tauri::command]
pub fn create_task(
    state: State<AppState>,
    input: NewTask,
    today: Option<String>,
) -> AppResult<Task> {
    db::create_task(&state.conn(), &input, &self::today(today))
}

#[tauri::command]
pub fn update_task(
    state: State<AppState>,
    id: String,
    patch: TaskPatch,
    today: Option<String>,
) -> AppResult<Task> {
    db::update_task(&mut state.conn(), &id, &patch, &self::today(today))
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: String) -> AppResult<()> {
    db::delete_task(&state.conn(), &id)
}

#[tauri::command]
pub fn restore_task(state: State<AppState>, task: Task) -> AppResult<Task> {
    db::restore_task(&mut state.conn(), &task)
}

#[tauri::command]
pub fn reorder_tasks(state: State<AppState>, ids: Vec<String>) -> AppResult<()> {
    db::reorder_tasks(&mut state.conn(), &ids)
}

#[tauri::command]
pub fn get_counts(state: State<AppState>, today: Option<String>) -> AppResult<Counts> {
    db::get_counts(&state.conn(), &self::today(today))
}

#[tauri::command]
pub fn add_step(state: State<AppState>, task_id: String, title: String) -> AppResult<Task> {
    db::add_step(&state.conn(), &task_id, &title)
}

#[tauri::command]
pub fn update_step(
    state: State<AppState>,
    id: String,
    title: Option<String>,
    completed: Option<bool>,
) -> AppResult<Task> {
    db::update_step(&state.conn(), &id, title.as_deref(), completed)
}

#[tauri::command]
pub fn delete_step(state: State<AppState>, id: String) -> AppResult<Task> {
    db::delete_step(&state.conn(), &id)
}

#[tauri::command]
pub fn promote_step(state: State<AppState>, id: String, today: Option<String>) -> AppResult<Task> {
    db::promote_step(&mut state.conn(), &id, &self::today(today))
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppResult<HashMap<String, String>> {
    db::get_settings(&state.conn())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> AppResult<()> {
    db::set_setting(&state.conn(), &key, &value)
}

// ---------------------------------------------------------------------------
// Backup & restore
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn export_backup(state: State<AppState>, path: String) -> AppResult<BackupSummary> {
    backup::write_file(&state.conn(), Path::new(&path))
}

#[tauri::command]
pub fn export_csv(state: State<AppState>, path: String) -> AppResult<()> {
    let csv = backup::to_csv(&state.conn())?;
    std::fs::write(path, csv)?;
    Ok(())
}

#[tauri::command]
pub fn preview_backup(path: String) -> AppResult<BackupSummary> {
    Ok(backup::summary(&backup::read_file(Path::new(&path))?))
}

#[tauri::command]
pub fn import_backup(
    state: State<AppState>,
    path: String,
    mode: ImportMode,
) -> AppResult<BackupSummary> {
    let data = backup::read_file(Path::new(&path))?;
    let mut conn = state.conn();
    if mode == ImportMode::Replace {
        backup::safety_backup(&conn, &state.data_dir)?;
    }
    backup::import(&mut conn, &data, mode, chrono::Utc::now())
}

#[tauri::command]
pub fn list_auto_backups(state: State<AppState>) -> AppResult<Vec<BackupFile>> {
    backup::list_backups(&state.data_dir)
}

#[tauri::command]
pub fn open_data_folder(app: AppHandle, state: State<AppState>) -> AppResult<()> {
    app.opener()
        .open_path(state.data_dir.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::Invalid(e.to_string()))
}

// ---------------------------------------------------------------------------
// Updates
// ---------------------------------------------------------------------------

/// How this build can be updated.
///
/// `inApp`: the updater can replace the app (Windows, macOS, AppImage).
/// `notify`: installed through a package manager (.deb, .rpm, Arch), so we
/// only tell the user about the new version.
#[tauri::command]
pub fn update_mode() -> &'static str {
    if cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_none() {
        "notify"
    } else {
        "inApp"
    }
}

/// Opens the GitHub release page for `version` (e.g. "0.3.0") in the browser.
#[tauri::command]
pub fn open_release_page(app: AppHandle, version: String) -> AppResult<()> {
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(AppError::Invalid(format!("invalid version: {version}")));
    }
    let url = format!("https://github.com/iniandrew/todos/releases/tag/v{version}");
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Invalid(e.to_string()))
}
