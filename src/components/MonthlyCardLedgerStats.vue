<script setup lang="ts">
import type { MonthlyCardLedgerStats } from "../types";

defineProps<{
  stats: MonthlyCardLedgerStats | null;
}>();

const formatMoney = (n: number) => `¥${n.toFixed(2)}`;
</script>

<template>
  <div class="bg-white rounded-lg shadow p-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div>
        <h2 class="text-base font-semibold text-slate-800">筛选统计</h2>
        <p class="text-xs text-slate-500 mt-0.5">
          根据当前搜索条件统计的租金、管理费及费用总计。
        </p>
      </div>
      <div v-if="stats" class="flex items-center gap-4 sm:gap-6">
        <div class="text-right">
          <p class="text-xs text-slate-500">记录数</p>
          <p class="text-lg font-bold text-slate-800">{{ stats.totalCount.toLocaleString() }}</p>
        </div>
        <div class="text-right">
          <p class="text-xs text-slate-500">租金合计</p>
          <p class="text-lg font-bold text-blue-600">{{ formatMoney(stats.totalRent) }}</p>
        </div>
        <div class="text-right">
          <p class="text-xs text-slate-500">管理费合计</p>
          <p class="text-lg font-bold text-emerald-600">{{ formatMoney(stats.totalManagementFee) }}</p>
        </div>
        <div class="text-right">
          <p class="text-xs text-slate-500">费用总计</p>
          <p class="text-lg font-bold text-rose-600">{{ formatMoney(stats.totalAmount) }}</p>
        </div>
      </div>
      <div v-else class="text-sm text-slate-400">
        暂无数据
      </div>
    </div>
  </div>
</template>
