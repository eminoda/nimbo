use crate::models::{
    CommandPreset, ProjectBookmark, SessionOutputEvent, SessionStateEvent, SessionStatus,
    TerminalSession,
};
use portable_pty::{ChildKiller, CommandBuilder, PtySize, native_pty_system};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};

pub struct AppState {
    pub settings: Mutex<crate::models::NimboSettings>,
    pub sessions: Mutex<HashMap<String, ManagedSession>>,
}

pub struct ManagedSession {
    pub session: TerminalSession,
    killer: Option<Box<dyn ChildKiller + Send + Sync>>,
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    prompt_sequence: Arc<AtomicU64>,
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

fn live(status: &SessionStatus) -> bool {
    !matches!(status, SessionStatus::Exited | SessionStatus::Failed)
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
            if let Some(killer) = managed.killer.as_mut() {
                let _ = killer.kill();
                managed.killer = None;
            }
        }
    }
    let _ = app;
}

pub fn close_project_sessions(state: &AppState, project_id: &str) {
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.retain(|_, managed| {
            if managed.session.project_id == project_id {
                if let Some(killer) = managed.killer.as_mut() {
                    let _ = killer.kill();
                }
                false
            } else {
                true
            }
        });
    }
}

pub fn close_removed_command_sessions(state: &AppState, project: &ProjectBookmark) {
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.retain(|_, managed| {
            let removed = managed.session.project_id == project.id
                && !project
                    .commands
                    .iter()
                    .any(|command| command.id == managed.session.command_id);
            if removed {
                if let Some(killer) = managed.killer.as_mut() {
                    let _ = killer.kill();
                }
            }
            !removed
        });
    }
}

fn shell_builder(working_dir: &str, command: &CommandPreset) -> Result<CommandBuilder, String> {
    let mut builder = if cfg!(windows) {
        let mut cmd = CommandBuilder::new("cmd.exe");
        cmd.args(["/D", "/K", "chcp 65001 >NUL"]);
        // cmd's PROMPT expands the current directory while exposing standard OSC markers.
        cmd.env("PROMPT", "$E]633;A$E\\$P$E]633;B$E\\$G");
        cmd
    } else {
        let mut sh = CommandBuilder::new("/bin/bash");
        sh.args(["--noprofile", "--norc", "-i"]);
        sh.env("PS1", "\\[\\e]633;A\\a\\]\\w\\[\\e]633;B\\a\\]\\$ ");
        sh.env("TERM", "xterm-256color");
        sh
    };
    builder.cwd(working_dir);
    for (key, value) in &command.env {
        builder.env(key, value);
    }
    Ok(builder)
}

