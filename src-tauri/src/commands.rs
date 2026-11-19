use crate::{
    editors::{self, EditorId, InstalledEditor},
    git::{self, GitBranchInfo},
    models::{
        NimboSettings, PackageScript, PackageScripts, ProjectBookmark, SessionStatus,
        TerminalSession,
    },
    process::{self, AppState},
    storage,
};
use std::path::PathBuf;
use tauri::{AppHandle, State};

fn package_manager(
    project_path: &std::path::Path,
    package_json: &serde_json::Value,
) -> &'static str {
    if let Some(value) = package_json
        .get("packageManager")
        .and_then(serde_json::Value::as_str)
    {
        let manager = value.split('@').next().unwrap_or_default();
        if matches!(manager, "npm" | "pnpm" | "yarn" | "bun") {
            return match manager {
                "pnpm" => "pnpm",
                "yarn" => "yarn",
                "bun" => "bun",
                _ => "npm",
            };
        }
    }
    if project_path.join("pnpm-lock.yaml").is_file() {
        "pnpm"
    } else if project_path.join("yarn.lock").is_file() {
        "yarn"
    } else if project_path.join("bun.lock").is_file() || project_path.join("bun.lockb").is_file() {
        "bun"
    } else {
        "npm"
    }
}

fn is_safe_script_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
        })
}

fn persist_settings(
    app: &AppHandle,
    current: &mut NimboSettings,
    update: impl FnOnce(&mut NimboSettings),
) -> Result<(), String> {
    let mut next = current.clone();
    update(&mut next);
    storage::save(app, &next)?;
    *current = next;
    Ok(())
}

#[tauri::command]
pub fn load_settings(state: State<'_, AppState>) -> Result<NimboSettings, String> {
    state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|_| "配置不可用".into())
}

#[tauri::command]
pub fn settings_file_path(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let path = storage::settings_path(&app)?;
    if !path.exists() {
        let settings = state.settings.lock().map_err(|_| "配置不可用")?.clone();
        storage::save(&app, &settings)?;
    }
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn list_package_scripts(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<PackageScripts, String> {
    let project_path = {
        let settings = state.settings.lock().map_err(|_| "配置不可用")?;
        settings
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .map(|project| std::path::PathBuf::from(&project.path))
            .ok_or("项目不存在")?
    };
    let manifest_path = project_path.join("package.json");
    let contents = std::fs::read_to_string(&manifest_path)
        .map_err(|_| "项目目录下没有可读取的 package.json".to_string())?;
    let manifest: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|error| format!("package.json 格式无效：{error}"))?;
    let manager = package_manager(&project_path, &manifest);
    let mut scripts = manifest
        .get("scripts")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|entries| entries.iter())
        .filter_map(|(name, script)| {
            if !is_safe_script_name(name) {
                return None;
            }
            let script = script.as_str()?.to_owned();
            let command_line = match manager {
                "npm" => format!("npm run {name}"),
                "pnpm" => format!("pnpm run {name}"),
                "yarn" => format!("yarn run {name}"),
                "bun" => format!("bun run {name}"),
                _ => unreachable!(),
            };
            Some(PackageScript {
                name: name.clone(),
                script,
                command_line,
            })
        })
        .collect::<Vec<_>>();
    scripts.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(PackageScripts {
        package_manager: manager.to_string(),
        scripts,
    })
}

fn project_path(state: &AppState, project_id: &str) -> Result<PathBuf, String> {
    let settings = state.settings.lock().map_err(|_| "配置不可用")?;
    settings
        .projects
        .iter()
        .find(|project| project.id == project_id)
        .map(|project| PathBuf::from(&project.path))
        .ok_or_else(|| "项目不存在".to_string())
}

