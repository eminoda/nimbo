<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, watch } from 'vue'
import { Check, ChevronDown, GitBranch, RefreshCw, Search } from '@lucide/vue'
import { nimboApi } from '@/services/tauri'
import type { GitBranchInfo } from '@/types/git'

const props = defineProps<{ projectId: string; busy: boolean }>()
const isOpen = shallowRef(false)
const isLoading = shallowRef(true)
const isSwitching = shallowRef(false)
const searchQuery = shallowRef('')
const error = shallowRef('')
const branchInfo = shallowRef<GitBranchInfo>({ currentBranch: null, localBranches: [], worktreeBranches: [] })
const rootElement = shallowRef<HTMLDivElement | null>(null)
const filteredBranches = computed(() => {
  const query = searchQuery.value.trim().toLocaleLowerCase()
  if (!query) return branchInfo.value.localBranches
  return branchInfo.value.localBranches.filter((branch) => branch.toLocaleLowerCase().includes(query))
})
const currentLabel = computed(() => {
  if (branchInfo.value.currentBranch) return branchInfo.value.currentBranch
  if (isLoading.value) return '读取中…'
  if (error.value) return 'Git 不可用'
  return 'HEAD 分离状态'
})
let requestVersion = 0

async function refreshBranches() {
  const version = ++requestVersion
  const projectId = props.projectId
  isLoading.value = true
  error.value = ''
  try {
    const result = await nimboApi.listProjectBranches(projectId)
    if (version !== requestVersion) return
    branchInfo.value = result
  } catch (cause) {
    if (version !== requestVersion) return
    error.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    if (version === requestVersion) isLoading.value = false
  }
}

function toggleOpen() {
  isOpen.value = !isOpen.value
  if (!isOpen.value) searchQuery.value = ''
}

async function switchBranch(branchName: string) {
  if (props.busy || isSwitching.value || branchName === branchInfo.value.currentBranch) return
  isSwitching.value = true
  error.value = ''
  try {
    branchInfo.value = await nimboApi.switchProjectBranch(props.projectId, branchName)
    searchQuery.value = ''
    isOpen.value = false
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    isSwitching.value = false
  }
}

function closeOnOutsidePointer(event: PointerEvent) {
  if (isOpen.value && !rootElement.value?.contains(event.target as Node)) isOpen.value = false
}

function closeOnEscape(event: KeyboardEvent) {
  if (event.key === 'Escape') isOpen.value = false
}

watch(isOpen, (open) => {
  if (open) void refreshBranches()
})

watch(() => props.projectId, () => {
  branchInfo.value = { currentBranch: null, localBranches: [], worktreeBranches: [] }
  searchQuery.value = ''
  error.value = ''
  void refreshBranches()
})

onMounted(() => {
  void refreshBranches()
  document.addEventListener('pointerdown', closeOnOutsidePointer)
  document.addEventListener('keydown', closeOnEscape)
})

onBeforeUnmount(() => {
  requestVersion += 1
  document.removeEventListener('pointerdown', closeOnOutsidePointer)
  document.removeEventListener('keydown', closeOnEscape)
})
</script>

<template>
  <div ref="rootElement" class="branch-switcher">
    <button
      class="branch-trigger"
      type="button"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
      :title="busy ? '请先停止此项目中的运行命令，再切换分支' : `当前分支：${currentLabel}`"
      @click="toggleOpen"
    >
      <GitBranch :size="15" />
      <span>{{ currentLabel }}</span>
      <ChevronDown :size="13" />
    </button>

    <div v-if="isOpen" class="branch-popover" @pointerdown.stop>
      <div class="branch-popover-heading">
        <div class="branch-current-label">
          <GitBranch :size="14" />
          <span>当前分支</span>
        </div>
        <strong :title="currentLabel">{{ currentLabel }}</strong>
      </div>

      <div class="branch-search">
        <Search :size="14" />
        <input v-model="searchQuery" type="search" placeholder="搜索本地分支…" aria-label="搜索本地分支" />
      </div>

      <p v-if="busy" class="branch-notice">请先停止此项目中的运行命令，再切换分支。</p>
      <p v-if="error" class="branch-error" role="alert">{{ error }}</p>
      <p v-else-if="isLoading" class="branch-empty" role="status">正在读取分支…</p>
      <div v-else-if="filteredBranches.length" class="branch-list" role="listbox" aria-label="本地分支">
        <button
          v-for="branch in filteredBranches"
          :key="branch"
          class="branch-option"
          :class="{ current: branch === branchInfo.currentBranch }"
          type="button"
          role="option"
          :aria-selected="branch === branchInfo.currentBranch"
          :disabled="busy || isSwitching || branch === branchInfo.currentBranch"
          :title="busy ? '请先停止此项目中的运行命令，再切换分支' : branch"
          @click="switchBranch(branch)"
        >
          <span class="branch-option-name">{{ branch }}</span>
          <span v-if="branchInfo.worktreeBranches.includes(branch)" class="branch-worktree-badge" title="已在其他 worktree 中检出">Worktree</span>
          <Check v-if="branch === branchInfo.currentBranch" :size="14" />
          <span v-else-if="isSwitching" class="branch-option-hint">切换中</span>
        </button>
      </div>
      <p v-else-if="branchInfo.localBranches.length" class="branch-empty">没有匹配的本地分支</p>
      <p v-else class="branch-empty">此目录没有可用的本地 Git 分支</p>

      <footer class="branch-popover-footer">
        <span>本地分支</span>
        <button class="branch-refresh" type="button" :disabled="isLoading || isSwitching" @click="refreshBranches">
          <RefreshCw :size="13" :class="{ spinning: isLoading }" />刷新
        </button>
      </footer>
    </div>
  </div>
</template>
