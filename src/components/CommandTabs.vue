<script setup lang="ts">
import { FilePenLine, Plus, TerminalSquare, X } from '@lucide/vue'
import type { CommandPreset } from '@/types/project'
import type { SessionStatus } from '@/types/session'

defineProps<{
  commands: CommandPreset[]
  selectedCommandId?: string
  statuses: Record<string, SessionStatus>
}>()
const emit = defineEmits<{
  select: [commandId: string]
  add: []
  edit: []
  remove: [commandId: string]
}>()
</script>

<template>
  <section class="command-tabs" aria-label="项目命令">
    <div class="tab-strip" role="tablist">
      <div
        v-for="command in commands"
        :key="command.id"
        role="presentation"
        class="command-tab-shell"
        :class="{ selected: command.id === selectedCommandId }"
      >
        <button
          class="command-tab"
          type="button"
          role="tab"
          :aria-selected="command.id === selectedCommandId"
          @click="emit('select', command.id)"
        >
          <span class="tab-terminal" :class="{ live: statuses[command.id] === 'running' }">
            <TerminalSquare :size="15" />
          </span>
          <span class="tab-name">{{ command.name }}</span>
          <span v-if="statuses[command.id] === 'running'" class="tab-live-dot" />
        </button>
        <button class="command-tab-close" type="button" :aria-label="`删除命令 ${command.name}`" :title="`删除 ${command.name}`" @click="emit('remove', command.id)">
          <X :size="17" />
        </button>
      </div>
    </div>
    <div class="tab-actions">
      <button class="icon-button tab-edit" type="button" title="编辑命令" aria-label="编辑命令" :disabled="!selectedCommandId" @click="emit('edit')">
        <FilePenLine :size="17" />
      </button>
      <button class="icon-button add-command" type="button" title="添加命令" aria-label="添加命令" @click="emit('add')">
        <Plus :size="18" />
      </button>
    </div>
  </section>
</template>
