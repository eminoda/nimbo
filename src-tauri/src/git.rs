use serde::Serialize;
use std::{path::Path, process::Command};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchInfo {
    pub current_branch: Option<String>,
    pub local_branches: Vec<String>,
    pub worktree_branches: Vec<String>,
}

fn run_git(project_path: &Path, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command
        .arg("-C")
        .arg(project_path)
        .args(args)
        .output()
        .map_err(|error| format!("无法启动 Git：{error}"))?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if message.is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        } else {
            message
        };
        return Err(if message.is_empty() {
            format!("Git 命令失败（状态码 {:?}）", output.status.code())
        } else {
            message
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn current_branch(project_path: &Path) -> Result<Option<String>, String> {
    let branch = run_git(project_path, &["branch", "--show-current"])?;
    Ok(match branch.trim() {
        "" => None,
        name => Some(name.to_owned()),
    })
}

pub fn branch_info(project_path: &Path) -> Result<GitBranchInfo, String> {
    let current = run_git(project_path, &["branch", "--show-current"])?;
    let branches = run_git(project_path, &["branch", "--format=%(refname:short)"])?;
    let worktrees = run_git(project_path, &["worktree", "list", "--porcelain"])?;
    let project_path =
        std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
    let mut local_branches = branches
        .lines()
        .map(str::trim)
        .filter(|branch| !branch.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    local_branches.sort_unstable();
    local_branches.dedup();

    let mut worktree_branches = Vec::new();
    let mut worktree_path = None;
    for line in worktrees.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            worktree_path = Some(Path::new(path));
        } else if let Some(name) = line.strip_prefix("branch refs/heads/") {
            if let Some(path) = worktree_path {
                let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                let same_path = if cfg!(windows) {
                    path.to_string_lossy()
                        .eq_ignore_ascii_case(&project_path.to_string_lossy())
                } else {
                    path == project_path
                };
                if !same_path {
                    worktree_branches.push(name.to_owned());
                }
            }
        }
    }
    worktree_branches.sort_unstable();
    worktree_branches.dedup();

    Ok(GitBranchInfo {
        current_branch: match current.trim() {
            "" => None,
            branch => Some(branch.to_owned()),
        },
        local_branches,
        worktree_branches,
    })
}

pub fn switch_branch(project_path: &Path, branch_name: &str) -> Result<(), String> {
    run_git(project_path, &["switch", "--", branch_name]).map(|_| ())
}
