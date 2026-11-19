<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, toRaw, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openPath } from '@tauri-apps/plugin-opener'
import { Minus, Square, X, Command, PanelLeftClose } from '@lucide/vue'
import nimboLogo from './assets/nimbo-logo.svg'
import CommandEditor from '@/components/CommandEditor.vue'
import CommandTabs from '@/components/CommandTabs.vue'
import CommandToolbar from '@/components/CommandToolbar.vue'
import ProjectDock from '@/components/ProjectDock.vue'
import ProjectEditor from '@/components/ProjectEditor.vue'
import BranchSwitcher from '@/components/BranchSwitcher.vue'
import SettingsPanel from '@/components/SettingsPanel.vue'
import TerminalPane from '@/components/TerminalPane.vue'
import { useProjectsStore } from '@/stores/projects'
import { useSessionsStore } from '@/stores/sessions'
import { nimboApi } from '@/services/tauri'
import type { CommandPreset, ProjectBookmark } from '@/types/project'
import type { SessionStatus } from '@/types/session'

const projectsStore = useProjectsStore()
const sessionsStore = useSessionsStore()
const projectDialogOpen = shallowRef(false)
const editingProject = shallowRef<ProjectBookmark>()
const commandDialogOpen = shallowRef(false)
const editingCommand = shallowRef<CommandPreset>()
const settingsOpen = shallowRef(false)
const settingsFilePath = shallowRef('')
const toast = shallowRef('')
let toastTimer: ReturnType<typeof setTimeout> | undefined

const project = computed(() => projectsStore.selectedProject)
const command = computed(() => project.value?.commands.find((item) => item.id === projectsStore.selectedCommandId)
  ?? project.value?.commands[0])
const session = computed(() => sessionsStore.sessions.find((item) =>
  item.projectId === project.value?.id && item.commandId === command.value?.id,
))
const status = computed<SessionStatus>(() => session.value?.status ?? 'stopped')
const logChunks = computed(() => session.value ? sessionsStore.logs[session.value.sessionId] ?? [] : [])
const statuses = computed<Record<string, SessionStatus>>(() => Object.fromEntries(
  (project.value?.commands ?? []).map((item) => [
    item.id,
    sessionsStore.sessions.find((running) => running.projectId === project.value?.id && running.commandId === item.id)?.status ?? 'stopped',
  ]),
))
const runningProjectIds = computed(() => [...new Set(sessionsStore.sessions
  .filter((item) => item.status === 'running' || item.status === 'starting' || item.status === 'stopping')
  .map((item) => item.projectId))])

function showToast(message: string) {
  toast.value = message
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toast.value = '' }, 3_500)
}

async function run(action: () => Promise<unknown>) {
  try { await action() }
  catch (error) { showToast(error instanceof Error ? error.message : String(error)) }
}

onMounted(async () => {
  try {
    await Promise.all([projectsStore.initialize(), sessionsStore.initialize()])
  } catch (error) {
    showToast(error instanceof Error ? error.message : String(error))
  }
})

watch(() => sessionsStore.error, (value) => { if (value) showToast(value) })
onBeforeUnmount(() => {
  if (toastTimer) clearTimeout(toastTimer)
  sessionsStore.dispose()
})

function openProjectEditor(target?: ProjectBookmark) {
  editingProject.value = target ? structuredClone(toRaw(target)) : undefined
  projectDialogOpen.value = true
}

async function saveProject(value: ProjectBookmark) {
  await run(async () => {
    await projectsStore.saveProject(value)
    projectDialogOpen.value = false
    editingProject.value = undefined
  })
}

function openProjectById(projectId: string) {
  const target = projectsStore.projects.find((item) => item.id === projectId)
  if (target) openProjectEditor(target)
}

async function removeProject(projectId: string) {
  const target = projectsStore.projects.find((item) => item.id === projectId)
  if (!target) return
  if (sessionsStore.sessions.some((item) =>
    item.projectId === projectId && ['running', 'starting', 'stopping'].includes(item.status),
  )) {
    showToast('请先停止此项目中的运行命令，再删除项目。')
    return
  }
  if (!window.confirm('删除“' + target.name + '”书签？本地文件夹不会被删除。')) return
  await run(async () => {
    await projectsStore.deleteProject(projectId)
    sessionsStore.forgetProject(projectId)
  })
}

function openCommandEditor(target?: CommandPreset) {
  editingCommand.value = target ? structuredClone(toRaw(target)) : undefined
  commandDialogOpen.value = true
}

async function saveCommand(value: CommandPreset) {
  const current = project.value
  if (!current) return
  const next: ProjectBookmark = {
    ...current,
    commands: editingCommand.value
      ? current.commands.map((item) => item.id === value.id ? value : item)
      : [...current.commands, value],
  }
  await run(async () => {
    await projectsStore.saveProject(next)
    await projectsStore.selectCommand(current.id, value.id)
    commandDialogOpen.value = false
    editingCommand.value = undefined
  })
}

