<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { exportUnpaidIntervalStats, getUnpaidIntervalStats } from "../api";
import type { UnpaidIntervalStats } from "../types";

const props = defineProps<{
  mode: "equal-freq" | "equal-width";
}>();

const loading = ref(false);
const exporting = ref(false);
const error = ref("");
const stats = ref<UnpaidIntervalStats | null>(null);
const bins = ref(5);
const filtersExpanded = ref(false);
const timeFilters = ref({
  entryTimeStart: "",
  entryTimeEnd: "",
  exitTimeStart: "",
  exitTimeEnd: "",
});

const binOptions = [3, 5, 7, 10];

const title = computed(() =>
  props.mode === "equal-width" ? "欠费等宽分箱分析" : "欠费等频分箱分析"
);

const methodName = computed(() =>
  props.mode === "equal-width" ? "等宽分箱" : "等频分箱"
);

const methodDescription = computed(() =>
  props.mode === "equal-width"
    ? "将车辆累计欠费金额按固定金额区间均匀划分，每档金额范围相同，便于观察金额边界处的车辆分布。"
    : "将车辆按累计欠费金额排序后，按车辆数平均分配到各区间，保证每档车辆数量大致相同，适用于欠费金额长尾分布的场景。"
);

const formatMoney = (n: number) => `¥${n.toFixed(2)}`;

function formatDateTime(value: string): string {
  if (!value) return "";
  return value.replace("T", " ").replace(/\+.*$/, "").padEnd(19, ":00").slice(0, 19);
}

const maxVehicleCount = computed(() => {
  if (!stats.value) return 0;
  return Math.max(...stats.value.intervals.map((i) => i.vehicleCount), 1);
});

const displayIntervals = computed(() => {
  if (!stats.value) return [];
  const totalVehicles = stats.value.totalVehicles;
  const totalUnpaid = stats.value.totalUnpaid;
  const intervals = stats.value.intervals;

  let vehicleSum = 0;
  let amountSum = 0;

  return intervals.map((item, index) => {
    const isLast = index === intervals.length - 1;
    let vehicleRatio: number;
    let amountRatio: number;

    if (totalVehicles <= 0) {
      vehicleRatio = 0;
    } else if (isLast) {
      vehicleRatio = parseFloat((100 - vehicleSum).toFixed(1));
    } else {
      vehicleRatio = parseFloat(((item.vehicleCount / totalVehicles) * 100).toFixed(1));
      vehicleSum += vehicleRatio;
    }

    if (totalUnpaid <= 0) {
      amountRatio = 0;
    } else if (isLast) {
      amountRatio = parseFloat((100 - amountSum).toFixed(1));
    } else {
      amountRatio = parseFloat(((item.totalUnpaid / totalUnpaid) * 100).toFixed(1));
      amountSum += amountRatio;
    }

    return { ...item, vehicleRatio, amountRatio };
  });
});

function buildFilter() {
  return {
    entryTimeStart: formatDateTime(timeFilters.value.entryTimeStart) || undefined,
    entryTimeEnd: formatDateTime(timeFilters.value.entryTimeEnd) || undefined,
    exitTimeStart: formatDateTime(timeFilters.value.exitTimeStart) || undefined,
    exitTimeEnd: formatDateTime(timeFilters.value.exitTimeEnd) || undefined,
    bins: bins.value,
    mode: props.mode,
  };
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    stats.value = await getUnpaidIntervalStats(buildFilter());
  } catch (err) {
    error.value = `加载区间分析失败：${err instanceof Error ? err.message : String(err)}`;
  } finally {
    loading.value = false;
  }
}

function resetFilters() {
  timeFilters.value = {
    entryTimeStart: "",
    entryTimeEnd: "",
    exitTimeStart: "",
    exitTimeEnd: "",
  };
  load();
}

async function exportExcel() {
  if (!stats.value || stats.value.intervals.length === 0) {
    error.value = "暂无数据可导出";
    return;
  }

  const now = new Date();
  const fileDate = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
  const modeSuffix = props.mode === "equal-width" ? "等宽分箱" : "等频分箱";
  const defaultName = `欠费${modeSuffix}分析_${fileDate}.xlsx`;
  const filePath = await save({
    filters: [{ name: "Excel 文件", extensions: ["xlsx"] }],
    defaultPath: defaultName,
  });
  if (!filePath) return;

  exporting.value = true;
  error.value = "";
  try {
    await exportUnpaidIntervalStats(filePath, buildFilter());
  } catch (err) {
    error.value = `导出失败：${err instanceof Error ? err.message : String(err)}`;
  } finally {
    exporting.value = false;
  }
}

onMounted(() => {
  load();
});
</script>

