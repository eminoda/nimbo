use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum EditorId {
    #[serde(rename = "codex")]
    Codex,
    #[serde(rename = "vscode")]
    VsCode,
    #[serde(rename = "cursor")]
    Cursor,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledEditor {
    pub id: EditorId,
    pub name: &'static str,
}

pub fn list_installed() -> Result<Vec<InstalledEditor>, String> {
    #[cfg(target_os = "windows")]
    {
        let mut installed = Vec::new();
        if codex_desktop_installed()? {
            installed.push(InstalledEditor {
                id: EditorId::Codex,
                name: "Codex",
            });
        }
        if vscode_executable().is_some() {
            installed.push(InstalledEditor {
                id: EditorId::VsCode,
                name: "VS Code",
            });
        }
        if cursor_executable().is_some() {
            installed.push(InstalledEditor {
                id: EditorId::Cursor,
                name: "Cursor",
            });
        }
        Ok(installed)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

pub fn open_project(app: &AppHandle, editor: EditorId, project_path: &Path) -> Result<(), String> {
    if !project_path.is_dir() {
        return Err("项目目录不存在，请先检查项目配置。".into());
    }

    #[cfg(target_os = "windows")]
    {
        match editor {
            EditorId::Codex => {
                if !codex_desktop_installed()? {
                    return Err("未检测到 Codex 桌面客户端，请重新扫描。".into());
                }
                let path = project_path.to_string_lossy();
                let url = format!("codex://threads/new?path={}", encode_query_value(&path));
                app.opener()
                    .open_url(url, None::<&str>)
                    .map_err(|error| format!("无法打开 Codex：{error}"))
            }
            EditorId::VsCode => {
                let executable = vscode_executable()
                    .ok_or_else(|| "未检测到 VS Code，请重新扫描。".to_string())?;
                launch_gui(executable, [project_path.as_os_str()])
                    .map_err(|error| format!("无法打开 VS Code：{error}"))
            }
            EditorId::Cursor => {
                let executable = cursor_executable()
                    .ok_or_else(|| "未检测到 Cursor，请重新扫描。".to_string())?;
                launch_gui(
                    executable,
                    [project_path.as_os_str(), std::ffi::OsStr::new("--classic")],
                )
                .map_err(|error| format!("无法打开 Cursor：{error}"))
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, editor);
        Err("IDE 启动目前仅支持 Windows。".into())
    }
}

#[cfg(target_os = "windows")]
fn codex_desktop_installed() -> Result<bool, String> {
    const START_MENU_QUERY: &str = "Get-StartApps | Where-Object { $_.AppID -like 'OpenAI.Codex_*!App' } | Select-Object -First 1 -ExpandProperty AppID";
    let output = hidden_command("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            START_MENU_QUERY,
        ])
        .output()
        .map_err(|error| format!("无法检测 Codex 桌面客户端：{error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if detail.is_empty() {
            "无法读取 Windows 已安装应用列表".into()
        } else {
            detail
        });
    }
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

#[cfg(target_os = "windows")]
fn vscode_executable() -> Option<PathBuf> {
    resolve_gui_executable("code", "Code.exe", &["Microsoft VS Code"])
}

#[cfg(target_os = "windows")]
fn cursor_executable() -> Option<PathBuf> {
    resolve_gui_executable("cursor", "Cursor.exe", &["Cursor", "cursor"])
}

#[cfg(target_os = "windows")]
fn resolve_gui_executable(
    command_name: &str,
    executable_name: &str,
    install_folders: &[&str],
) -> Option<PathBuf> {
    if let Some(launcher) = find_on_path(command_name) {
        if launcher
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case(executable_name))
        {
            return Some(launcher);
        }
        for parent in launcher.ancestors().take(8) {
            let candidate = parent.join(executable_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    let roots = ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"];
    for variable in roots {
        let Some(root) = env::var_os(variable).map(PathBuf::from) else {
            continue;
        };
        for folder in install_folders {
            let candidate = root.join(folder).join(executable_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        if variable == "LOCALAPPDATA" {
            for folder in install_folders {
                let candidate = root.join("Programs").join(folder).join(executable_name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_on_path(command_name: &str) -> Option<PathBuf> {
    let output = hidden_command("where.exe")
        .arg(command_name)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

#[cfg(target_os = "windows")]
fn hidden_command(program: &str) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(target_os = "windows")]
fn launch_gui<const N: usize>(
    executable: PathBuf,
    args: [&std::ffi::OsStr; N],
) -> Result<(), String> {
    Command::new(executable)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn encode_query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(b"0123456789ABCDEF"[(byte >> 4) as usize]));
            encoded.push(char::from(b"0123456789ABCDEF"[(byte & 0x0f) as usize]));
        }
    }
    encoded
}
