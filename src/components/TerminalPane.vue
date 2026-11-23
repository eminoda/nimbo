<script setup lang="ts">
import { onBeforeUnmount, onMounted, shallowRef, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { LogChunk } from '@/types/session'

const props = defineProps<{ sessionId?: string; chunks: LogChunk[]; clearVersion: number }>()
const emit = defineEmits<{
  input: [data: string]
  resize: [rows: number, cols: number]
}>()
const terminalHost = shallowRef<HTMLDivElement | null>(null)
let terminal: Terminal | undefined
let fitAddon: FitAddon | undefined
let resizeObserver: ResizeObserver | undefined
let lastSequence = 0
let lastResizedSession = ''

function writeChunks(chunks: LogChunk[]) {
  if (!terminal) return
  const missing = chunks.filter((chunk) => chunk.sequence > lastSequence)
  if (missing.length && missing[0].sequence > lastSequence + 1 && lastSequence !== 0) {
    terminal.reset()
    lastSequence = 0
    for (const chunk of chunks) {
      terminal.write(Uint8Array.from(chunk.chunk))
      lastSequence = chunk.sequence
    }
    return
  }
  for (const chunk of missing) {
    terminal.write(Uint8Array.from(chunk.chunk))
    lastSequence = chunk.sequence
  }
}

function fitTerminal() {
  if (!fitAddon || !terminal) return
  fitAddon.fit()
  if (props.sessionId) {
    emit('resize', terminal.rows, terminal.cols)
    lastResizedSession = props.sessionId
  }
}

watch(() => props.chunks, writeChunks)
watch(() => props.clearVersion, () => {
  terminal?.clear()
  lastSequence = 0
})
watch(() => props.sessionId, () => {
  if (!terminal) return
  terminal.reset()
  lastSequence = 0
  lastResizedSession = ''
  writeChunks(props.chunks)
  requestAnimationFrame(() => {
    fitTerminal()
    terminal?.focus()
  })
})

onMounted(() => {
  if (!terminalHost.value) return
  terminal = new Terminal({
    allowTransparency: false,
    convertEol: false,
    cursorBlink: true,
    cursorStyle: 'bar',
    disableStdin: false,
    fontFamily: 'Cascadia Code, Consolas, "Courier New", monospace',
    fontSize: 14,
    lineHeight: 1.25,
    scrollback: 10_000,
    theme: {
      background: '#11151b', foreground: '#c5ceda', cursor: '#69d9b0',
      selectionBackground: '#33465b', black: '#11151b', red: '#f17d84',
      green: '#72d6ae', yellow: '#e9c77a', blue: '#85b8f4', magenta: '#c49be9',
      cyan: '#7ec8d8', white: '#d9e0ea', brightBlack: '#667286', brightRed: '#ff9aa0',
      brightGreen: '#91e6bf', brightYellow: '#f1d792', brightBlue: '#a4c9ff',
      brightMagenta: '#d2b2f4', brightCyan: '#9bdce8', brightWhite: '#f2f5f9',
    },
  })
  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.loadAddon(new WebLinksAddon((_event, uri) => { void openUrl(uri).catch(() => undefined) }))
  terminal.open(terminalHost.value)
  fitAddon.fit()
  terminal.focus()
  terminal.onData((data) => emit('input', data))
  terminal.onResize(({ rows, cols }) => {
    if (props.sessionId && props.sessionId === lastResizedSession) emit('resize', rows, cols)
  })
  terminal.attachCustomKeyEventHandler((event) => {
    if (event.type === 'keydown' && (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'c' && terminal?.hasSelection()) {
      void navigator.clipboard?.writeText(terminal.getSelection()).catch(() => undefined)
      return false
    }
    return true
  })
  resizeObserver = new ResizeObserver(fitTerminal)
  resizeObserver.observe(terminalHost.value)
  if (props.sessionId) writeChunks(props.chunks)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  terminal?.dispose()
})
</script>

<template>
  <div class="terminal-pane">
    <div ref="terminalHost" class="terminal-host" />
  </div>
</template>