pub fn open(
    app: AppHandle,
    state: &AppState,
    project: ProjectBookmark,
    command: CommandPreset,
) -> Result<TerminalSession, String> {
    let working_dir = command.cwd.as_deref().unwrap_or(&project.path);
    if !Path::new(working_dir).is_dir() {
        return Err(format!("工作目录不存在：{working_dir}"));
    }
    let mut sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
    if let Some(existing) = sessions.values().find(|managed| {
        managed.session.project_id == project.id
            && managed.session.command_id == command.id
            && live(&managed.session.status)
    }) {
        return Ok(existing.session.clone());
    }

    sessions.retain(|_, managed| {
        managed.session.project_id != project.id
            || managed.session.command_id != command.id
            || live(&managed.session.status)
    });

    let session_id = uuid::Uuid::new_v4().to_string();
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| format!("创建终端失败：{error}"))?;
    let builder = shell_builder(working_dir, &command)?;
    let child = pair
        .slave
        .spawn_command(builder)
        .map_err(|error| format!("启动终端失败：{error}"))?;
    let pid = child.process_id();
    let killer = child.clone_killer();
    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| format!("读取终端失败：{error}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|error| format!("连接终端失败：{error}"))?;
    let prompt_sequence = Arc::new(AtomicU64::new(0));
    let session = TerminalSession {
        session_id: session_id.clone(),
        project_id: project.id,
        command_id: command.id,
        status: SessionStatus::Ready,
        pid,
        started_at: Some(now_ms()),
        exited_at: None,
        exit_code: None,
    };
    sessions.insert(
        session_id.clone(),
        ManagedSession {
            session: session.clone(),
            killer: Some(killer),
            writer,
            master: pair.master,
            prompt_sequence: prompt_sequence.clone(),
        },
    );

    let event_app = app.clone();
    let event_session = session_id.clone();
    let prompt_counter = prompt_sequence;
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = [0_u8; 8192];
        let mut marker_buffer = Vec::new();
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    let bytes = &buffer[..count];
                    let _ = event_app.emit(
                        "session-output",
                        SessionOutputEvent {
                            session_id: event_session.clone(),
                            stream: "stdout".into(),
                            chunk: bytes.to_vec(),
                        },
                    );
                    marker_buffer.extend_from_slice(bytes);
                    if marker_buffer.len() > 128 {
                        marker_buffer.drain(..marker_buffer.len() - 64);
                    }
                    let prompt_markers = marker_buffer
                        .windows(7)
                        .filter(|window| *window == b"\x1b]633;A")
                        .count();
                    if prompt_markers > 0 {
                        prompt_counter.fetch_add(prompt_markers as u64, Ordering::Relaxed);
                        if let Ok(mut all) = event_app.state::<AppState>().sessions.lock() {
                            if let Some(managed) = all.get_mut(&event_session) {
                                if managed.session.status != SessionStatus::Ready {
                                    managed.session.status = SessionStatus::Ready;
                                    managed.session.exit_code = None;
                                    emit_state(
                                        &event_app,
                                        &event_session,
                                        SessionStatus::Ready,
                                        None,
                                        None,
                                    );
                                }
                            }
                        }
                    }
                    // Keep only a possible marker prefix so markers split across
                    // reads are recognized without matching an old marker again.
                    if marker_buffer.len() > 6 {
                        marker_buffer.drain(..marker_buffer.len() - 6);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    let wait_app = app.clone();
    let wait_session = session_id.clone();
    std::thread::spawn(move || {
        let mut child = child;
        let result = child.wait();
        let (status, code, failure) = match result {
            Ok(exit) => (SessionStatus::Exited, Some(exit.exit_code() as i32), None),
            Err(error) => (SessionStatus::Failed, None, Some(error.to_string())),
        };
        if let Ok(mut all) = wait_app.state::<AppState>().sessions.lock() {
            if let Some(managed) = all.get_mut(&wait_session) {
                managed.session.status = status.clone();
                managed.session.exit_code = code;
                managed.session.exited_at = Some(now_ms());
                managed.killer = None;
            }
        }
        emit_state(&wait_app, &wait_session, status, code, failure);
    });
    Ok(session)
}

pub fn write(
    app: &AppHandle,
    state: &AppState,
    session_id: &str,
    data: &[u8],
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
    let managed = sessions.get_mut(session_id).ok_or("找不到该会话")?;
    if !live(&managed.session.status) {
        return Err("终端已关闭，请重新启动".into());
    }
    managed
        .writer
        .write_all(data)
        .map_err(|error| format!("发送终端输入失败：{error}"))?;
    managed
        .writer
        .flush()
        .map_err(|error| format!("发送终端输入失败：{error}"))?;
    if data.contains(&b'\r') && managed.session.status != SessionStatus::Running {
        managed.session.status = SessionStatus::Running;
        managed.session.exit_code = None;
        emit_state(app, session_id, SessionStatus::Running, None, None);
    } else if data.contains(&3)
        && matches!(
            managed.session.status,
            SessionStatus::Running | SessionStatus::Starting
        )
    {
        managed.session.status = SessionStatus::Stopping;
        emit_state(app, session_id, SessionStatus::Stopping, None, None);
    }
    Ok(())
}

pub fn resize(state: &AppState, session_id: &str, rows: u16, cols: u16) -> Result<(), String> {
    let sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
    let managed = sessions.get(session_id).ok_or("找不到该会话")?;
    managed
        .master
        .resize(PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| format!("调整终端尺寸失败：{error}"))
}

pub async fn stop(app: &AppHandle, state: &AppState, session_id: &str) -> Result<(), String> {
    write(app, state, session_id, &[3])
}

pub fn start_command(
    app: &AppHandle,
    state: &AppState,
    session_id: &str,
    command: &CommandPreset,
) -> Result<(), String> {
    if command.command_line.trim().is_empty() {
        return Err("启动命令不能为空".into());
    }
    {
        let sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
        if sessions
            .get(session_id)
            .is_some_and(|managed| managed.session.status == SessionStatus::Running)
        {
            return Err("已有命令正在运行".into());
        }
    }
    let mut bytes = command.command_line.as_bytes().to_vec();
    bytes.push(b'\r');
    write(app, state, session_id, &bytes)
}

pub async fn restart_command(
    app: AppHandle,
    state: &AppState,
    session_id: &str,
    command: &CommandPreset,
) -> Result<(), String> {
    let prompt_sequence = {
        let sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
        sessions
            .get(session_id)
            .ok_or("找不到该会话")?
            .prompt_sequence
            .clone()
    };
    let starting_prompt = prompt_sequence.load(Ordering::Relaxed);
    stop(&app, state, session_id).await?;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while prompt_sequence.load(Ordering::Relaxed) == starting_prompt {
        if tokio::time::Instant::now() >= deadline {
            return Err("等待当前命令停止超时，请检查终端后再试".into());
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    start_command(&app, state, session_id, command)
}
