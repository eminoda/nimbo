<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, shallowRef } from 'vue'
import { ChevronRight, Code2, FolderPlus, PanelLeftClose, PanelLeftOpen, Pencil, RefreshCw, Settings2, Trash2 } from '@lucide/vue'
import { nimboApi } from '@/services/tauri'
import codexIcon from '@/assets/editors/codex.png'
import cursorIcon from '@/assets/editors/cursor.png'
import vscodeIcon from '@/assets/editors/vscode.png'
import type { EditorId, InstalledEditor } from '@/types/editor'
import type { ProjectBookmark } from '@/types/project'

defineProps<{ projects: ProjectBookmark[]; selectedProjectId?: string; runningProjectIds: string[]; collapsed: boolean }>()
const emit = defineEmits<{
  toggleCollapse: []
  select: [projectId: string]
  editProject: [projectId: string]
  deleteProject: [projectId: string]
  openInEditor: [projectId: string, editorId: EditorId]
  addProject: []
  openSettings: []
}>()

const contextMenu = shallowRef<{ projectId: string; x: number; y: number }>()
const contextMenuElement = shallowRef<HTMLDivElement | null>(null)
const firstMenuAction = shallowRef<HTMLButtonElement | null>(null)
const ideSubmenuElement = shallowRef<HTMLDivElement | null>(null)
const ideSubmenuOpen = shallowRef(false)
const installedEditors = shallowRef<InstalledEditor[]>([])
const editorScanState = shallowRef<'idle' | 'loading' | 'ready' | 'failed'>('idle')
const editorScanError = shallowRef('')
const submenuOpensLeft = shallowRef(false)
let contextMenuGeneration = 0

const editorIcons: Record<EditorId, string> = {
  codex: codexIcon,
  vscode: vscodeIcon,
  cursor: cursorIcon,
}

const contextMenuWidth = 172
const contextMenuHeight = 160
const ideSubmenuWidth = 166

function closeContextMenu() {
  contextMenuGeneration += 1
  contextMenu.value = undefined
  ideSubmenuOpen.value = false
}

async function openContextMenu(event: MouseEvent, projectId: string) {
  const margin = 8
  const x = Math.max(margin, Math.min(event.clientX, window.innerWidth - contextMenuWidth - margin))
  const generation = ++contextMenuGeneration
  contextMenu.value = {
    projectId,
    x,
    y: Math.max(margin, Math.min(event.clientY, window.innerHeight - contextMenuHeight - margin)),
  }
  submenuOpensLeft.value = x + contextMenuWidth + ideSubmenuWidth > window.innerWidth - margin
  ideSubmenuOpen.value = false
  installedEditors.value = []
  editorScanError.value = ''
  editorScanState.value = 'loading'
  void scanInstalledEditors(generation)
  await nextTick()
  firstMenuAction.value?.focus()
}

async function scanInstalledEditors(generation = contextMenuGeneration) {
  editorScanState.value = 'loading'
  editorScanError.value = ''
  try {
    const editors = await nimboApi.listInstalledEditors()
    if (generation !== contextMenuGeneration || !contextMenu.value) return
    installedEditors.value = editors
    editorScanState.value = 'ready'
  } catch (error) {
    if (generation !== contextMenuGeneration || !contextMenu.value) return
    editorScanError.value = error instanceof Error ? error.message : String(error)
    editorScanState.value = 'failed'
  }
}

function openProjectInEditor(editorId: EditorId) {
  if (!contextMenu.value) return
  emit('openInEditor', contextMenu.value.projectId, editorId)
  closeContextMenu()
}

async function openIdeSubmenu() {
  ideSubmenuOpen.value = true
  await nextTick()
  ideSubmenuElement.value?.querySelector<HTMLButtonElement>('.editor-menu-option, .editor-menu-refresh')?.focus()
}

function handleContextEscape() {
  if (ideSubmenuOpen.value) {
    ideSubmenuOpen.value = false
    firstMenuAction.value?.focus()
  } else {
    closeContextMenu()
  }
}

