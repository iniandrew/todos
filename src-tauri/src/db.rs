//! SQLite storage layer. Every function takes a plain `rusqlite::Connection`
//! so the logic can be unit-tested against an in-memory database.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    Counts, DueReminder, ListPatch, NewTask, Step, Task, TaskList, TaskPatch, View,
};

const SCHEMA_VERSION: i32 = 1;

pub fn open(path: &Path) -> AppResult<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let conn = Connection::open(path)?;
    init(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()?;
    init(&conn)?;
    Ok(conn)
}

fn init(conn: &Connection) -> AppResult<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    migrate(conn)?;
    ensure_default_list(conn)?;
    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS lists (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                icon        TEXT,
                theme       TEXT NOT NULL DEFAULT 'blue',
                sort_order  REAL NOT NULL DEFAULT 0,
                is_default  INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tasks (
                id              TEXT PRIMARY KEY,
                list_id         TEXT NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
                title           TEXT NOT NULL,
                notes           TEXT NOT NULL DEFAULT '',
                completed       INTEGER NOT NULL DEFAULT 0,
                completed_at    TEXT,
                important       INTEGER NOT NULL DEFAULT 0,
                due_date        TEXT,
                reminder_at     TEXT,
                reminder_fired  INTEGER NOT NULL DEFAULT 0,
                my_day_date     TEXT,
                sort_order      REAL NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks(list_id);
            CREATE TABLE IF NOT EXISTS steps (
                id          TEXT PRIMARY KEY,
                task_id     TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                title       TEXT NOT NULL,
                completed   INTEGER NOT NULL DEFAULT 0,
                sort_order  REAL NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_steps_task ON steps(task_id);
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;
    }
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn ensure_default_list(conn: &Connection) -> AppResult<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM lists WHERE is_default = 1)",
        [],
        |r| r.get(0),
    )?;
    if !exists {
        conn.execute(
            "INSERT INTO lists (id, name, icon, theme, sort_order, is_default, created_at)
             VALUES (?1, 'Tasks', NULL, 'blue', 0, 1, ?2)",
            params![new_id(), now()],
        )?;
    }
    Ok(())
}

pub fn default_list_id(conn: &Connection) -> AppResult<String> {
    Ok(conn.query_row(
        "SELECT id FROM lists WHERE is_default = 1 LIMIT 1",
        [],
        |r| r.get(0),
    )?)
}

// ---------------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------------

fn row_to_list(r: &Row) -> rusqlite::Result<TaskList> {
    Ok(TaskList {
        id: r.get("id")?,
        name: r.get("name")?,
        icon: r.get("icon")?,
        theme: r.get("theme")?,
        sort_order: r.get("sort_order")?,
        is_default: r.get("is_default")?,
        created_at: r.get("created_at")?,
    })
}

pub fn get_lists(conn: &Connection) -> AppResult<Vec<TaskList>> {
    let mut stmt = conn
        .prepare("SELECT * FROM lists ORDER BY is_default DESC, sort_order ASC, created_at ASC")?;
    let lists = stmt
        .query_map([], row_to_list)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(lists)
}

pub fn get_list(conn: &Connection, id: &str) -> AppResult<TaskList> {
    conn.query_row("SELECT * FROM lists WHERE id = ?1", [id], row_to_list)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("list {id}")))
}

pub fn create_list(conn: &Connection, name: &str, theme: Option<&str>) -> AppResult<TaskList> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("list name cannot be empty".into()));
    }
    let max: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) FROM lists WHERE is_default = 0",
        [],
        |r| r.get(0),
    )?;
    let id = new_id();
    conn.execute(
        "INSERT INTO lists (id, name, icon, theme, sort_order, is_default, created_at)
         VALUES (?1, ?2, NULL, ?3, ?4, 0, ?5)",
        params![id, name, theme.unwrap_or("purple"), max + 1.0, now()],
    )?;
    get_list(conn, &id)
}

