use serde::Serialize;
use std::{path::Path, process::Command};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchInfo {
    pub current_branch: Option<String>,
    pub local_branches: Vec<String>,
}

fn run_git(project_path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
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

pub fn branch_info(project_path: &Path) -> Result<GitBranchInfo, String> {
    let current = run_git(project_path, &["branch", "--show-current"])?;
    let branches = run_git(project_path, &["branch", "--format=%(refname:short)"])?;
    let mut local_branches = branches
        .lines()
        .map(str::trim)
        .filter(|branch| !branch.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    local_branches.sort_unstable();
    local_branches.dedup();

    Ok(GitBranchInfo {
        current_branch: match current.trim() {
            "" => None,
            branch => Some(branch.to_owned()),
        },
        local_branches,
    })
}

pub fn switch_branch(project_path: &Path, branch_name: &str) -> Result<(), String> {
    run_git(project_path, &["switch", "--", branch_name]).map(|_| ())
}
