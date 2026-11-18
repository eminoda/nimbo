use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandKind {
    Service,
    Task,
}

impl Default for CommandKind {
    fn default() -> Self {
        Self::Service
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPreset {
    pub id: String,
    pub name: String,
    pub command_line: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub kind: CommandKind,
    #[serde(default)]
    pub auto_start: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBookmark {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub icon_path: Option<String>,
    #[serde(default)]
    pub initials: Option<String>,
    #[serde(default)]
    pub commands: Vec<CommandPreset>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NimboSettings {
    pub schema_version: u8,
    #[serde(default)]
    pub projects: Vec<ProjectBookmark>,
    #[serde(default)]
    pub project_order: Vec<String>,
    #[serde(default)]
    pub selected_project_id: Option<String>,
    #[serde(default)]
    pub selected_command_by_project: HashMap<String, String>,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NimboSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            projects: Vec::new(),
            project_order: Vec::new(),
            selected_project_id: None,
            selected_command_by_project: HashMap::new(),
            close_to_tray: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Exited,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSession {
    pub session_id: String,
    pub project_id: String,
    pub command_id: String,
    pub status: SessionStatus,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub exited_at: Option<u64>,
    pub exit_code: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionOutputEvent {
    pub session_id: String,
    pub stream: String,
    pub chunk: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStateEvent {
    pub session_id: String,
    pub status: SessionStatus,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageScript {
    pub name: String,
    pub script: String,
    pub command_line: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageScripts {
    pub package_manager: String,
    pub scripts: Vec<PackageScript>,
}
