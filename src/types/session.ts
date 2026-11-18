export type SessionStatus = 'stopped' | 'starting' | 'running' | 'stopping' | 'exited' | 'failed'

export interface TerminalSession {
  sessionId: string
  projectId: string
  commandId: string
  status: SessionStatus
  pid?: number
  startedAt?: number
  exitedAt?: number
  exitCode?: number | null
}

export interface SessionOutputEvent {
  sessionId: string
  stream: 'stdout' | 'stderr'
  chunk: string
}

export interface SessionStateEvent {
  sessionId: string
  status: SessionStatus
  exitCode?: number | null
  error?: string
}

export interface LogChunk extends SessionOutputEvent {
  sequence: number
}
