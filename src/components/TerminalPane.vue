<script setup lang="ts">
import { onBeforeUnmount, onMounted, shallowRef, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { LogChunk } from '@/types/session'

const props = defineProps<{ sessionId?: string; chunks: LogChunk[]; clearVersion: number; running: boolean }>()
const emit = defineEmits<{ interrupt: [] }>()
const terminalHost = shallowRef<HTMLDivElement | null>(null)
let terminal: Terminal | undefined
let fitAddon: FitAddon | undefined
let resizeObserver: ResizeObserver | undefined
let lastSequence = 0

function writeChunks(chunks: LogChunk[]) {
  if (!terminal) return
  const missing = chunks.filter((chunk) => chunk.sequence > lastSequence)
  if (missing.length && missing[0].sequence > lastSequence + 1 && lastSequence !== 0) {
    terminal.reset()
    lastSequence = 0
    for (const chunk of chunks) {
      terminal.write(chunk.chunk)
      lastSequence = chunk.sequence
    }
    return
  }
  for (const chunk of missing) {
    terminal.write(chunk.chunk)
    lastSequence = chunk.sequence
  }
}

watch(() => props.chunks, writeChunks)
watch(() => props.clearVersion, () => {
  if (!terminal) return
  terminal.clear()
  lastSequence = 0
})
watch(() => props.sessionId, () => {
  if (!terminal) return
  terminal.reset()
  lastSequence = 0
  writeChunks(props.chunks)
})

onMounted(() => {
  if (!terminalHost.value) return
  terminal = new Terminal({
    allowTransparency: false,
    convertEol: true,
    cursorBlink: false,
    cursorStyle: 'bar',
    disableStdin: true,
    fontFamily: 'Cascadia Mono, Consolas, "Courier New", monospace',
    fontSize: 14,
    lineHeight: 1.25,
    scrollback: 2_000,
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
  terminal.attachCustomKeyEventHandler((event) => {
    if (event.type === 'keydown' && (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'c') {
      if (terminal?.hasSelection()) {
        void navigator.clipboard?.writeText(terminal.getSelection()).catch(() => undefined)
        return false
      }
      if (props.running) {
        emit('interrupt')
        return false
      }
    }
    return true
  })
  resizeObserver = new ResizeObserver(() => fitAddon?.fit())
  resizeObserver.observe(terminalHost.value)
  if (props.sessionId) {
    writeChunks(props.chunks)
  }
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  terminal?.dispose()
})
</script>

<template>
  <div class="terminal-pane">
    <div ref="terminalHost" class="terminal-host" />
    <p v-if="chunks.length === 0" class="terminal-empty">
      {{ running ? '等待命令输出…' : '点击上方「启动」查看命令输出' }}
    </p>
  </div>
</template>
