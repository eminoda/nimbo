<script setup lang="ts">
import { X } from '@lucide/vue'

defineProps<{ closeToTray: boolean; configPath: string }>()
const emit = defineEmits<{ updateCloseToTray: [enabled: boolean]; openConfig: []; close: [] }>()
</script>

<template>
  <div class="dialog-backdrop" @click.self="emit('close')">
    <section class="dialog-card settings-dialog" role="dialog" aria-modal="true" aria-label="设置">
      <header class="dialog-header"><div><p class="eyebrow">PREFERENCES</p><h2>设置</h2></div><button class="icon-button" type="button" aria-label="关闭" @click="emit('close')"><X :size="19" /></button></header>
      <label class="setting-row"><span><strong>关闭时最小化到托盘</strong><small>后台命令保持运行，可从系统托盘重新打开窗口。</small></span><input type="checkbox" :checked="closeToTray" @change="emit('updateCloseToTray', ($event.target as HTMLInputElement).checked)" /></label>
      <button class="secondary-button config-file-button" type="button" aria-label="打开本地配置文件" title="打开本地配置文件" :disabled="!configPath" @click="emit('openConfig')">
        <code>{{ configPath }}</code>
      </button>
    </section>
  </div>
</template>