pub fn update_list(conn: &Connection, id: &str, patch: &ListPatch) -> AppResult<TaskList> {
    if let Some(name) = &patch.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Invalid("list name cannot be empty".into()));
        }
        conn.execute(
            "UPDATE lists SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
    }
    if let Some(theme) = &patch.theme {
        conn.execute(
            "UPDATE lists SET theme = ?1 WHERE id = ?2",
            params![theme, id],
        )?;
    }
    if let Some(icon) = &patch.icon {
        conn.execute(
            "UPDATE lists SET icon = ?1 WHERE id = ?2",
            params![icon, id],
        )?;
    }
    get_list(conn, id)
}

pub fn delete_list(conn: &Connection, id: &str) -> AppResult<()> {
    let list = get_list(conn, id)?;
    if list.is_default {
        return Err(AppError::Invalid(
            "the default list cannot be deleted".into(),
        ));
    }
    conn.execute("DELETE FROM lists WHERE id = ?1", [id])?;
    Ok(())
}

pub fn reorder_lists(conn: &mut Connection, ids: &[String]) -> AppResult<()> {
    let tx = conn.transaction()?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE lists SET sort_order = ?1 WHERE id = ?2",
            params![i as f64 + 1.0, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

fn row_to_task(r: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: r.get("id")?,
        list_id: r.get("list_id")?,
        title: r.get("title")?,
        notes: r.get("notes")?,
        completed: r.get("completed")?,
        completed_at: r.get("completed_at")?,
        important: r.get("important")?,
        due_date: r.get("due_date")?,
        reminder_at: r.get("reminder_at")?,
        my_day_date: r.get("my_day_date")?,
        sort_order: r.get("sort_order")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
        steps: Vec::new(),
    })
}

fn load_steps(conn: &Connection, tasks: &mut [Task]) -> AppResult<()> {
    let mut stmt =
        conn.prepare_cached("SELECT * FROM steps WHERE task_id = ?1 ORDER BY sort_order ASC")?;
    for task in tasks.iter_mut() {
        task.steps = stmt
            .query_map([&task.id], row_to_step)?
            .collect::<Result<Vec<_>, _>>()?;
    }
    Ok(())
}

fn query_tasks(
    conn: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> AppResult<Vec<Task>> {
    let mut stmt = conn.prepare(sql)?;
    let mut tasks = stmt
        .query_map(params, row_to_task)?
        .collect::<Result<Vec<_>, _>>()?;
    load_steps(conn, &mut tasks)?;
    Ok(tasks)
}

pub fn get_task(conn: &Connection, id: &str) -> AppResult<Task> {
    let mut tasks = query_tasks(conn, "SELECT * FROM tasks WHERE id = ?1", [id])?;
    tasks
        .pop()
        .ok_or_else(|| AppError::NotFound(format!("task {id}")))
}

/// Tasks for a view. `today` is the user's local date (`YYYY-MM-DD`).
pub fn get_tasks(conn: &Connection, view: &View, today: &str) -> AppResult<Vec<Task>> {
    match view {
        View::MyDay => query_tasks(
            conn,
            "SELECT * FROM tasks WHERE my_day_date = ?1 ORDER BY sort_order ASC, created_at DESC",
            [today],
        ),
        View::Important => query_tasks(
            conn,
            "SELECT * FROM tasks WHERE important = 1 ORDER BY sort_order ASC, created_at DESC",
            [],
        ),
        View::Planned => query_tasks(
            conn,
            "SELECT * FROM tasks WHERE due_date IS NOT NULL OR reminder_at IS NOT NULL
             ORDER BY COALESCE(due_date, substr(reminder_at, 1, 10)) ASC, sort_order ASC",
            [],
        ),
        View::Tasks => {
            let id = default_list_id(conn)?;
            query_tasks(
                conn,
                "SELECT * FROM tasks WHERE list_id = ?1 ORDER BY sort_order ASC, created_at DESC",
                [id],
            )
        }
        View::List(id) => query_tasks(
            conn,
            "SELECT * FROM tasks WHERE list_id = ?1 ORDER BY sort_order ASC, created_at DESC",
            [id],
        ),
    }
}

pub fn search_tasks(conn: &Connection, query: &str) -> AppResult<Vec<Task>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let escaped = q
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{escaped}%");
    query_tasks(
        conn,
        r"SELECT * FROM tasks t
          WHERE t.title LIKE ?1 ESCAPE '\'
             OR t.notes LIKE ?1 ESCAPE '\'
             OR EXISTS (SELECT 1 FROM steps s WHERE s.task_id = t.id AND s.title LIKE ?1 ESCAPE '\')
          ORDER BY t.completed ASC, t.updated_at DESC",
        [pattern],
    )
}

pub fn create_task(conn: &Connection, input: &NewTask, today: &str) -> AppResult<Task> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::Invalid("task title cannot be empty".into()));
    }
    let list_id = match &input.list_id {
        Some(id) => get_list(conn, id)?.id,
        None => default_list_id(conn)?,
    };
    // New tasks go to the top of the list, like Microsoft To Do.
    let min: f64 = conn.query_row(
        "SELECT COALESCE(MIN(sort_order), 0) FROM tasks WHERE list_id = ?1",
        [&list_id],
        |r| r.get(0),
    )?;
    let id = new_id();
    let ts = now();
    conn.execute(
        "INSERT INTO tasks (id, list_id, title, important, due_date, my_day_date, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id,
            list_id,
            title,
            input.important,
            input.due_date,
            input.my_day.then_some(today),
            min - 1.0,
            ts
        ],
    )?;
    get_task(conn, &id)
}