function handleOutsideClick(event: MouseEvent) {
  if (contextMenu.value && !contextMenuElement.value?.contains(event.target as Node)) {
    closeContextMenu()
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') handleContextEscape()
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
  <aside class="project-dock" :class="{ 'is-collapsed': collapsed }" aria-label="项目栏">
    <button
      v-if="!collapsed"
      class="dock-action dock-collapse"
      type="button"
      title="收起侧栏"
      aria-label="收起侧栏"
      :aria-expanded="!collapsed"
      @click="emit('toggleCollapse')"
    >
      <PanelLeftClose :size="18" />
    </button>
    <nav class="project-list" aria-label="项目">
      <button
        v-for="(project, index) in projects"
        :key="project.id"
        class="project-tile"
        :class="{ active: project.id === selectedProjectId }"
        :style="{ '--tile-color': project.iconColor || `hsl(${(index * 51 + 218) % 360} 38% 59%)` }"
        type="button"
        :aria-label="project.name"
        :aria-current="project.id === selectedProjectId ? 'page' : undefined"
        :title="`${project.name} · ${project.path}`"
        @click="emit('select', project.id)"
        @contextmenu.prevent.stop="openContextMenu($event, project.id)"
      >
        <img v-if="project.iconDataUrl" class="project-tile-image" :src="project.iconDataUrl" alt="" />
        <span v-else>{{ project.initials || project.name.slice(0, 2).toUpperCase() }}</span>
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
      @keydown.esc.stop.prevent="handleContextEscape"
    >
      <div class="project-context-submenu-anchor" :class="{ 'opens-left': submenuOpensLeft }" @mouseleave="ideSubmenuOpen = false">
        <button
          ref="firstMenuAction"
          class="project-context-submenu-trigger"
          type="button"
          role="menuitem"
          aria-haspopup="menu"
          :aria-expanded="ideSubmenuOpen"
          @mouseenter="ideSubmenuOpen = true"
          @click="ideSubmenuOpen = true"
          @keydown.right.prevent="openIdeSubmenu"
        >
          <Code2 :size="15" />在 IDE 中打开<ChevronRight class="context-submenu-arrow" :size="14" />
        </button>
        <div v-if="ideSubmenuOpen" ref="ideSubmenuElement" class="project-context-submenu" role="menu" aria-label="选择 IDE" @click.stop @keydown.left.stop.prevent="handleContextEscape">
          <p v-if="editorScanState === 'loading'" class="editor-menu-message" role="status">正在检测 IDE…</p>
          <p v-else-if="editorScanState === 'failed'" class="editor-menu-message editor-menu-error" role="alert">检测失败：{{ editorScanError }}</p>
          <p v-else-if="installedEditors.length === 0" class="editor-menu-message">未检测到 IDE</p>
          <button v-for="editor in installedEditors" v-else :key="editor.id" type="button" role="menuitem" class="editor-menu-option" @click="openProjectInEditor(editor.id)">
            <img class="editor-menu-icon" :src="editorIcons[editor.id]" alt="" aria-hidden="true" />
            {{ editor.name }}
          </button>
          <div class="editor-menu-divider" role="separator" />
          <button class="editor-menu-refresh" type="button" role="menuitem" :disabled="editorScanState === 'loading'" @click="scanInstalledEditors()">
            <RefreshCw :size="13" :class="{ spinning: editorScanState === 'loading' }" />重新扫描
          </button>
        </div>
      </div>
      <div class="project-context-divider" role="separator" />
      <button type="button" role="menuitem" @click="emit('editProject', contextMenu.projectId); closeContextMenu()">
        <Pencil :size="15" />编辑项目
      </button>
      <button class="danger" type="button" role="menuitem" @click="emit('deleteProject', contextMenu.projectId); closeContextMenu()">
        <Trash2 :size="15" />删除项目
      </button>
    </div>
    <div class="dock-spacer" />
    <button
      v-if="collapsed"
      class="dock-action dock-expand"
      type="button"
      title="展开侧栏"
      aria-label="展开侧栏"
      :aria-expanded="!collapsed"
      @click="emit('toggleCollapse')"
    >
      <PanelLeftOpen :size="18" />
    </button>
    <template v-else>
      <button class="dock-action" type="button" title="添加项目" aria-label="添加项目" @click="emit('addProject')">
        <FolderPlus :size="18" />
      </button>
      <button class="dock-action" type="button" title="设置" aria-label="设置" @click="emit('openSettings')">
        <Settings2 :size="18" />
      </button>
    </template>
  </aside>
</template>
