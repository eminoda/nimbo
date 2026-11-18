<script setup lang="ts">
import { onMounted, shallowRef, watch } from 'vue'
import { RefreshCw, X } from '@lucide/vue'
import { nimboApi } from '@/services/tauri'
import type { CommandPreset, PackageScript } from '@/types/project'

const props = defineProps<{ command?: CommandPreset; projectId: string; projectPath: string }>()
const emit = defineEmits<{ save: [command: CommandPreset]; cancel: [] }>()
const name = shallowRef(props.command?.name ?? '')
const commandLine = shallowRef(props.command?.commandLine ?? '')
const cwd = shallowRef(props.command?.cwd ?? '')
const autoStart = shallowRef(props.command?.autoStart ?? false)
const envText = shallowRef(Object.entries(props.command?.env ?? {}).map(([key, value]) => `${key}=${value}`).join('\n'))
const error = shallowRef('')
const packageScripts = shallowRef<PackageScript[]>([])
const packageManager = shallowRef('')
const selectedScriptName = shallowRef('')
const scriptsLoading = shallowRef(false)
const scriptsError = shallowRef('')

onMounted(() => { void loadPackageScripts() })

async function loadPackageScripts() {
  scriptsLoading.value = true
  scriptsError.value = ''
  try {
    const result = await nimboApi.listPackageScripts(props.projectId)
    packageManager.value = result.packageManager
    packageScripts.value = result.scripts
  } catch (cause) {
    packageScripts.value = []
    scriptsError.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    scriptsLoading.value = false
  }
}

function applySelectedScript() {
  const selected = packageScripts.value.find((script) => script.name === selectedScriptName.value)
  if (!selected) return
  name.value = selected.name
  commandLine.value = selected.commandLine
  error.value = ''
}

function markManualCommand() {
  selectedScriptName.value = ''
}

watch(() => props.command, (command) => {
  name.value = command?.name ?? ''
  commandLine.value = command?.commandLine ?? ''
  cwd.value = command?.cwd ?? ''
  autoStart.value = command?.autoStart ?? false
  envText.value = Object.entries(command?.env ?? {}).map(([key, value]) => `${key}=${value}`).join('\n')
  selectedScriptName.value = ''
})

function save() {
  if (!name.value.trim()) { error.value = '请输入命令名称'; return }
  if (!commandLine.value.trim()) { error.value = '请输入要运行的命令'; return }
  const env: Record<string, string> = {}
  for (const row of envText.value.split(/\r?\n/).filter(Boolean)) {
    const separator = row.indexOf('=')
    if (separator < 1) { error.value = `环境变量格式无效：${row}`; return }
    env[row.slice(0, separator).trim()] = row.slice(separator + 1)
  }
  emit('save', {
    id: props.command?.id ?? crypto.randomUUID(),
    name: name.value.trim(),
    commandLine: commandLine.value.trim(),
    cwd: cwd.value.trim() || undefined,
    env,
    kind: props.command?.kind ?? 'service',
    autoStart: autoStart.value,
  })
}
</script>

<template>
  <div class="dialog-backdrop" @click.self="emit('cancel')">
    <form class="dialog-card command-dialog" role="dialog" aria-modal="true" :aria-label="command ? '编辑命令' : '添加命令'" @submit.prevent="save">
      <header class="dialog-header"><div><p class="eyebrow">COMMAND</p><h2>{{ command ? '编辑命令' : '添加命令' }}</h2></div><button class="icon-button" type="button" aria-label="关闭" @click="emit('cancel')"><X :size="19" /></button></header>
      <section class="script-picker" aria-label="package.json 脚本">
        <div class="script-picker-heading">
          <span><strong>选择启动脚本</strong><small v-if="packageManager">{{ packageManager }}</small></span>
          <button class="script-refresh" type="button" :disabled="scriptsLoading" title="重新扫描 scripts" @click="loadPackageScripts">
            <RefreshCw :size="14" :class="{ spinning: scriptsLoading }" />重新扫描
          </button>
        </div>
        <label class="field-label script-select-label">
          <select v-model="selectedScriptName" :disabled="scriptsLoading || packageScripts.length === 0" @change="applySelectedScript">
            <option value="">{{ scriptsLoading ? '正在读取…' : packageScripts.length ? '选择项目脚本' : '没有找到项目脚本' }}</option>
            <option v-for="script in packageScripts" :key="script.name" :value="script.name">{{ script.name }}</option>
          </select>
        </label>
        <p v-if="selectedScriptName" class="script-source">{{ packageScripts.find((script) => script.name === selectedScriptName)?.script }}</p>
        <p v-else-if="scriptsError" class="script-hint">{{ scriptsError }}</p>
        <p v-else-if="!scriptsLoading && packageScripts.length === 0" class="script-hint">也可以在下方手动填写命令。</p>
      </section>
      <label class="field-label">命令名称<input v-model="name" autofocus maxlength="48" placeholder="例如 dev" /></label>
      <label class="field-label">启动命令<input v-model="commandLine" placeholder="例如 pnpm run dev" autocomplete="off" @input="markManualCommand" /></label>
      <details class="advanced-fields"><summary>高级选项</summary>
        <label class="field-label">工作目录 <span class="field-note">默认使用项目目录</span><input v-model="cwd" :placeholder="projectPath" /></label>
        <label class="field-label">环境变量 <span class="field-note">每行一个 KEY=value</span><textarea v-model="envText" rows="3" placeholder="PORT=3000"></textarea></label>
      </details>
      <p v-if="error" class="form-error" role="alert">{{ error }}</p>
      <footer class="dialog-actions"><button class="secondary-button" type="button" @click="emit('cancel')">取消</button><button class="primary-button" type="submit">保存命令</button></footer>
    </form>
  </div>
</template>