pub fn update_task(
    conn: &mut Connection,
    id: &str,
    patch: &TaskPatch,
    today: &str,
) -> AppResult<Task> {
    let tx = conn.transaction()?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM tasks WHERE id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound(format!("task {id}")));
    }
    if let Some(title) = &patch.title {
        let title = title.trim();
        if title.is_empty() {
            return Err(AppError::Invalid("task title cannot be empty".into()));
        }
        tx.execute(
            "UPDATE tasks SET title = ?1 WHERE id = ?2",
            params![title, id],
        )?;
    }
    if let Some(notes) = &patch.notes {
        tx.execute(
            "UPDATE tasks SET notes = ?1 WHERE id = ?2",
            params![notes, id],
        )?;
    }
    if let Some(completed) = patch.completed {
        let completed_at = completed.then(now);
        tx.execute(
            "UPDATE tasks SET completed = ?1, completed_at = ?2 WHERE id = ?3",
            params![completed, completed_at, id],
        )?;
    }
    if let Some(important) = patch.important {
        tx.execute(
            "UPDATE tasks SET important = ?1 WHERE id = ?2",
            params![important, id],
        )?;
    }
    if let Some(list_id) = &patch.list_id {
        let list = get_list(&tx, list_id)?;
        let min: f64 = tx.query_row(
            "SELECT COALESCE(MIN(sort_order), 0) FROM tasks WHERE list_id = ?1",
            [&list.id],
            |r| r.get(0),
        )?;
        tx.execute(
            "UPDATE tasks SET list_id = ?1, sort_order = ?2 WHERE id = ?3",
            params![list.id, min - 1.0, id],
        )?;
    }
    if let Some(my_day) = patch.my_day {
        tx.execute(
            "UPDATE tasks SET my_day_date = ?1 WHERE id = ?2",
            params![my_day.then_some(today), id],
        )?;
    }
    if let Some(due) = &patch.due_date {
        if let Some(d) = due {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| AppError::Invalid(format!("invalid due date: {d}")))?;
        }
        tx.execute(
            "UPDATE tasks SET due_date = ?1 WHERE id = ?2",
            params![due, id],
        )?;
    }
    if let Some(reminder) = &patch.reminder_at {
        if let Some(r) = reminder {
            DateTime::parse_from_rfc3339(r)
                .map_err(|_| AppError::Invalid(format!("invalid reminder: {r}")))?;
        }
        tx.execute(
            "UPDATE tasks SET reminder_at = ?1, reminder_fired = 0 WHERE id = ?2",
            params![reminder, id],
        )?;
    }
    tx.execute(
        "UPDATE tasks SET updated_at = ?1 WHERE id = ?2",
        params![now(), id],
    )?;
    tx.commit()?;
    get_task(conn, id)
}

