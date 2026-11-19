<script setup lang="ts">
import { Pencil, Trash2 } from '@lucide/vue'
const emit = defineEmits<{ edit: []; delete: [] }>()
</script>

<template>
  <div class="project-header">
    <div class="project-menu">
      <button class="icon-button" type="button" aria-label="编辑项目" title="编辑项目" @click="emit('edit')">
        <Pencil :size="16" />
      </button>
      <button class="icon-button delete-project-button" type="button" aria-label="删除项目" title="删除项目" @click="emit('delete')">
        <Trash2 :size="16" />
      </button>
    </div>
  </div>
</template>