<template>
  <div class="h-full flex flex-col gap-4 overflow-hidden">
    <div class="bg-white rounded-lg shadow p-4 shrink-0">
      <div class="flex items-center justify-between flex-wrap gap-3">
        <h2 class="text-lg font-semibold text-slate-800">{{ title }}</h2>
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-2">
            <label class="text-sm text-slate-500">区间数量</label>
            <select
              v-model.number="bins"
              @change="load"
              class="px-3 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option v-for="n in binOptions" :key="n" :value="n">{{ n }} 档</option>
            </select>
          </div>
          <button
            @click="filtersExpanded = !filtersExpanded"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            {{ filtersExpanded ? "收起筛选" : "时间筛选" }}
          </button>
          <button
            @click="load"
            :disabled="loading"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            刷新
          </button>
          <button
            @click="exportExcel"
            :disabled="!stats || stats.intervals.length === 0 || exporting"
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            {{ exporting ? "导出中..." : "导出 Excel" }}
          </button>
        </div>
      </div>

      <div
        v-show="filtersExpanded"
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 mt-4 pt-4 border-t"
      >
        <div>
          <label class="block text-xs text-slate-500 mb-1">进场时间 起</label>
          <input
            v-model="timeFilters.entryTimeStart"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">进场时间 止</label>
          <input
            v-model="timeFilters.entryTimeEnd"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">出场时间 起</label>
          <input
            v-model="timeFilters.exitTimeStart"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">出场时间 止</label>
          <input
            v-model="timeFilters.exitTimeEnd"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div class="flex items-end lg:col-span-4">
          <button
            @click="resetFilters"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            重置时间条件
          </button>
        </div>
      </div>

      <div v-if="error" class="text-sm text-amber-700 bg-amber-50 border border-amber-200 rounded-lg px-3 py-2 mt-3">
        {{ error }}
      </div>
    </div>

    <div v-if="loading" class="flex-1 flex items-center justify-center text-slate-500 text-sm">
      <div class="text-center">
        <div class="inline-block w-8 h-8 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin mb-3"></div>
        <p>正在计算区间分布...</p>
      </div>
    </div>

    <template v-else-if="stats">
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4 shrink-0">
        <div class="bg-white rounded-lg shadow p-4">
          <p class="text-sm text-slate-500">欠费车辆数</p>
          <p class="text-xl font-bold text-slate-800 mt-1">{{ stats.totalVehicles.toLocaleString() }}</p>
        </div>
        <div class="bg-white rounded-lg shadow p-4">
          <p class="text-sm text-slate-500">欠费总额</p>
          <p class="text-xl font-bold text-rose-600 mt-1">{{ formatMoney(stats.totalUnpaid) }}</p>
        </div>
        <div class="bg-white rounded-lg shadow p-4">
          <p class="text-sm text-slate-500">平均每车</p>
          <p class="text-xl font-bold text-blue-600 mt-1">{{ formatMoney(stats.mean) }}</p>
        </div>
        <div class="bg-white rounded-lg shadow p-4">
          <p class="text-sm text-slate-500">中位数</p>
          <p class="text-xl font-bold text-blue-600 mt-1">{{ formatMoney(stats.median) }}</p>
        </div>
        <div class="bg-white rounded-lg shadow p-4">
          <p class="text-sm text-slate-500">标准差</p>
          <p class="text-xl font-bold text-slate-700 mt-1">{{ formatMoney(stats.stdDev) }}</p>
        </div>
        <div class="bg-white rounded-lg shadow p-4">
          <p class="text-sm text-slate-500">最小 / 最大</p>
          <p class="text-xl font-bold text-slate-700 mt-1">{{ stats.min }} / {{ stats.max }}</p>
        </div>
      </div>

      <div class="bg-white rounded-lg shadow flex flex-col flex-1 min-h-0 overflow-hidden">
        <div class="p-4 border-b">
          <h3 class="font-semibold text-slate-800">{{ methodName }}分布</h3>
          <p class="text-xs text-slate-500 mt-1">{{ methodDescription }}</p>
        </div>

        <div class="flex-1 overflow-auto p-4">
          <table v-if="stats.intervals.length > 0" class="w-full text-sm">
            <thead class="bg-slate-100 text-slate-600 sticky top-0">
              <tr>
                <th class="text-left px-4 py-2">欠费区间</th>
                <th class="text-right px-4 py-2">车辆数</th>
                <th class="text-right px-4 py-2">车辆占比</th>
                <th class="text-right px-4 py-2">欠费总额</th>
                <th class="text-right px-4 py-2">金额占比</th>
                <th class="text-left px-4 py-2 w-1/3">车辆分布</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr v-for="item in displayIntervals" :key="item.range" class="hover:bg-slate-50">
                <td class="px-4 py-3 font-medium text-slate-800">{{ item.range }}</td>
                <td class="text-right px-4 py-3">{{ item.vehicleCount.toLocaleString() }}</td>
                <td class="text-right px-4 py-3 text-slate-500">{{ item.vehicleRatio }}%</td>
                <td class="text-right px-4 py-3">{{ formatMoney(item.totalUnpaid) }}</td>
                <td class="text-right px-4 py-3 text-slate-500">{{ item.amountRatio }}%</td>
                <td class="px-4 py-3">
                  <div class="flex items-center gap-2">
                    <div class="flex-1 h-2.5 bg-slate-100 rounded-full overflow-hidden">
                      <div
                        class="h-full bg-blue-500 rounded-full"
                        :style="{ width: `${(item.vehicleCount / maxVehicleCount) * 100}%` }"
                      ></div>
                    </div>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>

          <div v-else class="h-full flex items-center justify-center text-slate-400 text-sm">
            暂无欠费车辆数据
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