pub fn delete_task(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    Ok(())
}

/// All tasks in every list, with steps.
pub fn all_tasks(conn: &Connection) -> AppResult<Vec<Task>> {
    query_tasks(
        conn,
        "SELECT * FROM tasks ORDER BY list_id, completed ASC, sort_order ASC",
        [],
    )
}

/// Insert or update a task (and its steps) in `list_id`, keeping its id.
///
/// Uses a real upsert instead of `INSERT OR REPLACE`: REPLACE deletes the old
/// row first, which would cascade-delete rows that reference it.
pub fn upsert_task(
    conn: &Connection,
    task: &Task,
    list_id: &str,
    reminder_fired: bool,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO tasks (id, list_id, title, notes, completed, completed_at, important,
            due_date, reminder_at, reminder_fired, my_day_date, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET
            list_id = excluded.list_id, title = excluded.title, notes = excluded.notes,
            completed = excluded.completed, completed_at = excluded.completed_at,
            important = excluded.important, due_date = excluded.due_date,
            reminder_at = excluded.reminder_at, reminder_fired = excluded.reminder_fired,
            my_day_date = excluded.my_day_date, sort_order = excluded.sort_order,
            created_at = excluded.created_at, updated_at = excluded.updated_at",
        params![
            task.id,
            list_id,
            task.title,
            task.notes,
            task.completed,
            task.completed_at,
            task.important,
            task.due_date,
            task.reminder_at,
            reminder_fired,
            task.my_day_date,
            task.sort_order,
            task.created_at,
            task.updated_at
        ],
    )?;
    for s in &task.steps {
        conn.execute(
            "INSERT INTO steps (id, task_id, title, completed, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                task_id = excluded.task_id, title = excluded.title,
                completed = excluded.completed, sort_order = excluded.sort_order",
            params![s.id, task.id, s.title, s.completed, s.sort_order],
        )?;
    }
    Ok(())
}

/// Re-insert a previously deleted task (used by "Undo").
pub fn restore_task(conn: &mut Connection, task: &Task) -> AppResult<Task> {
    let tx = conn.transaction()?;
    // Fall back to the default list if the original list is gone.
    let list_id = match get_list(&tx, &task.list_id) {
        Ok(l) => l.id,
        Err(_) => default_list_id(&tx)?,
    };
    upsert_task(&tx, task, &list_id, true)?;
    tx.commit()?;
    get_task(conn, &task.id)
}

pub fn reorder_tasks(conn: &mut Connection, ids: &[String]) -> AppResult<()> {
    let tx = conn.transaction()?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE tasks SET sort_order = ?1 WHERE id = ?2",
            params![i as f64, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn get_counts(conn: &Connection, today: &str) -> AppResult<Counts> {
    let count = |sql: &str, p: &[&dyn rusqlite::ToSql]| -> AppResult<i64> {
        Ok(conn.query_row(sql, p, |r| r.get(0))?)
    };
    let default_id = default_list_id(conn)?;
    let mut counts = Counts {
        my_day: count(
            "SELECT COUNT(*) FROM tasks WHERE completed = 0 AND my_day_date = ?1",
            &[&today],
        )?,
        important: count(
            "SELECT COUNT(*) FROM tasks WHERE completed = 0 AND important = 1",
            &[],
        )?,
        planned: count(
            "SELECT COUNT(*) FROM tasks WHERE completed = 0 AND (due_date IS NOT NULL OR reminder_at IS NOT NULL)",
            &[],
        )?,
        tasks: count(
            "SELECT COUNT(*) FROM tasks WHERE completed = 0 AND list_id = ?1",
            &[&default_id],
        )?,
        lists: HashMap::new(),
    };
    let mut stmt =
        conn.prepare("SELECT list_id, COUNT(*) FROM tasks WHERE completed = 0 GROUP BY list_id")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    for row in rows {
        let (id, n) = row?;
        counts.lists.insert(id, n);
    }
    Ok(counts)
}

// ---------------------------------------------------------------------------
// Steps
// ---------------------------------------------------------------------------

fn row_to_step(r: &Row) -> rusqlite::Result<Step> {
    Ok(Step {
        id: r.get("id")?,
        task_id: r.get("task_id")?,
        title: r.get("title")?,
        completed: r.get("completed")?,
        sort_order: r.get("sort_order")?,
    })
}

fn touch_task(conn: &Connection, task_id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE tasks SET updated_at = ?1 WHERE id = ?2",
        params![now(), task_id],
    )?;
    Ok(())
}

pub fn add_step(conn: &Connection, task_id: &str, title: &str) -> AppResult<Task> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::Invalid("step title cannot be empty".into()));
    }
    get_task(conn, task_id)?;
    let max: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) FROM steps WHERE task_id = ?1",
        [task_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO steps (id, task_id, title, completed, sort_order) VALUES (?1, ?2, ?3, 0, ?4)",
        params![new_id(), task_id, title, max + 1.0],
    )?;
    touch_task(conn, task_id)?;
    get_task(conn, task_id)
}

