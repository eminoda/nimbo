export type CommandKind = 'service' | 'task'

export interface CommandPreset {
  id: string
  name: string
  commandLine: string
  cwd?: string
  env: Record<string, string>
  kind: CommandKind
  autoStart: boolean
}

export interface ProjectBookmark {
  id: string
  name: string
  path: string
  iconPath?: string
  initials?: string
  commands: CommandPreset[]
}

export interface NimboSettings {
  schemaVersion: 1
  projects: ProjectBookmark[]
  projectOrder: string[]
  selectedProjectId?: string
  selectedCommandByProject: Record<string, string>
  closeToTray: boolean
}

export interface PackageScript {
  name: string
  script: string
  commandLine: string
}

export interface PackageScripts {
  packageManager: string
  scripts: PackageScript[]
}
