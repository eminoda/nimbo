<script setup lang="ts">
import { X } from '@lucide/vue'

defineProps<{ closeToTray: boolean }>()
const emit = defineEmits<{ updateCloseToTray: [enabled: boolean]; close: [] }>()
</script>

<template>
  <div class="dialog-backdrop" @click.self="emit('close')">
    <section class="dialog-card settings-dialog" role="dialog" aria-modal="true" aria-label="设置">
      <header class="dialog-header"><div><p class="eyebrow">PREFERENCES</p><h2>设置</h2></div><button class="icon-button" type="button" aria-label="关闭" @click="emit('close')"><X :size="19" /></button></header>
      <label class="setting-row"><span><strong>关闭时最小化到托盘</strong><small>后台命令保持运行，可从系统托盘重新打开窗口。</small></span><input type="checkbox" :checked="closeToTray" @change="emit('updateCloseToTray', ($event.target as HTMLInputElement).checked)" /></label>
      <div class="about-card"><span class="about-logo"><span>N</span></span><span><strong>Nimbo Terminal</strong><small>本地开发服务工作区</small></span><span class="version-tag">MVP</span></div>
    </section>
  </div>
</template>