fn step_task_id(conn: &Connection, step_id: &str) -> AppResult<String> {
    conn.query_row("SELECT task_id FROM steps WHERE id = ?1", [step_id], |r| {
        r.get(0)
    })
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("step {step_id}")))
}

pub fn update_step(
    conn: &Connection,
    step_id: &str,
    title: Option<&str>,
    completed: Option<bool>,
) -> AppResult<Task> {
    let task_id = step_task_id(conn, step_id)?;
    if let Some(title) = title {
        let title = title.trim();
        if title.is_empty() {
            return Err(AppError::Invalid("step title cannot be empty".into()));
        }
        conn.execute(
            "UPDATE steps SET title = ?1 WHERE id = ?2",
            params![title, step_id],
        )?;
    }
    if let Some(completed) = completed {
        conn.execute(
            "UPDATE steps SET completed = ?1 WHERE id = ?2",
            params![completed, step_id],
        )?;
    }
    touch_task(conn, &task_id)?;
    get_task(conn, &task_id)
}

pub fn delete_step(conn: &Connection, step_id: &str) -> AppResult<Task> {
    let task_id = step_task_id(conn, step_id)?;
    conn.execute("DELETE FROM steps WHERE id = ?1", [step_id])?;
    touch_task(conn, &task_id)?;
    get_task(conn, &task_id)
}

/// Turn a step into its own task in the same list, then remove the step.
pub fn promote_step(conn: &mut Connection, step_id: &str, today: &str) -> AppResult<Task> {
    let task_id = step_task_id(conn, step_id)?;
    let task = get_task(conn, &task_id)?;
    let step = task
        .steps
        .iter()
        .find(|s| s.id == step_id)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("step {step_id}")))?;
    create_task(
        conn,
        &NewTask {
            list_id: Some(task.list_id.clone()),
            title: step.title,
            ..Default::default()
        },
        today,
    )?;
    delete_step(conn, step_id)
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub fn get_settings(conn: &Connection) -> AppResult<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<Result<HashMap<_, _>, _>>()?)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Reminders
// ---------------------------------------------------------------------------

