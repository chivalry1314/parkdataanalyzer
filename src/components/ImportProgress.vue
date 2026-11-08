<script setup lang="ts">
defineProps<{
  loading: boolean;
  progress: number;
  progressText: string;
}>();
</script>

<template>
  <div v-if="loading" class="bg-white rounded-lg shadow p-4">
    <div class="flex items-center justify-between text-xs text-slate-600 mb-1">
      <span>正在处理数据...</span>
      <span>{{ progressText }}</span>
    </div>
    <div class="w-full bg-slate-200 rounded-full h-2 overflow-hidden">
      <div
        class="bg-blue-600 h-2 rounded-full transition-all duration-300"
        :style="{ width: `${progress}%` }"
      ></div>
    </div>
  </div>
</template>
