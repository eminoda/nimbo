<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { FolderOpen, X } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import type { ProjectBookmark } from '@/types/project'

const props = defineProps<{ project?: ProjectBookmark }>()
const emit = defineEmits<{ save: [project: ProjectBookmark]; cancel: [] }>()
const name = shallowRef(props.project?.name ?? '')
const path = shallowRef(props.project?.path ?? '')
const initials = shallowRef(props.project?.initials ?? '')
const error = shallowRef('')
const title = computed(() => props.project ? '编辑项目' : '添加项目')

async function chooseDirectory() {
  const result = await open({ directory: true, multiple: false, title: '选择项目目录' })
  if (typeof result !== 'string') return
  path.value = result
  const segments = result.split(/[\\/]/).filter(Boolean)
  if (!name.value.trim()) name.value = segments[segments.length - 1] ?? ''
}

function save() {
  if (!name.value.trim()) { error.value = '请输入项目名称'; return }
  if (!path.value.trim()) { error.value = '请选择项目目录'; return }
  emit('save', {
    id: props.project?.id ?? crypto.randomUUID(),
    name: name.value.trim(),
    path: path.value,
    initials: initials.value.trim().slice(0, 3).toUpperCase() || undefined,
    iconPath: props.project?.iconPath,
    commands: props.project?.commands ?? [],
  })
}
</script>

<template>
  <div class="dialog-backdrop" @click.self="emit('cancel')">
    <form class="dialog-card" role="dialog" aria-modal="true" :aria-label="title" @submit.prevent="save">
      <header class="dialog-header"><div><p class="eyebrow">PROJECT</p><h2>{{ title }}</h2></div><button class="icon-button" type="button" aria-label="关闭" @click="emit('cancel')"><X :size="19" /></button></header>
      <label class="field-label">项目名称<input v-model="name" autofocus maxlength="64" placeholder="例如 Admin Web" /></label>
      <label class="field-label">项目目录<div class="field-with-button"><input v-model="path" placeholder="选择一个本地项目文件夹" /><button class="secondary-button folder-button" type="button" @click="chooseDirectory"><FolderOpen :size="16" />选择目录</button></div></label>
      <label class="field-label">侧栏缩写 <span class="field-note">可选，最多 3 个字符</span><input v-model="initials" maxlength="3" placeholder="由项目名称自动生成" /></label>
      <p v-if="error" class="form-error" role="alert">{{ error }}</p>
      <footer class="dialog-actions"><button class="secondary-button" type="button" @click="emit('cancel')">取消</button><button class="primary-button" type="submit">保存项目</button></footer>
    </form>
  </div>
</template>