/// Returns reminders that are due at `now` and marks them as fired.
pub fn take_due_reminders(conn: &Connection, now: DateTime<Utc>) -> AppResult<Vec<DueReminder>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.reminder_at, l.name FROM tasks t JOIN lists l ON l.id = t.list_id
         WHERE t.reminder_at IS NOT NULL AND t.reminder_fired = 0 AND t.completed = 0",
    )?;
    let candidates = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut due = Vec::new();
    for (id, title, at, list_name) in candidates {
        let Ok(at) = DateTime::parse_from_rfc3339(&at) else {
            continue;
        };
        if at.with_timezone(&Utc) <= now {
            conn.execute("UPDATE tasks SET reminder_fired = 1 WHERE id = ?1", [&id])?;
            due.push(DueReminder {
                task_id: id,
                title,
                list_name,
            });
        }
    }
    Ok(due)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODAY: &str = "2026-11-05";

    fn new_task(conn: &Connection, title: &str) -> Task {
        create_task(
            conn,
            &NewTask {
                title: title.into(),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap()
    }

    #[test]
    fn seeds_default_list_once() {
        let conn = open_in_memory().unwrap();
        init(&conn).unwrap();
        let lists = get_lists(&conn).unwrap();
        assert_eq!(lists.len(), 1);
        assert!(lists[0].is_default);
        assert!(delete_list(&conn, &lists[0].id).is_err());
    }

    #[test]
    fn new_tasks_go_on_top() {
        let conn = open_in_memory().unwrap();
        new_task(&conn, "first");
        new_task(&conn, "second");
        let tasks = get_tasks(&conn, &View::Tasks, TODAY).unwrap();
        let titles: Vec<_> = tasks.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["second", "first"]);
    }

    #[test]
    fn smart_lists() {
        let mut conn = open_in_memory().unwrap();
        let a = new_task(&conn, "a");
        let b = new_task(&conn, "b");
        let c = new_task(&conn, "c");
        update_task(
            &mut conn,
            &a.id,
            &TaskPatch {
                my_day: Some(true),
                important: Some(true),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        update_task(
            &mut conn,
            &b.id,
            &TaskPatch {
                due_date: Some(Some("2026-11-10".into())),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        update_task(
            &mut conn,
            &c.id,
            &TaskPatch {
                completed: Some(true),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();

        let ids = |conn: &Connection, v: View, today: &str| -> Vec<String> {
            get_tasks(conn, &v, today)
                .unwrap()
                .into_iter()
                .map(|t| t.id)
                .collect()
        };
        assert_eq!(ids(&conn, View::MyDay, TODAY), vec![a.id.clone()]);
        // My Day resets the next day.
        assert!(ids(&conn, View::MyDay, "2026-11-06").is_empty());
        assert_eq!(ids(&conn, View::Important, TODAY), vec![a.id.clone()]);
        assert_eq!(ids(&conn, View::Planned, TODAY), vec![b.id.clone()]);
        assert_eq!(ids(&conn, View::Tasks, TODAY).len(), 3);

        let counts = get_counts(&conn, TODAY).unwrap();
        assert_eq!(counts.my_day, 1);
        assert_eq!(counts.important, 1);
        assert_eq!(counts.planned, 1);
        assert_eq!(counts.tasks, 2);

        // Clearing a due date with explicit null.
        update_task(
            &mut conn,
            &b.id,
            &TaskPatch {
                due_date: Some(None),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        assert!(ids(&conn, View::Planned, TODAY).is_empty());
    }

    #[test]
    fn patch_deserialization_distinguishes_null_and_missing() {
        let p: TaskPatch = serde_json::from_str(r#"{"dueDate": null}"#).unwrap();
        assert_eq!(p.due_date, Some(None));
        assert_eq!(p.reminder_at, None);
        let p: TaskPatch = serde_json::from_str(r#"{"dueDate": "2026-01-01"}"#).unwrap();
        assert_eq!(p.due_date, Some(Some("2026-01-01".into())));
    }

    #[test]
    fn lists_crud_and_cascade() {
        let mut conn = open_in_memory().unwrap();
        let groceries = create_list(&conn, "Groceries", Some("green")).unwrap();
        let work = create_list(&conn, "Work", None).unwrap();
        reorder_lists(&mut conn, &[work.id.clone(), groceries.id.clone()]).unwrap();
        let names: Vec<_> = get_lists(&conn)
            .unwrap()
            .into_iter()
            .map(|l| l.name)
            .collect();
        assert_eq!(names, ["Tasks", "Work", "Groceries"]);

        let t = create_task(
            &conn,
            &NewTask {
                list_id: Some(groceries.id.clone()),
                title: "Milk".into(),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        add_step(&conn, &t.id, "Oat milk").unwrap();
        delete_list(&conn, &groceries.id).unwrap();
        assert!(get_task(&conn, &t.id).is_err());
        let steps: i64 = conn
            .query_row("SELECT COUNT(*) FROM steps", [], |r| r.get(0))
            .unwrap();
        assert_eq!(steps, 0);
    }

    #[test]
    fn steps_and_promote() {
        let mut conn = open_in_memory().unwrap();
        let t = new_task(&conn, "Trip");
        add_step(&conn, &t.id, "Book flight").unwrap();
        let t = add_step(&conn, &t.id, "Pack").unwrap();
        assert_eq!(t.steps.len(), 2);
        let t = update_step(&conn, &t.steps[0].id, None, Some(true)).unwrap();
        assert!(t.steps[0].completed);
        let t = promote_step(&mut conn, &t.steps[1].id, TODAY).unwrap();
        assert_eq!(t.steps.len(), 1);
        let tasks = get_tasks(&conn, &View::Tasks, TODAY).unwrap();
        assert_eq!(tasks[0].title, "Pack");
    }

    #[test]
    fn delete_and_restore() {
        let mut conn = open_in_memory().unwrap();
        let t = new_task(&conn, "Undo me");
        let t = add_step(&conn, &t.id, "step").unwrap();
        delete_task(&conn, &t.id).unwrap();
        assert!(get_task(&conn, &t.id).is_err());
        let restored = restore_task(&mut conn, &t).unwrap();
        assert_eq!(restored.title, "Undo me");
        assert_eq!(restored.steps.len(), 1);
    }

    #[test]
    fn reorder_and_move() {
        let mut conn = open_in_memory().unwrap();
        let a = new_task(&conn, "a");
        let b = new_task(&conn, "b");
        reorder_tasks(&mut conn, &[a.id.clone(), b.id.clone()]).unwrap();
        let tasks = get_tasks(&conn, &View::Tasks, TODAY).unwrap();
        assert_eq!(tasks[0].id, a.id);

        let other = create_list(&conn, "Other", None).unwrap();
        update_task(
            &mut conn,
            &b.id,
            &TaskPatch {
                list_id: Some(other.id.clone()),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        let moved = get_tasks(&conn, &View::List(other.id), TODAY).unwrap();
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].id, b.id);
    }

    #[test]
    fn search_matches_title_notes_and_steps() {
        let mut conn = open_in_memory().unwrap();
        let a = new_task(&conn, "Buy groceries");
        let b = new_task(&conn, "Call mom");
        let c = new_task(&conn, "Plan trip");
        update_task(
            &mut conn,
            &b.id,
            &TaskPatch {
                notes: Some("ask about groceries".into()),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        add_step(&conn, &c.id, "100% fun").unwrap();
        let found = search_tasks(&conn, "GROCER").unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|t| t.id == a.id));
        assert_eq!(search_tasks(&conn, "100%").unwrap().len(), 1);
        assert_eq!(search_tasks(&conn, "%").unwrap().len(), 1);
        assert!(search_tasks(&conn, "   ").unwrap().is_empty());
    }

    #[test]
    fn reminders_fire_once() {
        let mut conn = open_in_memory().unwrap();
        let t = new_task(&conn, "Ping");
        update_task(
            &mut conn,
            &t.id,
            &TaskPatch {
                reminder_at: Some(Some("2026-11-05T09:00:00+07:00".into())),
                ..Default::default()
            },
            TODAY,
        )
        .unwrap();
        let before = DateTime::parse_from_rfc3339("2026-11-05T01:59:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(take_due_reminders(&conn, before).unwrap().is_empty());
        let after = DateTime::parse_from_rfc3339("2026-11-05T02:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let due = take_due_reminders(&conn, after).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].list_name, "Tasks");
        assert!(take_due_reminders(&conn, after).unwrap().is_empty());
        assert!(update_task(
            &mut conn,
            &t.id,
            &TaskPatch {
                reminder_at: Some(Some("not a date".into())),
                ..Default::default()
            },
            TODAY,
        )
        .is_err());
    }

    #[test]
    fn settings_roundtrip() {
        let conn = open_in_memory().unwrap();
        set_setting(&conn, "theme", "dark").unwrap();
        set_setting(&conn, "theme", "light").unwrap();
        assert_eq!(get_settings(&conn).unwrap()["theme"], "light");
    }
}
