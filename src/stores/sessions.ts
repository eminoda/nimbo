import { computed, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { nimboApi } from '@/services/tauri'
import type { LogChunk, SessionOutputEvent, SessionStateEvent, TerminalSession } from '@/types/session'

const MAX_LOG_CHUNKS = 2_000
const MAX_LOG_CHARACTERS = 1_000_000

export const useSessionsStore = defineStore('sessions', () => {
  const sessions = shallowRef<TerminalSession[]>([])
  const logs = shallowRef<Record<string, LogChunk[]>>({})
  const clearVersion = shallowRef(0)
  const error = shallowRef<string | null>(null)
  const unlisten = shallowRef<UnlistenFn[]>([])
  let sequence = 0
  let initialized = false
  const inputQueues = new Map<string, Promise<void>>()
  const pendingStates = new Map<string, SessionStateEvent>()

  function updateSession(session: TerminalSession) {
    const index = sessions.value.findIndex((item) => item.sessionId === session.sessionId)
    sessions.value = index < 0
      ? [...sessions.value, session]
      : sessions.value.map((item) => item.sessionId === session.sessionId ? session : item)
  }

  async function initialize() {
    if (initialized) return
    initialized = true
    const [outputListener, stateListener] = await Promise.all([
      listen<SessionOutputEvent>('session-output', (event) => {
        const item: LogChunk = { ...event.payload, sequence: ++sequence }
        const previous = logs.value[item.sessionId] ?? []
        const next = [...previous, item].slice(-MAX_LOG_CHUNKS)
        let retainedCharacters = 0
        let firstRetained = next.length
        while (firstRetained > 0 && retainedCharacters < MAX_LOG_CHARACTERS) {
          firstRetained -= 1
          retainedCharacters += next[firstRetained].chunk.length
        }
        logs.value = {
          ...logs.value,
          [item.sessionId]: next.slice(firstRetained),
        }
      }),
      listen<SessionStateEvent>('session-state', (event) => {
        const current = sessions.value.find((item) => item.sessionId === event.payload.sessionId)
        if (!current) {
          pendingStates.set(event.payload.sessionId, event.payload)
          return
        }
        updateSession({
          ...current,
          status: event.payload.status,
          exitCode: event.payload.exitCode,
          exitedAt: ['stopped', 'exited', 'failed'].includes(event.payload.status) ? Date.now() : undefined,
        })
        if (event.payload.error) error.value = event.payload.error
      }),
    ])
    unlisten.value = [outputListener, stateListener]
    sessions.value = await nimboApi.listSessions()
  }

  const currentSession = (projectId?: string, commandId?: string) => computed(() =>
    sessions.value.find((session) => session.projectId === projectId && session.commandId === commandId),
  )

  async function start(projectId: string, commandId: string) {
    error.value = null
    const session = await nimboApi.startCommand(projectId, commandId)
    const pending = pendingStates.get(session.sessionId)
    pendingStates.delete(session.sessionId)
    if (pending) updateSession({ ...session, status: pending.status, exitCode: pending.exitCode })
    else if (!sessions.value.some((item) => item.sessionId === session.sessionId)) updateSession(session)
  }

  async function openTerminal(projectId: string, commandId: string) {
    error.value = null
    const session = await nimboApi.openTerminal(projectId, commandId)
    const pending = pendingStates.get(session.sessionId)
    pendingStates.delete(session.sessionId)
    sessions.value = sessions.value.filter((item) =>
      item.sessionId === session.sessionId
        || item.projectId !== projectId
        || item.commandId !== commandId,
    )
    logs.value = Object.fromEntries(Object.entries(logs.value)
      .filter(([sessionId]) => sessionId === session.sessionId || sessions.value.some((item) => item.sessionId === sessionId)))
    const resolved = pending ? { ...session, status: pending.status, exitCode: pending.exitCode } : session
    updateSession(resolved)
    return resolved
  }

  async function writeTerminal(sessionId: string, data: Uint8Array) {
    const previous = inputQueues.get(sessionId) ?? Promise.resolve()
    const next = previous.then(() => nimboApi.writeTerminal(sessionId, Array.from(data)))
    inputQueues.set(sessionId, next)
    try { await next }
    finally {
      if (inputQueues.get(sessionId) === next) inputQueues.delete(sessionId)
    }
  }

  async function resizeTerminal(sessionId: string, rows: number, cols: number) {
    await nimboApi.resizeTerminal(sessionId, rows, cols)
  }

  async function stop(sessionId: string) {
    error.value = null
    await nimboApi.stopSession(sessionId)
  }

  async function reload(sessionId: string) {
    error.value = null
    const session = await nimboApi.reloadSession(sessionId)
    const pending = pendingStates.get(session.sessionId)
    pendingStates.delete(session.sessionId)
    if (pending) updateSession({ ...session, status: pending.status, exitCode: pending.exitCode })
    else if (!sessions.value.some((item) => item.sessionId === session.sessionId)) updateSession(session)
  }

  function clear(sessionId: string) {
    logs.value = { ...logs.value, [sessionId]: [] }
    clearVersion.value++
  }

  function forgetProject(projectId: string) {
    const forgottenIds = new Set(sessions.value
      .filter((session) => session.projectId === projectId)
      .map((session) => session.sessionId))
    sessions.value = sessions.value.filter((session) => session.projectId !== projectId)
    logs.value = Object.fromEntries(Object.entries(logs.value)
      .filter(([sessionId]) => !forgottenIds.has(sessionId)))
  }

  function clearError() { error.value = null }

  function dispose() {
    unlisten.value.forEach((disposeListener) => disposeListener())
    unlisten.value = []
    initialized = false
  }

  return { sessions, logs, clearVersion, error, initialize, currentSession, start, openTerminal, writeTerminal, resizeTerminal, stop, reload, clear, forgetProject, clearError, dispose }
})
