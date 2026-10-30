<script setup lang="ts">
import type { StatsSummary } from "../types";

defineProps<{
  stats: StatsSummary | null;
  loading: boolean;
  progress: number;
  progressText: string;
}>();

const formatMoney = (n: number) => `¥${n.toFixed(2)}`;
</script>

<template>
  <div class="bg-white rounded-lg shadow p-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div>
        <h2 class="text-base font-semibold text-slate-800">筛选统计</h2>
        <p class="text-xs text-slate-500 mt-0.5">
          根据当前搜索条件统计的应收、实收、未收金额。
        </p>
      </div>
      <div v-if="stats" class="flex items-center gap-4 sm:gap-6">
        <div class="text-right">
          <p class="text-xs text-slate-500">记录数</p>
          <p class="text-lg font-bold text-slate-800">{{ stats.totalCount.toLocaleString() }}</p>
        </div>
        <div class="text-right">
          <p class="text-xs text-slate-500">应收金额</p>
          <p class="text-lg font-bold text-blue-600">{{ formatMoney(stats.totalDue) }}</p>
        </div>
        <div class="text-right">
          <p class="text-xs text-slate-500">实收金额</p>
          <p class="text-lg font-bold text-emerald-600">{{ formatMoney(stats.totalPaid) }}</p>
        </div>
        <div class="text-right">
          <p class="text-xs text-slate-500">未收金额</p>
          <p class="text-lg font-bold text-rose-600">{{ formatMoney(stats.totalUnpaid) }}</p>
        </div>
      </div>
      <div v-else-if="!loading" class="text-sm text-slate-400">
        暂无数据
      </div>
    </div>

    <div v-if="loading" class="mt-3">
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
  </div>
</template>
