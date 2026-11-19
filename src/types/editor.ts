export type EditorId = 'codex' | 'vscode' | 'cursor'

export interface InstalledEditor {
  id: EditorId
  name: string
}
