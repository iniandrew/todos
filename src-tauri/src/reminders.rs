//! Background loop that turns due reminders into desktop notifications.

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::commands::AppState;
use crate::{backup, db};

const POLL_INTERVAL: Duration = Duration::from_secs(15);

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("reminders".into())
        .spawn(move || loop {
            check(&app);
            auto_backup(&app);
            std::thread::sleep(POLL_INTERVAL);
        })
        .expect("failed to spawn reminder thread");
}

fn check(app: &AppHandle) {
    let due = {
        let state = app.state::<AppState>();
        let conn = state.conn();
        match db::take_due_reminders(&conn, chrono::Utc::now()) {
            Ok(due) => due,
            Err(e) => {
                eprintln!("reminder check failed: {e}");
                return;
            }
        }
    };
    if due.is_empty() {
        return;
    }
    for r in &due {
        if let Err(e) = app
            .notification()
            .builder()
            .title(&r.title)
            .body(&r.list_name)
            .show()
        {
            eprintln!("failed to show notification: {e}");
        }
    }
    let _ = app.emit("reminders-fired", &due);
}

/// Writes today's automatic backup once per day (cheap no-op otherwise).
fn auto_backup(app: &AppHandle) {
    let state = app.state::<AppState>();
    let conn = state.conn();
    let enabled = db::get_settings(&conn)
        .map(|s| s.get("autoBackup").map(String::as_str) != Some("off"))
        .unwrap_or(true);
    if !enabled {
        return;
    }
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if let Err(e) = backup::auto_backup(&conn, &state.data_dir, &today) {
        eprintln!("automatic backup failed: {e}");
    }
}
