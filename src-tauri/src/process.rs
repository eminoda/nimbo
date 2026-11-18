use crate::models::{
    CommandPreset, ProjectBookmark, SessionOutputEvent, SessionStateEvent, SessionStatus,
    TerminalSession,
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::{
    ShellExt,
    process::{CommandChild, CommandEvent},
};
use tokio::sync::Notify;

pub struct AppState {
    pub settings: Mutex<crate::models::NimboSettings>,
    pub sessions: Mutex<HashMap<String, ManagedSession>>,
}

pub struct ManagedSession {
    pub session: TerminalSession,
    child: Option<CommandChild>,
    pub stopped: Arc<Notify>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(crate::models::NimboSettings::default()),
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn emit_state(
    app: &AppHandle,
    session_id: &str,
    status: SessionStatus,
    exit_code: Option<i32>,
    error: Option<String>,
) {
    let _ = app.emit(
        "session-state",
        SessionStateEvent {
            session_id: session_id.to_owned(),
            status,
            exit_code,
            error,
        },
    );
}

fn active(status: &SessionStatus) -> bool {
    matches!(
        status,
        SessionStatus::Starting | SessionStatus::Running | SessionStatus::Stopping
    )
}

pub fn list_sessions(state: &AppState) -> Vec<TerminalSession> {
    state
        .sessions
        .lock()
        .map(|sessions| {
            sessions
                .values()
                .map(|value| value.session.clone())
                .collect()
        })
        .unwrap_or_default()
}

pub fn stop_all(app: &AppHandle, state: &AppState) {
    if let Ok(mut sessions) = state.sessions.lock() {
        for managed in sessions.values_mut() {
            if active(&managed.session.status) {
                managed.session.status = SessionStatus::Stopping;
                if let Some(child) = managed.child.take() {
                    let _ = child.kill();
                }
                emit_state(
                    app,
                    &managed.session.session_id,
                    SessionStatus::Stopping,
                    None,
                    None,
                );
            }
        }
    }
}

pub fn start(
    app: AppHandle,
    state: &AppState,
    project: ProjectBookmark,
    command: CommandPreset,
    reuse_session_id: Option<String>,
) -> Result<TerminalSession, String> {
    let working_dir = command.cwd.as_deref().unwrap_or(&project.path);
    let path = Path::new(working_dir);
    if !path.is_dir() {
        return Err(format!("工作目录不存在：{working_dir}"));
    }
    if command.command_line.trim().is_empty() {
        return Err("启动命令不能为空".into());
    }

    let mut sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
    let existing = sessions.values().find(|managed| {
        managed.session.project_id == project.id && managed.session.command_id == command.id
    });
    if let Some(existing) = existing {
        if active(&existing.session.status) {
            return Err("这条命令已经在运行".into());
        }
    }

    let session_id = reuse_session_id
        .or_else(|| existing.map(|managed| managed.session.session_id.clone()))
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Commands are user-configured and shown in the UI before execution. Windows cmd
    // is used deliberately so npm.cmd/pnpm.cmd and ordinary shell syntax resolve.
    #[cfg(target_os = "windows")]
    let shell = app
        .shell()
        .command("cmd.exe")
        .args(["/D", "/S", "/C", &command.command_line]);
    #[cfg(not(target_os = "windows"))]
    let shell = app
        .shell()
        .command("/bin/sh")
        .args(["-lc", &command.command_line]);

    let mut shell = shell
        .current_dir(working_dir)
        .set_process_group(true)
        .kill_on_drop(true)
        .set_raw_out(true);
    for (key, value) in &command.env {
        shell = shell.env(key, value);
    }
    let (mut receiver, child) = shell
        .spawn()
        .map_err(|error| format!("启动失败：{error}"))?;
    let session = TerminalSession {
        session_id: session_id.clone(),
        project_id: project.id,
        command_id: command.id,
        status: SessionStatus::Running,
        pid: Some(child.pid()),
        started_at: Some(now_ms()),
        exited_at: None,
        exit_code: None,
    };
    let stopped = Arc::new(Notify::new());
    sessions.insert(
        session_id.clone(),
        ManagedSession {
            session: session.clone(),
            child: Some(child),
            stopped: stopped.clone(),
        },
    );

    let event_app = app.clone();
    let event_session = session_id.clone();
    tauri::async_runtime::spawn(async move {
        let mut exit_code = None;
        let mut failure = None;
        while let Some(event) = receiver.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => {
                    let _ = event_app.emit(
                        "session-output",
                        SessionOutputEvent {
                            session_id: event_session.clone(),
                            stream: "stdout".into(),
                            chunk: String::from_utf8_lossy(&bytes).into_owned(),
                        },
                    );
                }
                CommandEvent::Stderr(bytes) => {
                    let _ = event_app.emit(
                        "session-output",
                        SessionOutputEvent {
                            session_id: event_session.clone(),
                            stream: "stderr".into(),
                            chunk: String::from_utf8_lossy(&bytes).into_owned(),
                        },
                    );
                }
                CommandEvent::Terminated(payload) => {
                    exit_code = payload.code;
                    break;
                }
                CommandEvent::Error(error) => {
                    failure = Some(error);
                }
                _ => {}
            }
        }

        let (status, code) = if let Ok(mut all) = event_app.state::<AppState>().sessions.lock() {
            if let Some(managed) = all.get_mut(&event_session) {
                let stopped_by_user = managed.session.status == SessionStatus::Stopping;
                let code = exit_code;
                let status = if failure.is_some() {
                    SessionStatus::Failed
                } else if stopped_by_user {
                    SessionStatus::Stopped
                } else if code == Some(0) {
                    SessionStatus::Exited
                } else {
                    SessionStatus::Failed
                };
                managed.session.status = status.clone();
                managed.session.exit_code = code;
                managed.session.exited_at = Some(now_ms());
                managed.child = None;
                (status, code)
            } else {
                (SessionStatus::Exited, exit_code)
            }
        } else {
            (SessionStatus::Failed, exit_code)
        };
        emit_state(&event_app, &event_session, status, code, failure);
        stopped.notify_one();
    });

    emit_state(&app, &session_id, SessionStatus::Running, None, None);
    Ok(session)
}

pub async fn stop(
    app: &AppHandle,
    state: &AppState,
    session_id: &str,
) -> Result<Arc<Notify>, String> {
    let (child, stopped) = {
        let mut sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
        let managed = sessions.get_mut(session_id).ok_or("找不到该会话")?;
        if active(&managed.session.status) {
            managed.session.status = SessionStatus::Stopping;
            emit_state(app, session_id, SessionStatus::Stopping, None, None);
            (managed.child.take(), managed.stopped.clone())
        } else {
            managed.stopped.notify_one();
            (None, managed.stopped.clone())
        }
    };

    if let Some(child) = child {
        let result = tauri::async_runtime::spawn_blocking(move || child.kill())
            .await
            .map_err(|error| format!("停止任务失败：{error}"))?;
        if let Err(error) = result {
            if let Ok(mut sessions) = state.sessions.lock() {
                if let Some(managed) = sessions.get_mut(session_id) {
                    managed.session.status = SessionStatus::Failed;
                }
            }
            emit_state(
                app,
                session_id,
                SessionStatus::Failed,
                None,
                Some(error.to_string()),
            );
            return Err(format!("停止失败：{error}"));
        }
    }
    Ok(stopped)
}
