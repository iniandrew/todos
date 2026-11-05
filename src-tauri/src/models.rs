use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskList {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    /// Theme key (see `src/lib/themes.ts`).
    pub theme: String,
    pub sort_order: f64,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub completed: bool,
    pub sort_order: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub list_id: String,
    pub title: String,
    pub notes: String,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub important: bool,
    /// Local calendar date, `YYYY-MM-DD`.
    pub due_date: Option<String>,
    /// RFC 3339 timestamp.
    pub reminder_at: Option<String>,
    /// Local calendar date the task was added to "My Day".
    pub my_day_date: Option<String>,
    pub sort_order: f64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub steps: Vec<Step>,
}

/// Deserialize helper that distinguishes "field missing" (`None`) from
/// "field explicitly null" (`Some(None)`).
fn double_option<'de, D, T>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(de).map(Some)
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatch {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub completed: Option<bool>,
    pub important: Option<bool>,
    pub list_id: Option<String>,
    /// `true` adds the task to today's "My Day", `false` removes it.
    pub my_day: Option<bool>,
    #[serde(default, deserialize_with = "double_option")]
    pub due_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub reminder_at: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    pub list_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub important: bool,
    #[serde(default)]
    pub my_day: bool,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPatch {
    pub name: Option<String>,
    pub theme: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub icon: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub my_day: i64,
    pub important: i64,
    pub planned: i64,
    pub tasks: i64,
    pub lists: std::collections::HashMap<String, i64>,
}

/// Which tasks to show in the main view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    MyDay,
    Important,
    Planned,
    Tasks,
    List(String),
}

impl std::str::FromStr for View {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "myDay" => Ok(View::MyDay),
            "important" => Ok(View::Important),
            "planned" => Ok(View::Planned),
            "tasks" => Ok(View::Tasks),
            other => other
                .strip_prefix("list:")
                .map(|id| View::List(id.to_string()))
                .ok_or_else(|| format!("unknown view: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DueReminder {
    pub task_id: String,
    pub title: String,
    pub list_name: String,
}