#[tauri::command]
pub fn list_project_branches(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<GitBranchInfo, String> {
    let path = project_path(&state, &project_id)?;
    git::branch_info(&path)
}

#[tauri::command]
pub fn list_installed_editors() -> Result<Vec<InstalledEditor>, String> {
    editors::list_installed()
}

#[tauri::command]
pub fn open_project_in_editor(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    editor_id: EditorId,
) -> Result<(), String> {
    let path = project_path(&state, &project_id)?;
    editors::open_project(&app, editor_id, &path)
}

#[tauri::command]
pub fn switch_project_branch(
    state: State<'_, AppState>,
    project_id: String,
    branch_name: String,
) -> Result<GitBranchInfo, String> {
    if branch_name.trim().is_empty() {
        return Err("请选择一个分支".into());
    }

    let sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
    if sessions.values().any(|managed| {
        managed.session.project_id == project_id
            && matches!(
                managed.session.status,
                SessionStatus::Starting | SessionStatus::Running | SessionStatus::Stopping
            )
    }) {
        return Err("请先停止此项目中的运行命令，再切换分支。".into());
    }

    let path = project_path(&state, &project_id)?;
    let current = git::branch_info(&path)?;
    if !current
        .local_branches
        .iter()
        .any(|branch| branch == &branch_name)
    {
        return Err(format!("本地分支不存在：{branch_name}"));
    }
    git::switch_branch(&path, &branch_name)?;
    git::branch_info(&path)
}

#[tauri::command]
pub fn save_project(
    app: AppHandle,
    state: State<'_, AppState>,
    project: ProjectBookmark,
) -> Result<(), String> {
    if project.name.trim().is_empty() {
        return Err("项目名称不能为空".into());
    }
    if !std::path::Path::new(&project.path).is_dir() {
        return Err("所选项目目录不存在".into());
    }
    let mut sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
    if sessions.values().any(|managed| {
        managed.session.project_id == project.id
            && matches!(
                managed.session.status,
                SessionStatus::Starting | SessionStatus::Running | SessionStatus::Stopping
            )
            && !project
                .commands
                .iter()
                .any(|command| command.id == managed.session.command_id)
    }) {
        return Err("项目仍有运行中的命令，请先停止后再删除".into());
    }
    let mut settings = state.settings.lock().map_err(|_| "配置不可用")?;
    let mut next = settings.clone();
    if let Some(existing) = next.projects.iter_mut().find(|item| item.id == project.id) {
        *existing = project.clone();
    } else {
        next.project_order.push(project.id.clone());
        next.projects.push(project.clone());
    }
    next.selected_project_id = Some(project.id.clone());
    let selected_is_valid = next
        .selected_command_by_project
        .get(&project.id)
        .is_some_and(|selected| {
            project
                .commands
                .iter()
                .any(|command| &command.id == selected)
        });
    if !selected_is_valid {
        if let Some(command) = project.commands.first() {
            next.selected_command_by_project
                .insert(project.id.clone(), command.id.clone());
        } else {
            next.selected_command_by_project.remove(&project.id);
        }
    }
    storage::save(&app, &next)?;
    *settings = next;
    sessions.retain(|_, managed| {
        managed.session.project_id != project.id
            || project
                .commands
                .iter()
                .any(|command| command.id == managed.session.command_id)
    });
    Ok(())
}

#[tauri::command]
pub fn delete_project(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
    if sessions.values().any(|managed| {
        managed.session.project_id == project_id
            && matches!(
                managed.session.status,
                SessionStatus::Starting | SessionStatus::Running | SessionStatus::Stopping
            )
    }) {
        return Err("项目仍有运行中的命令，请先停止后再删除".into());
    }
    let mut settings = state.settings.lock().map_err(|_| "配置不可用")?;
    persist_settings(&app, &mut settings, |next| {
        next.projects.retain(|project| project.id != project_id);
        next.project_order.retain(|id| id != &project_id);
        next.selected_command_by_project.remove(&project_id);
        if next.selected_project_id.as_deref() == Some(&project_id) {
            next.selected_project_id = next.project_order.first().cloned();
        }
    })?;
    sessions.retain(|_, managed| managed.session.project_id != project_id);
    Ok(())
}

#[tauri::command]
pub fn select_project(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|_| "配置不可用")?;
    if !settings
        .projects
        .iter()
        .any(|project| project.id == project_id)
    {
        return Err("项目不存在".into());
    }
    persist_settings(&app, &mut settings, |next| {
        next.selected_project_id = Some(project_id)
    })
}

#[tauri::command]
pub fn select_command(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    command_id: String,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|_| "配置不可用")?;
    let valid = settings
        .projects
        .iter()
        .find(|project| project.id == project_id)
        .is_some_and(|project| {
            project
                .commands
                .iter()
                .any(|command| command.id == command_id)
        });
    if !valid {
        return Err("命令不存在".into());
    }
    persist_settings(&app, &mut settings, |next| {
        next.selected_command_by_project
            .insert(project_id, command_id);
    })
}

#[tauri::command]
pub fn set_close_to_tray(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|_| "配置不可用")?;
    persist_settings(&app, &mut settings, |next| next.close_to_tray = enabled)
}

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Vec<TerminalSession> {
    process::list_sessions(&state)
}

fn find_project_command(
    settings: &NimboSettings,
    project_id: &str,
    command_id: &str,
) -> Result<(ProjectBookmark, crate::models::CommandPreset), String> {
    let project = settings
        .projects
        .iter()
        .find(|project| project.id == project_id)
        .cloned()
        .ok_or("项目不存在")?;
    let command = project
        .commands
        .iter()
        .find(|command| command.id == command_id)
        .cloned()
        .ok_or("命令不存在")?;
    Ok((project, command))
}

#[tauri::command]
pub fn start_command(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    command_id: String,
) -> Result<TerminalSession, String> {
    let (project, command) = {
        let settings = state.settings.lock().map_err(|_| "配置不可用")?;
        find_project_command(&settings, &project_id, &command_id)?
    };
    process::start(app, &state, project, command, None)
}

#[tauri::command]
pub async fn stop_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    process::stop(&app, &state, &session_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn reload_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<TerminalSession, String> {
    let current = {
        let sessions = state.sessions.lock().map_err(|_| "会话状态不可用")?;
        sessions
            .get(&session_id)
            .map(|managed| managed.session.clone())
    };
    let Some(current) = current else {
        return Err("找不到该会话".into());
    };
    let (project, command) = {
        let settings = state.settings.lock().map_err(|_| "配置不可用")?;
        find_project_command(&settings, &current.project_id, &current.command_id)?
    };
    let stopped = process::stop(&app, &state, &session_id).await?;
    tokio::time::timeout(std::time::Duration::from_secs(8), stopped.notified())
        .await
        .map_err(|_| "旧进程未能在 8 秒内退出，已取消 Reload".to_string())?;
    process::start(app, &state, project, command, Some(session_id))
}
