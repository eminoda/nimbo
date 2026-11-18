<script setup lang="ts">
import { CircleAlert, CircleCheck, CirclePlay, CircleStop, Eraser, LoaderCircle, RotateCw, Square } from '@lucide/vue'
import type { SessionStatus } from '@/types/session'

defineProps<{ status: SessionStatus; commandName?: string }>()
const emit = defineEmits<{ start: []; stop: []; reload: []; clear: [] }>()
</script>

<template>
  <div class="command-toolbar">
    <div class="toolbar-buttons">
      <button class="toolbar-button start-button" type="button" :disabled="status === 'running' || status === 'starting' || status === 'stopping' || !commandName" @click="emit('start')">
        <CirclePlay :size="16" />
        <span>启动</span>
      </button>
      <button class="toolbar-button" type="button" :disabled="status !== 'running' && status !== 'starting'" @click="emit('stop')">
        <Square :size="15" />
        <span>停止</span>
      </button>
      <button class="toolbar-button" type="button" :disabled="!commandName || status === 'starting' || status === 'stopping'" @click="emit('reload')">
        <RotateCw :size="16" />
        <span>Reload</span>
      </button>
      <button class="icon-button clear-logs" type="button" title="清空日志" aria-label="清空日志" @click="emit('clear')">
        <Eraser :size="16" />
      </button>
    </div>
    <div class="session-state" :class="`state-${status}`" role="status">
      <CircleCheck v-if="status === 'running'" class="session-state-icon" :size="16" />
      <LoaderCircle v-else-if="status === 'starting' || status === 'stopping'" class="session-state-icon spinning" :size="16" />
      <CircleAlert v-else-if="status === 'failed'" class="session-state-icon" :size="16" />
      <CircleStop v-else class="session-state-icon" :size="16" />
      <span>{{ status === 'running' ? 'Running' : status === 'starting' ? 'Starting' : status === 'stopping' ? 'Stopping' : status === 'failed' ? 'Failed' : status === 'exited' ? 'Exited' : 'Stopped' }}</span>
    </div>
  </div>
</template>
