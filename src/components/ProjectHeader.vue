<script setup lang="ts">
import { MoreHorizontal } from '@lucide/vue'
const emit = defineEmits<{ edit: []; delete: [] }>()
</script>

<template>
  <div class="project-header">
    <div class="project-menu">
      <button class="icon-button" type="button" aria-label="项目选项" title="项目选项" @click="emit('edit')">
        <MoreHorizontal :size="22" />
      </button>
      <button class="text-button danger-text" type="button" @click="emit('delete')">删除项目</button>
    </div>
  </div>
</template>
