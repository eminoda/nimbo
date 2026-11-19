import { invoke } from '@tauri-apps/api/core'
import type { NimboSettings, PackageScripts, ProjectBookmark } from '@/types/project'
import type { TerminalSession } from '@/types/session'
import type { GitBranchInfo } from '@/types/git'
import type { EditorId, InstalledEditor } from '@/types/editor'

export const nimboApi = {
  loadSettings: () => invoke<NimboSettings>('load_settings'),
  settingsFilePath: () => invoke<string>('settings_file_path'),
  listPackageScripts: (projectId: string) =>
    invoke<PackageScripts>('list_package_scripts', { projectId }),
  listProjectBranches: (projectId: string) =>
    invoke<GitBranchInfo>('list_project_branches', { projectId }),
  switchProjectBranch: (projectId: string, branchName: string) =>
    invoke<GitBranchInfo>('switch_project_branch', { projectId, branchName }),
  listInstalledEditors: () => invoke<InstalledEditor[]>('list_installed_editors'),
  openProjectInEditor: (projectId: string, editorId: EditorId) =>
    invoke<void>('open_project_in_editor', { projectId, editorId }),
  saveProject: (project: ProjectBookmark) => invoke<void>('save_project', { project }),
  deleteProject: (projectId: string) => invoke<void>('delete_project', { projectId }),
  selectProject: (projectId: string) => invoke<void>('select_project', { projectId }),
  selectCommand: (projectId: string, commandId: string) =>
    invoke<void>('select_command', { projectId, commandId }),
  setCloseToTray: (enabled: boolean) => invoke<void>('set_close_to_tray', { enabled }),
  listSessions: () => invoke<TerminalSession[]>('list_sessions'),
  startCommand: (projectId: string, commandId: string) =>
    invoke<TerminalSession>('start_command', { projectId, commandId }),
  stopSession: (sessionId: string) => invoke<void>('stop_session', { sessionId }),
  reloadSession: (sessionId: string) =>
    invoke<TerminalSession>('reload_session', { sessionId }),
}
