<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { getStats } from "../api";
import type { StatsResult } from "../types";

const props = defineProps<{
  refreshKey: number;
}>();

const stats = ref<StatsResult | null>(null);
const loading = ref(false);
const error = ref("");

const formatMoney = (n: number) => `¥${n.toFixed(2)}`;

async function load() {
  loading.value = true;
  error.value = "";
  try {
    stats.value = await getStats();
  } catch (err) {
    error.value = `加载统计失败：${err instanceof Error ? err.message : String(err)}`;
  } finally {
    loading.value = false;
  }
}

watch(() => props.refreshKey, load);
onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div v-if="loading" class="text-center py-12 text-slate-500">
      <div class="inline-block w-8 h-8 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin mb-3"></div>
      <p>正在计算统计数据...</p>
    </div>

    <div v-else-if="error" class="bg-amber-50 border border-amber-200 rounded-lg p-4 text-amber-800 text-sm">
      {{ error }}
    </div>

    <template v-else-if="stats">
      <div class="flex justify-end">
        <button
          @click="load"
          :disabled="loading"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
        >
          刷新统计
        </button>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="bg-white rounded-lg shadow p-5">
          <p class="text-sm text-slate-500">总记录数</p>
          <p class="text-2xl font-bold text-slate-800 mt-1">
            {{ stats.summary.totalCount.toLocaleString() }}
          </p>
        </div>
        <div class="bg-white rounded-lg shadow p-5">
          <p class="text-sm text-slate-500">应收总额</p>
          <p class="text-2xl font-bold text-blue-600 mt-1">
            {{ formatMoney(stats.summary.totalDue) }}
          </p>
        </div>
        <div class="bg-white rounded-lg shadow p-5">
          <p class="text-sm text-slate-500">实收总额</p>
          <p class="text-2xl font-bold text-emerald-600 mt-1">
            {{ formatMoney(stats.summary.totalPaid) }}
          </p>
        </div>
        <div class="bg-white rounded-lg shadow p-5">
          <p class="text-sm text-slate-500">未收金额</p>
          <p class="text-2xl font-bold text-rose-600 mt-1">
            {{ formatMoney(stats.summary.totalUnpaid) }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="bg-white rounded-lg shadow p-5">
          <h3 class="font-semibold text-slate-800 mb-4">车辆类型分布</h3>
          <table class="w-full text-sm">
            <thead class="text-slate-500 border-b">
              <tr>
                <th class="text-left py-2">类型</th>
                <th class="text-right py-2">数量</th>
                <th class="text-right py-2">应收</th>
                <th class="text-right py-2">实收</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in stats.vehicleTypeDistribution.slice(0, 10)" :key="item.name" class="border-b last:border-0">
                <td class="py-2">{{ item.name }}</td>
                <td class="text-right">{{ item.count.toLocaleString() }}</td>
                <td class="text-right">{{ formatMoney(item.due) }}</td>
                <td class="text-right">{{ formatMoney(item.paid) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="bg-white rounded-lg shadow p-5">
          <h3 class="font-semibold text-slate-800 mb-4">放行方式分布</h3>
          <table class="w-full text-sm">
            <thead class="text-slate-500 border-b">
              <tr>
                <th class="text-left py-2">放行方式</th>
                <th class="text-right py-2">数量</th>
                <th class="text-right py-2">应收</th>
                <th class="text-right py-2">实收</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in stats.releaseMethodDistribution" :key="item.name" class="border-b last:border-0">
                <td class="py-2">{{ item.name }}</td>
                <td class="text-right">{{ item.count.toLocaleString() }}</td>
                <td class="text-right">{{ formatMoney(item.due) }}</td>
                <td class="text-right">{{ formatMoney(item.paid) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="bg-white rounded-lg shadow p-5">
          <h3 class="font-semibold text-slate-800 mb-4">缴费方式分布</h3>
          <table class="w-full text-sm">
            <thead class="text-slate-500 border-b">
              <tr>
                <th class="text-left py-2">缴费方式</th>
                <th class="text-right py-2">数量</th>
                <th class="text-right py-2">应收</th>
                <th class="text-right py-2">实收</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in stats.paymentMethodDistribution" :key="item.name" class="border-b last:border-0">
                <td class="py-2">{{ item.name || "未指定" }}</td>
                <td class="text-right">{{ item.count.toLocaleString() }}</td>
                <td class="text-right">{{ formatMoney(item.due) }}</td>
                <td class="text-right">{{ formatMoney(item.paid) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="bg-white rounded-lg shadow p-5">
          <h3 class="font-semibold text-slate-800 mb-4">收费车型分布</h3>
          <table class="w-full text-sm">
            <thead class="text-slate-500 border-b">
              <tr>
                <th class="text-left py-2">收费车型</th>
                <th class="text-right py-2">数量</th>
                <th class="text-right py-2">应收</th>
                <th class="text-right py-2">实收</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in stats.chargeTypeDistribution" :key="item.name" class="border-b last:border-0">
                <td class="py-2">{{ item.name }}</td>
                <td class="text-right">{{ item.count.toLocaleString() }}</td>
                <td class="text-right">{{ formatMoney(item.due) }}</td>
                <td class="text-right">{{ formatMoney(item.paid) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="bg-white rounded-lg shadow p-5">
        <h3 class="font-semibold text-slate-800 mb-4">月度收费趋势</h3>
        <div class="overflow-auto max-h-96">
          <table class="w-full text-sm">
            <thead class="bg-slate-100 text-slate-600 sticky top-0">
              <tr>
                <th class="text-left px-4 py-2">月份</th>
                <th class="text-right px-4 py-2">记录数</th>
                <th class="text-right px-4 py-2">应收金额</th>
                <th class="text-right px-4 py-2">实收金额</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr v-for="item in stats.monthlyTrend" :key="item.date">
                <td class="px-4 py-2">{{ item.date }}</td>
                <td class="text-right px-4 py-2">{{ item.count.toLocaleString() }}</td>
                <td class="text-right px-4 py-2">{{ formatMoney(item.due) }}</td>
                <td class="text-right px-4 py-2">{{ formatMoney(item.paid) }}</td>
              </tr>
              <tr v-if="stats.monthlyTrend.length === 0">
                <td colspan="4" class="px-4 py-8 text-center text-slate-400">无时间数据</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="bg-white rounded-lg shadow p-5">
        <h3 class="font-semibold text-slate-800 mb-4">每日收费趋势（最近 30 天）</h3>
        <div class="overflow-auto max-h-96">
          <table class="w-full text-sm">
            <thead class="bg-slate-100 text-slate-600 sticky top-0">
              <tr>
                <th class="text-left px-4 py-2">日期</th>
                <th class="text-right px-4 py-2">记录数</th>
                <th class="text-right px-4 py-2">应收金额</th>
                <th class="text-right px-4 py-2">实收金额</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr v-for="item in stats.dailyTrend" :key="item.date">
                <td class="px-4 py-2">{{ item.date }}</td>
                <td class="text-right px-4 py-2">{{ item.count.toLocaleString() }}</td>
                <td class="text-right px-4 py-2">{{ formatMoney(item.due) }}</td>
                <td class="text-right px-4 py-2">{{ formatMoney(item.paid) }}</td>
              </tr>
              <tr v-if="stats.dailyTrend.length === 0">
                <td colspan="4" class="px-4 py-8 text-center text-slate-400">无时间数据</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>
