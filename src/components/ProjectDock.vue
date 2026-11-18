<script setup lang="ts">
import { FolderPlus, Settings2 } from '@lucide/vue'
import nimboLogo from '../assets/nimbo-logo.svg'
import type { ProjectBookmark } from '@/types/project'

defineProps<{ projects: ProjectBookmark[]; selectedProjectId?: string; runningProjectIds: string[] }>()
const emit = defineEmits<{
  select: [projectId: string]
  addProject: []
  openSettings: []
}>()
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
      >
        {{ project.initials || project.name.slice(0, 2).toUpperCase() }}
        <span v-if="runningProjectIds.includes(project.id)" class="project-dot" />
      </button>
    </nav>
    <div class="dock-spacer" />
    <button class="dock-action" type="button" title="添加项目" aria-label="添加项目" @click="emit('addProject')">
      <FolderPlus :size="18" />
    </button>
    <button class="dock-action" type="button" title="设置" aria-label="设置" @click="emit('openSettings')">
      <Settings2 :size="18" />
    </button>
  </aside>
</template>
