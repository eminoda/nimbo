import { computed, ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { nimboApi } from '@/services/tauri'
import type { NimboSettings, ProjectBookmark } from '@/types/project'

const emptySettings = (): NimboSettings => ({
  schemaVersion: 1,
  projects: [],
  projectOrder: [],
  selectedCommandByProject: {},
  closeToTray: true,
})

export const useProjectsStore = defineStore('projects', () => {
  const settings = ref<NimboSettings>(emptySettings())
  const initialized = shallowRef(false)
  const projects = computed(() => {
    const position = new Map(settings.value.projectOrder.map((id, index) => [id, index]))
    return [...settings.value.projects].sort((a, b) =>
      (position.get(a.id) ?? Number.MAX_SAFE_INTEGER) - (position.get(b.id) ?? Number.MAX_SAFE_INTEGER),
    )
  })
  const selectedProject = computed(() =>
    settings.value.projects.find((project) => project.id === settings.value.selectedProjectId),
  )
  const selectedCommandId = computed(() => {
    const projectId = selectedProject.value?.id
    return projectId ? settings.value.selectedCommandByProject[projectId] : undefined
  })

  async function initialize() {
    if (initialized.value) return
    settings.value = await nimboApi.loadSettings()
    if (!settings.value.selectedProjectId && settings.value.projectOrder[0]) {
      settings.value.selectedProjectId = settings.value.projectOrder[0]
    }
    initialized.value = true
  }

  async function saveProject(project: ProjectBookmark) {
    await nimboApi.saveProject(project)
    const index = settings.value.projects.findIndex((item) => item.id === project.id)
    settings.value.projects = index < 0
      ? [...settings.value.projects, project]
      : settings.value.projects.map((item) => item.id === project.id ? project : item)
    if (index < 0) settings.value.projectOrder = [...settings.value.projectOrder, project.id]
    settings.value.selectedProjectId = project.id
    const selectedCommand = settings.value.selectedCommandByProject[project.id]
    if (!project.commands.some((command) => command.id === selectedCommand)) {
      if (project.commands[0]) settings.value.selectedCommandByProject[project.id] = project.commands[0].id
      else delete settings.value.selectedCommandByProject[project.id]
    }
  }

  async function deleteProject(projectId: string) {
    await nimboApi.deleteProject(projectId)
    settings.value.projects = settings.value.projects.filter((project) => project.id !== projectId)
    settings.value.projectOrder = settings.value.projectOrder.filter((id) => id !== projectId)
    const { [projectId]: _removed, ...remainingCommands } = settings.value.selectedCommandByProject
    settings.value.selectedCommandByProject = remainingCommands
    if (settings.value.selectedProjectId === projectId) {
      settings.value.selectedProjectId = settings.value.projectOrder[0]
    }
  }

  async function selectProject(projectId: string) {
    await nimboApi.selectProject(projectId)
    settings.value.selectedProjectId = projectId
  }

  async function reorderProjects(projectOrder: string[]) {
    const previousOrder = settings.value.projectOrder
    settings.value.projectOrder = projectOrder
    try {
      await nimboApi.reorderProjects(projectOrder)
    } catch (error) {
      settings.value.projectOrder = previousOrder
      throw error
    }
  }

  async function selectCommand(projectId: string, commandId: string) {
    await nimboApi.selectCommand(projectId, commandId)
    settings.value.selectedCommandByProject[projectId] = commandId
  }

  async function setCloseToTray(enabled: boolean) {
    await nimboApi.setCloseToTray(enabled)
    settings.value.closeToTray = enabled
  }

  return {
    settings, projects, selectedProject, selectedCommandId, initialized,
    initialize, saveProject, deleteProject, selectProject, reorderProjects, selectCommand, setCloseToTray,
  }
})