async function removeCommand(commandId: string) {
  const currentProject = project.value
  const targetCommand = currentProject?.commands.find((item) => item.id === commandId)
  if (!currentProject || !targetCommand) return
  const targetSession = sessionsStore.sessions.find((item) =>
    item.projectId === currentProject.id && item.commandId === commandId,
  )
  if (targetSession && ['running', 'starting', 'stopping'].includes(targetSession.status)) {
    showToast('请先停止此命令，再删除它。')
    return
  }
  if (!window.confirm(`删除命令“${targetCommand.name}”？`)) return
  const next = { ...currentProject, commands: currentProject.commands.filter((item) => item.id !== commandId) }
  await run(() => projectsStore.saveProject(next))
}

async function startCommand() {
  if (!project.value || !command.value) return
  await run(() => sessionsStore.start(project.value!.id, command.value!.id))
}

async function stopCommand() {
  if (session.value) await run(() => sessionsStore.stop(session.value!.sessionId))
}

async function reloadCommand() {
  if (!project.value || !command.value) return
  if (session.value) await run(() => sessionsStore.reload(session.value!.sessionId))
  else await startCommand()
}

async function selectCommand(id: string) {
  if (project.value) await run(() => projectsStore.selectCommand(project.value!.id, id))
}

async function selectProject(id: string) {
  await run(() => projectsStore.selectProject(id))
}

async function minimizeWindow() { await run(() => getCurrentWindow().minimize()) }
async function toggleWindowSize() { await run(() => getCurrentWindow().toggleMaximize()) }
async function closeWindow() { await run(() => getCurrentWindow().close()) }
async function openSettingsPanel() {
  settingsOpen.value = true
  await run(async () => { settingsFilePath.value = await nimboApi.settingsFilePath() })
}
async function openSettingsFile() {
  await run(async () => {
    const path = settingsFilePath.value || await nimboApi.settingsFilePath()
    settingsFilePath.value = path
    await openPath(path)
  })
}
</script>

<template>
  <main class="app-frame">
    <header class="titlebar" data-tauri-drag-region @dblclick="toggleWindowSize">
      <div class="titlebar-brand" data-tauri-drag-region><span class="brand-icon"><img class="brand-logo" :src="nimboLogo" alt="" aria-hidden="true" /></span><strong>Nimbo Terminal</strong><span class="titlebar-separator">/</span><span class="titlebar-section">Workspace</span></div>
      <div class="titlebar-controls">
        <button type="button" title="最小化" aria-label="最小化" @mousedown.stop @click="minimizeWindow"><Minus :size="17" /></button>
        <button type="button" title="最大化 / 还原" aria-label="最大化或还原" @mousedown.stop @click="toggleWindowSize"><Square :size="16" /></button>
        <button class="close-control" type="button" title="关闭到托盘" aria-label="关闭" @mousedown.stop @click="closeWindow"><X :size="18" /></button>
      </div>
    </header>

    <div class="workspace">
      <ProjectDock
        :projects="projectsStore.projects"
        :selected-project-id="project?.id"
        :running-project-ids="runningProjectIds"
        @select="selectProject"
        @edit-project="openProjectById"
        @delete-project="removeProject"
        @add-project="openProjectEditor()"
        @open-settings="openSettingsPanel"
      />

      <section v-if="project" class="project-workspace">
        <div class="command-tabs-row">
          <CommandTabs
            :commands="project.commands"
            :selected-command-id="command?.id"
            :statuses="statuses"
            @select="selectCommand"
            @add="openCommandEditor()"
            @edit="openCommandEditor(command)"
            @remove="removeCommand"
          />
          <BranchSwitcher :key="project.id" :project-id="project.id" :busy="runningProjectIds.includes(project.id)" />
        </div>
        <CommandToolbar
          :status="status"
          :command-name="command?.name"
          @start="startCommand"
          @stop="stopCommand"
          @reload="reloadCommand"
          @clear="session && sessionsStore.clear(session.sessionId)"
        />
        <TerminalPane
          :session-id="session?.sessionId"
          :chunks="logChunks"
          :clear-version="sessionsStore.clearVersion"
          :running="status === 'running' || status === 'starting'"
          @interrupt="stopCommand"
        />
      </section>

      <section v-else class="welcome-workspace">
        <div class="welcome-card">
          <span class="welcome-icon"><Command :size="28" /></span>
          <p class="eyebrow">LOCAL WORKSPACE</p>
          <h1>开发环境，整齐一点。</h1>
          <p class="welcome-copy">把常用项目和启动命令放在一起。服务在后台持续运行，切换项目也会保留各自的日志。</p>
          <button class="primary-button" type="button" @click="openProjectEditor()">添加第一个项目</button>
        </div>
        <footer class="workspace-footer"><span><PanelLeftClose :size="17" />项目工作区</span><span class="footer-right">Windows 优先 · 本地运行</span></footer>
      </section>
    </div>

    <Transition name="toast"><div v-if="toast" class="toast-message" role="status">{{ toast }}</div></Transition>
    <ProjectEditor v-if="projectDialogOpen" :project="editingProject" @save="saveProject" @cancel="projectDialogOpen = false" />
    <CommandEditor v-if="commandDialogOpen && project" :command="editingCommand" :project-id="project.id" :project-path="project.path" @save="saveCommand" @cancel="commandDialogOpen = false" />
    <SettingsPanel v-if="settingsOpen" :close-to-tray="projectsStore.settings.closeToTray" :config-path="settingsFilePath" @close="settingsOpen = false" @open-config="openSettingsFile" @update-close-to-tray="(value) => run(() => projectsStore.setCloseToTray(value))" />
  </main>
</template>
