//! Backup & restore: JSON export/import, CSV export and daily automatic backups.
//!
//! Like `db.rs`, everything works on a plain `rusqlite::Connection` so it can be
//! unit-tested against an in-memory database.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{Task, TaskList};

pub const FORMAT: &str = "todos-backup";
pub const VERSION: u32 = 1;
/// Number of daily automatic backups to keep.
pub const KEEP_AUTO: usize = 7;
/// Settings that describe this device's UI state rather than the user's data.
const LOCAL_ONLY_SETTINGS: &[&str] = &["lastView"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub format: String,
    pub version: u32,
    #[serde(default)]
    pub app_version: String,
    pub exported_at: String,
    pub lists: Vec<TaskList>,
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    /// Custom lists (the built-in "Tasks" list is not counted).
    pub lists: usize,
    pub tasks: usize,
    pub steps: usize,
    pub exported_at: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportMode {
    /// Add new items and update existing ones; never deletes anything.
    Merge,
    /// Delete all current data, then load the backup.
    Replace,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    pub path: String,
    pub name: String,
    /// `auto` for daily backups, `safety` for copies made before a replace-import.
    pub kind: String,
    pub exported_at: String,
    pub tasks: usize,
}

// ---------------------------------------------------------------------------
// Export / parse
// ---------------------------------------------------------------------------

pub fn export(conn: &Connection) -> AppResult<Backup> {
    let mut settings = db::get_settings(conn)?;
    for key in LOCAL_ONLY_SETTINGS {
        settings.remove(*key);
    }
    Ok(Backup {
        format: FORMAT.into(),
        version: VERSION,
        app_version: env!("CARGO_PKG_VERSION").into(),
        exported_at: Utc::now().to_rfc3339(),
        lists: db::get_lists(conn)?,
        tasks: db::all_tasks(conn)?,
        settings,
    })
}

pub fn summary(backup: &Backup) -> BackupSummary {
    BackupSummary {
        lists: backup.lists.iter().filter(|l| !l.is_default).count(),
        tasks: backup.tasks.len(),
        steps: backup.tasks.iter().map(|t| t.steps.len()).sum(),
        exported_at: backup.exported_at.clone(),
    }
}

pub fn parse(json: &str) -> AppResult<Backup> {
    // Check the header first so unrelated JSON gets a clear message.
    #[derive(Deserialize)]
    struct Header {
        format: Option<String>,
        version: Option<u32>,
    }
    let header: Header = serde_json::from_str(json)
        .map_err(|e| AppError::Invalid(format!("not a Todos backup file: {e}")))?;
    if header.format.as_deref() != Some(FORMAT) {
        return Err(AppError::Invalid("not a Todos backup file".into()));
    }
    match header.version {
        Some(v) if v <= VERSION => {}
        Some(v) => return Err(AppError::Invalid(format!(
            "this backup was made by a newer version of Todos (format v{v}); please update the app"
        ))),
        None => return Err(AppError::Invalid("backup has no version".into())),
    }
    serde_json::from_str(json).map_err(|e| AppError::Invalid(format!("damaged backup file: {e}")))
}

pub fn read_file(path: &Path) -> AppResult<Backup> {
    parse(&fs::read_to_string(path)?)
}

pub fn write_file(conn: &Connection, path: &Path) -> AppResult<BackupSummary> {
    let backup = export(conn)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    // Write to a temp file first so a crash never leaves a half-written backup.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(&backup)?)?;
    fs::rename(&tmp, path)?;
    Ok(summary(&backup))
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

pub fn import(
    conn: &mut Connection,
    backup: &Backup,
    mode: ImportMode,
    now: DateTime<Utc>,
) -> AppResult<BackupSummary> {
    let tx = conn.transaction()?;
    let local_default = db::default_list_id(&tx)?;

    if mode == ImportMode::Replace {
        tx.execute("DELETE FROM tasks", [])?;
        tx.execute("DELETE FROM lists WHERE is_default = 0", [])?;
    }

    // Backup list id -> local list id.
    let mut list_ids: HashMap<&str, String> = HashMap::new();
    for list in &backup.lists {
        if list.is_default {
            // Every database has exactly one built-in list; keep ours, take its style.
            tx.execute(
                "UPDATE lists SET theme = ?1, icon = ?2 WHERE id = ?3",
                params![list.theme, list.icon, local_default],
            )?;
            list_ids.insert(&list.id, local_default.clone());
        } else {
            tx.execute(
                "INSERT INTO lists (id, name, icon, theme, sort_order, is_default, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name, icon = excluded.icon, theme = excluded.theme,
                    sort_order = excluded.sort_order",
                params![
                    list.id,
                    list.name,
                    list.icon,
                    list.theme,
                    list.sort_order,
                    list.created_at
                ],
            )?;
            list_ids.insert(&list.id, list.id.clone());
        }
    }

    for task in &backup.tasks {
        let list_id = match list_ids.get(task.list_id.as_str()) {
            Some(id) => id.clone(),
            // A list we already have locally (merge), otherwise the built-in list.
            None => match db::get_list(&tx, &task.list_id) {
                Ok(l) => l.id,
                Err(_) => local_default.clone(),
            },
        };
        // Don't fire a burst of notifications for reminders that are already past.
        let fired = task
            .reminder_at
            .as_deref()
            .and_then(|r| DateTime::parse_from_rfc3339(r).ok())
            .is_some_and(|r| r.with_timezone(&Utc) <= now);
        db::upsert_task(&tx, task, &list_id, fired)?;
    }

    for (key, value) in &backup.settings {
        if !LOCAL_ONLY_SETTINGS.contains(&key.as_str()) {
            db::set_setting(&tx, key, value)?;
        }
    }

    tx.commit()?;
    Ok(summary(backup))
}

// ---------------------------------------------------------------------------
// CSV export
// ---------------------------------------------------------------------------

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn to_csv(conn: &Connection) -> AppResult<String> {
    let names: HashMap<String, String> = db::get_lists(conn)?
        .into_iter()
        .map(|l| (l.id, l.name))
        .collect();
    let yes_no = |b: bool| if b { "yes" } else { "no" };

    // UTF-8 BOM so Excel detects the encoding.
    let mut out = String::from("\u{feff}");
    out.push_str(
        "List,Title,Completed,Important,Due date,Reminder,My Day,Notes,Steps,Created,Completed at\r\n",
    );
    for t in db::all_tasks(conn)? {
        let steps = t
            .steps
            .iter()
            .map(|s| format!("[{}] {}", if s.completed { "x" } else { " " }, s.title))
            .collect::<Vec<_>>()
            .join(" | ");
        let row = [
            names.get(&t.list_id).cloned().unwrap_or_default(),
            t.title.clone(),
            yes_no(t.completed).into(),
            yes_no(t.important).into(),
            t.due_date.clone().unwrap_or_default(),
            t.reminder_at.clone().unwrap_or_default(),
            t.my_day_date.clone().unwrap_or_default(),
            t.notes.clone(),
            steps,
            t.created_at.clone(),
            t.completed_at.clone().unwrap_or_default(),
        ];
        out.push_str(
            &row.iter()
                .map(|f| csv_field(f))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push_str("\r\n");
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Automatic & safety backups
// ---------------------------------------------------------------------------

pub fn backups_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

/// Writes today's automatic backup if it doesn't exist yet and prunes old ones.
pub fn auto_backup(conn: &Connection, data_dir: &Path, today: &str) -> AppResult<Option<PathBuf>> {
    let dir = backups_dir(data_dir);
    let path = dir.join(format!("auto-{today}.json"));
    if path.exists() {
        return Ok(None);
    }
    write_file(conn, &path)?;
    prune_auto(&dir, KEEP_AUTO)?;
    Ok(Some(path))
}

fn prune_auto(dir: &Path, keep: usize) -> AppResult<()> {
    let mut names: Vec<String> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("auto-") && n.ends_with(".json"))
        .collect();
    // `auto-YYYY-MM-DD.json` sorts chronologically.
    names.sort_unstable_by(|a, b| b.cmp(a));
    for old in names.into_iter().skip(keep) {
        fs::remove_file(dir.join(old))?;
    }
    Ok(())
}

/// Copy of the current data, written before a replace-import.
pub fn safety_backup(conn: &Connection, data_dir: &Path) -> AppResult<PathBuf> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = backups_dir(data_dir).join(format!("before-import-{stamp}.json"));
    write_file(conn, &path)?;
    Ok(path)
}

/// Automatic and safety backups, newest first.
pub fn list_backups(data_dir: &Path) -> AppResult<Vec<BackupFile>> {
    let dir = backups_dir(data_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = if name.starts_with("auto-") {
            "auto"
        } else if name.starts_with("before-import-") {
            "safety"
        } else {
            continue;
        };
        if !name.ends_with(".json") {
            continue;
        }
        // Skip unreadable files instead of failing the whole list.
        let Ok(backup) = read_file(&entry.path()) else {
            continue;
        };
        files.push(BackupFile {
            path: entry.path().to_string_lossy().into_owned(),
            name,
            kind: kind.into(),
            exported_at: backup.exported_at.clone(),
            tasks: backup.tasks.len(),
        });
    }
    files.sort_by(|a, b| b.exported_at.cmp(&a.exported_at));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NewTask, TaskPatch, View};

    const TODAY: &str = "2026-11-05";

    fn sample() -> Connection {
        let mut conn = db::open_in_memory().unwrap();
        let work = db::create_list(&conn, "Work", Some("indigo")).unwrap();
        let a = db::create_task(
            &conn,
            &NewTask {
                list_id: Some(work.id.clone()),
                title: "Ship v1".into(),
                important: true,
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        db::add_step(&conn, &a.id, "Write notes").unwrap();
        let b = db::create_task(
            &conn,
            &NewTask {
                title: "Call mom".into(),
                my_day: true,
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        db::update_task(
            &mut conn,
            &b.id,
            &TaskPatch {
                notes: Some("Ask about \"dinner\", Sunday\nat 7".into()),
                reminder_at: Some(Some("2026-11-05T09:00:00Z".into())),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        db::set_setting(&conn, "colorScheme", "dark").unwrap();
        db::set_setting(&conn, "lastView", "important").unwrap();
        conn
    }

    fn titles(conn: &Connection) -> Vec<String> {
        let mut t: Vec<_> = db::all_tasks(conn)
            .unwrap()
            .into_iter()
            .map(|t| t.title)
            .collect();
        t.sort();
        t
    }

    fn json(conn: &Connection) -> String {
        serde_json::to_string(&export(conn).unwrap()).unwrap()
    }

    #[test]
    fn export_contents() {
        let conn = sample();
        let backup = export(&conn).unwrap();
        assert_eq!(backup.format, FORMAT);
        assert_eq!(backup.version, VERSION);
        let s = summary(&backup);
        assert_eq!((s.lists, s.tasks, s.steps), (1, 2, 1));
        assert_eq!(backup.settings.get("colorScheme").unwrap(), "dark");
        assert!(!backup.settings.contains_key("lastView"));
    }

    #[test]
    fn replace_roundtrip_into_fresh_db() {
        let src = sample();
        let backup = parse(&json(&src)).unwrap();

        let mut dst = db::open_in_memory().unwrap();
        db::create_task(
            &dst,
            &NewTask {
                title: "Will be removed".into(),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        db::create_list(&dst, "Old list", None).unwrap();

        import(&mut dst, &backup, ImportMode::Replace, Utc::now()).unwrap();
        assert_eq!(titles(&dst), ["Call mom", "Ship v1"]);
        let lists = db::get_lists(&dst).unwrap();
        let names: Vec<_> = lists.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Tasks", "Work"]);
        // Tasks from the backup's built-in list land in this DB's built-in list.
        let default_tasks = db::get_tasks(&dst, &View::Tasks, TODAY).unwrap();
        assert_eq!(default_tasks.len(), 1);
        assert_eq!(default_tasks[0].title, "Call mom");
        assert_eq!(default_tasks[0].notes, "Ask about \"dinner\", Sunday\nat 7");
        assert_eq!(
            db::all_tasks(&dst)
                .unwrap()
                .iter()
                .map(|t| t.steps.len())
                .sum::<usize>(),
            1
        );
        assert_eq!(db::get_settings(&dst).unwrap()["colorScheme"], "dark");
    }

    #[test]
    fn merge_twice_does_not_duplicate_or_delete() {
        let src = sample();
        let backup = parse(&json(&src)).unwrap();
        let mut dst = db::open_in_memory().unwrap();
        db::create_task(
            &dst,
            &NewTask {
                title: "Local only".into(),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        import(&mut dst, &backup, ImportMode::Merge, Utc::now()).unwrap();
        import(&mut dst, &backup, ImportMode::Merge, Utc::now()).unwrap();
        assert_eq!(titles(&dst), ["Call mom", "Local only", "Ship v1"]);
        assert_eq!(db::get_lists(&dst).unwrap().len(), 2);
        let steps: i64 = dst
            .query_row("SELECT COUNT(*) FROM steps", [], |r| r.get(0))
            .unwrap();
        assert_eq!(steps, 1);
    }

    #[test]
    fn merge_keeps_local_tasks_in_reimported_list() {
        // Re-importing a list must not cascade-delete tasks added to it locally.
        let mut conn = sample();
        let backup = parse(&json(&conn)).unwrap();
        let work = db::get_lists(&conn)
            .unwrap()
            .into_iter()
            .find(|l| !l.is_default)
            .unwrap();
        db::create_task(
            &conn,
            &NewTask {
                list_id: Some(work.id.clone()),
                title: "Added later".into(),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        import(&mut conn, &backup, ImportMode::Merge, Utc::now()).unwrap();
        assert_eq!(
            db::get_tasks(&conn, &View::List(work.id), TODAY)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn past_reminders_are_not_refired() {
        let src = sample();
        let backup = parse(&json(&src)).unwrap();
        let mut dst = db::open_in_memory().unwrap();
        let after = DateTime::parse_from_rfc3339("2026-11-06T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        import(&mut dst, &backup, ImportMode::Replace, after).unwrap();
        assert!(db::take_due_reminders(&dst, after).unwrap().is_empty());

        let mut dst = db::open_in_memory().unwrap();
        let before = DateTime::parse_from_rfc3339("2026-11-05T08:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        import(&mut dst, &backup, ImportMode::Replace, before).unwrap();
        assert_eq!(db::take_due_reminders(&dst, after).unwrap().len(), 1);
    }

    #[test]
    fn rejects_foreign_or_newer_files() {
        assert!(parse("not json").is_err());
        assert!(parse(r#"{"hello": "world"}"#).is_err());
        assert!(parse(r#"{"format": "todos-backup"}"#).is_err());
        let err = parse(r#"{"format": "todos-backup", "version": 99}"#).unwrap_err();
        assert!(err.to_string().contains("newer version"));
        assert!(parse(r#"{"format": "todos-backup", "version": 1, "lists": 3}"#).is_err());
    }

    #[test]
    fn csv_escaping() {
        let conn = sample();
        let csv = to_csv(&conn).unwrap();
        assert!(csv.starts_with('\u{feff}'));
        let mut lines = csv.trim_start_matches('\u{feff}').split("\r\n");
        assert!(lines.next().unwrap().starts_with("List,Title,Completed"));
        assert!(csv.contains("Work,Ship v1,no,yes"));
        assert!(csv.contains("[ ] Write notes"));
        assert!(csv.contains("\"Ask about \"\"dinner\"\", Sunday\nat 7\""));
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
    }

    #[test]
    fn auto_backup_once_per_day_and_pruned() {
        let conn = sample();
        let dir = std::env::temp_dir().join(format!("todos-test-{}", uuid::Uuid::new_v4()));
        assert!(auto_backup(&conn, &dir, "2026-11-05").unwrap().is_some());
        assert!(auto_backup(&conn, &dir, "2026-11-05").unwrap().is_none());
        for day in 6..=14 {
            auto_backup(&conn, &dir, &format!("2026-11-{day:02}")).unwrap();
        }
        safety_backup(&conn, &dir).unwrap();
        let files = list_backups(&dir).unwrap();
        let autos: Vec<_> = files.iter().filter(|f| f.kind == "auto").collect();
        assert_eq!(autos.len(), KEEP_AUTO);
        assert!(autos
            .iter()
            .all(|f| f.name.as_str() >= "auto-2026-11-08.json"));
        assert_eq!(files.iter().filter(|f| f.kind == "safety").count(), 1);
        assert!(files.iter().all(|f| f.tasks == 2));
        let restored = read_file(Path::new(&autos[0].path)).unwrap();
        assert_eq!(restored.tasks.len(), 2);
        fs::remove_dir_all(&dir).unwrap();
    }
}
