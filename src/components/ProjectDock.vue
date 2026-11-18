<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, shallowRef } from 'vue'
import { FolderPlus, Pencil, Settings2, Trash2 } from '@lucide/vue'
import nimboLogo from '../assets/nimbo-logo.svg'
import type { ProjectBookmark } from '@/types/project'

defineProps<{ projects: ProjectBookmark[]; selectedProjectId?: string; runningProjectIds: string[] }>()
const emit = defineEmits<{
  select: [projectId: string]
  editProject: [projectId: string]
  deleteProject: [projectId: string]
  addProject: []
  openSettings: []
}>()

const contextMenu = shallowRef<{ projectId: string; x: number; y: number }>()
const contextMenuElement = shallowRef<HTMLDivElement | null>(null)
const firstMenuAction = shallowRef<HTMLButtonElement | null>(null)

function closeContextMenu() {
  contextMenu.value = undefined
}

async function openContextMenu(event: MouseEvent, projectId: string) {
  const menuWidth = 156
  const menuHeight = 84
  const margin = 8
  contextMenu.value = {
    projectId,
    x: Math.max(margin, Math.min(event.clientX, window.innerWidth - menuWidth - margin)),
    y: Math.max(margin, Math.min(event.clientY, window.innerHeight - menuHeight - margin)),
  }
  await nextTick()
  firstMenuAction.value?.focus()
}

function handleOutsideClick(event: MouseEvent) {
  if (contextMenu.value && !contextMenuElement.value?.contains(event.target as Node)) {
    closeContextMenu()
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') closeContextMenu()
}

onMounted(() => {
  window.addEventListener('click', handleOutsideClick)
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleOutsideClick)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <aside class="project-dock" aria-label="项目栏">
    <button class="brand-mark" type="button" aria-label="Nimbo Terminal" title="Nimbo Terminal">
      <img class="dock-logo" :src="nimboLogo" alt="" aria-hidden="true" />
    </button>
    <div class="dock-divider" />
    <nav class="project-list" aria-label="项目">
      <button
        v-for="(project, index) in projects"
        :key="project.id"
        class="project-tile"
        :class="{ active: project.id === selectedProjectId }"
        :style="{ '--tile-hue': `${(index * 51 + 218) % 360}` }"
        type="button"
        :aria-label="project.name"
        :aria-current="project.id === selectedProjectId ? 'page' : undefined"
        :title="`${project.name} · ${project.path}`"
        @click="emit('select', project.id)"
        @contextmenu.prevent.stop="openContextMenu($event, project.id)"
      >
        {{ project.initials || project.name.slice(0, 2).toUpperCase() }}
        <span v-if="runningProjectIds.includes(project.id)" class="project-dot" />
      </button>
    </nav>
    <div
      v-if="contextMenu"
      ref="contextMenuElement"
      class="project-context-menu"
      role="menu"
      :aria-label="'项目菜单 ' + (projects.find((project) => project.id === contextMenu?.projectId)?.name ?? '')"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
      @keydown.esc.stop.prevent="closeContextMenu"
    >
      <button ref="firstMenuAction" type="button" role="menuitem" @click="emit('editProject', contextMenu.projectId); closeContextMenu()">
        <Pencil :size="15" />编辑项目
      </button>
      <button class="danger" type="button" role="menuitem" @click="emit('deleteProject', contextMenu.projectId); closeContextMenu()">
        <Trash2 :size="15" />删除项目
      </button>
    </div>
    <div class="dock-spacer" />
    <button class="dock-action" type="button" title="添加项目" aria-label="添加项目" @click="emit('addProject')">
      <FolderPlus :size="18" />
    </button>
    <button class="dock-action" type="button" title="设置" aria-label="设置" @click="emit('openSettings')">
      <Settings2 :size="18" />
    </button>
  </aside>
</template>
