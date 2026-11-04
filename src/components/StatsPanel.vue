<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { getStats, getTopUnpaidVehicles } from "../api";
import type { StatsResult, TopUnpaidVehicle } from "../types";

const props = defineProps<{
  refreshKey: number;
}>();

const stats = ref<StatsResult | null>(null);
const loading = ref(false);
const error = ref("");

const topUnpaid = ref<TopUnpaidVehicle[]>([]);
const topTotal = ref(0);
const topPage = ref(1);
const topPerPage = ref(20);
const topLimit = ref(10);
const topLoading = ref(false);
const topError = ref("");

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

async function loadTopUnpaid() {
  topLoading.value = true;
  topError.value = "";
  try {
    const result = await getTopUnpaidVehicles({
      limit: topLimit.value || 10,
      page: topPage.value,
      perPage: topPerPage.value,
    });
    topUnpaid.value = result.items;
    topTotal.value = result.total;
  } catch (err) {
    topError.value = `加载欠费排名失败：${err instanceof Error ? err.message : String(err)}`;
  } finally {
    topLoading.value = false;
  }
}

function applyTopLimit() {
  topPage.value = 1;
  loadTopUnpaid();
}

function setTopPage(p: number) {
  topPage.value = p;
  loadTopUnpaid();
}

function onTopPerPageChange(e: Event) {
  topPerPage.value = Number((e.target as HTMLSelectElement).value);
  topPage.value = 1;
  loadTopUnpaid();
}

watch(() => props.refreshKey, () => {
  load();
  loadTopUnpaid();
});
onMounted(() => {
  load();
  loadTopUnpaid();
});
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

      <div class="bg-white rounded-lg shadow p-5">
        <div class="flex items-center justify-between mb-4 flex-wrap gap-3">
          <h3 class="font-semibold text-slate-800">欠费车辆排名</h3>
          <div class="flex items-center gap-2">
            <label class="text-sm text-slate-500">显示前</label>
            <input
              v-model.number="topLimit"
              type="number"
              min="1"
              class="w-20 px-2 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              @keyup.enter="applyTopLimit"
            />
            <label class="text-sm text-slate-500">名</label>
            <button
              @click="applyTopLimit"
              :disabled="topLoading"
              class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
            >
              查询
            </button>
          </div>
        </div>

        <div v-if="topLoading" class="text-center py-8 text-slate-500">
          <div class="inline-block w-6 h-6 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin mb-2"></div>
          <p class="text-sm">正在加载欠费排名...</p>
        </div>

        <div v-else-if="topError" class="bg-amber-50 border border-amber-200 rounded-lg p-3 text-amber-800 text-sm mb-4">
          {{ topError }}
        </div>

        <template v-else>
          <div class="overflow-auto max-h-96">
            <table class="w-full text-sm">
              <thead class="bg-slate-100 text-slate-600 sticky top-0">
                <tr>
                  <th class="text-left px-4 py-2">排名</th>
                  <th class="text-left px-4 py-2">车牌号</th>
                  <th class="text-right px-4 py-2">记录数</th>
                  <th class="text-right px-4 py-2">应收金额</th>
                  <th class="text-right px-4 py-2">实收金额</th>
                  <th class="text-right px-4 py-2">欠费金额</th>
                </tr>
              </thead>
              <tbody class="divide-y">
                <tr v-for="(item, index) in topUnpaid" :key="item.plate">
                  <td class="px-4 py-2 font-medium text-slate-700">
                    {{ (topPage - 1) * topPerPage + index + 1 }}
                  </td>
                  <td class="px-4 py-2">{{ item.plate }}</td>
                  <td class="text-right px-4 py-2">{{ item.count.toLocaleString() }}</td>
                  <td class="text-right px-4 py-2">{{ formatMoney(item.due) }}</td>
                  <td class="text-right px-4 py-2">{{ formatMoney(item.paid) }}</td>
                  <td class="text-right px-4 py-2 font-medium text-rose-600">{{ formatMoney(item.unpaid) }}</td>
                </tr>
                <tr v-if="topUnpaid.length === 0">
                  <td colspan="6" class="px-4 py-8 text-center text-slate-400">暂无欠费车辆</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="flex items-center justify-between pt-4">
            <span class="text-sm text-slate-500">
              第 {{ topPage }} 页，共 {{ topTotal }} 条
            </span>
            <div class="flex items-center gap-2">
              <select
                :value="topPerPage"
                @change="onTopPerPageChange"
                class="px-3 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option :value="10">10 条/页</option>
                <option :value="20">20 条/页</option>
                <option :value="50">50 条/页</option>
                <option :value="100">100 条/页</option>
              </select>
              <button
                @click="setTopPage(topPage - 1)"
                :disabled="topPage === 1"
                class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
              >
                上一页
              </button>
              <button
                @click="setTopPage(topPage + 1)"
                :disabled="topPage * topPerPage >= topTotal"
                class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
              >
                下一页
              </button>
            </div>
          </div>
        </template>
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
