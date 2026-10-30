<script setup lang="ts">
import { ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { getVehicleMonthlyStats, listVehicleRecords, exportVehicleCharges } from "../api";
import type { ParkingRecord, VehicleMonthlyItem } from "../types";

const plate = ref("");
const pendingPlate = ref("");
const loading = ref(false);
const exporting = ref(false);
const error = ref("");
const monthlyStats = ref<VehicleMonthlyItem[]>([]);
const filtersExpanded = ref(false);
const timeFilters = ref({
  entryTimeStart: "",
  entryTimeEnd: "",
  exitTimeStart: "",
  exitTimeEnd: "",
});

const toast = ref<{ show: boolean; message: string; type: "success" | "error" }>({
  show: false,
  message: "",
  type: "success",
});
let toastTimer: ReturnType<typeof setTimeout> | null = null;

function showToast(message: string, type: "success" | "error" = "success") {
  toast.value = { show: true, message, type };
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.value.show = false;
  }, 3000);
}

const detailMonth = ref("");
const detailRecords = ref<ParkingRecord[]>([]);
const detailTotal = ref(0);
const detailPage = ref(1);
const detailPageSize = ref(20);
const detailLoading = ref(false);

const formatMoney = (n: number) => `¥${n.toFixed(2)}`;

function buildTimeFilters() {
  return {
    entryTimeStart: timeFilters.value.entryTimeStart || undefined,
    entryTimeEnd: timeFilters.value.entryTimeEnd || undefined,
    exitTimeStart: timeFilters.value.exitTimeStart || undefined,
    exitTimeEnd: timeFilters.value.exitTimeEnd || undefined,
  };
}

async function search() {
  const p = pendingPlate.value.trim();
  if (!p) {
    error.value = "请输入车牌号";
    return;
  }
  error.value = "";
  plate.value = p;
  loading.value = true;
  detailMonth.value = "";
  detailRecords.value = [];
  detailTotal.value = 0;
  try {
    monthlyStats.value = await getVehicleMonthlyStats(p, buildTimeFilters());
  } catch (err) {
    error.value = `查询失败：${err instanceof Error ? err.message : String(err)}`;
  } finally {
    loading.value = false;
  }
}

async function showDetail(month: string) {
  detailMonth.value = month;
  detailPage.value = 1;
  await loadDetail();
}

async function loadDetail() {
  if (!plate.value || !detailMonth.value) return;
  detailLoading.value = true;
  try {
    const result = await listVehicleRecords(
      plate.value,
      detailMonth.value,
      detailPage.value,
      detailPageSize.value,
      buildTimeFilters()
    );
    detailRecords.value = result.items;
    detailTotal.value = result.total;
  } catch (err) {
    error.value = `加载明细失败：${err instanceof Error ? err.message : String(err)}`;
  } finally {
    detailLoading.value = false;
  }
}

function resetFilters() {
  timeFilters.value = {
    entryTimeStart: "",
    entryTimeEnd: "",
    exitTimeStart: "",
    exitTimeEnd: "",
  };
}

async function exportExcel() {
  if (!plate.value || monthlyStats.value.length === 0) {
    error.value = "请先搜索并获取车辆收费数据";
    return;
  }

  const fileDate = new Date().toISOString().slice(0, 10);
  const defaultName = `车辆收费统计_${plate.value}_${fileDate}.xlsx`;
  const filePath = await save({
    filters: [{ name: "Excel 文件", extensions: ["xlsx"] }],
    defaultPath: defaultName,
  });
  if (!filePath) return;

  exporting.value = true;
  error.value = "";
  try {
    await exportVehicleCharges(filePath, {
      plate: plate.value,
      ...buildTimeFilters(),
    });
    showToast("导出成功", "success");
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    error.value = `导出失败：${msg}`;
    showToast(`导出失败：${msg}`, "error");
  } finally {
    exporting.value = false;
  }
}

function setDetailPage(value: number) {
  const max = Math.max(1, Math.ceil(detailTotal.value / detailPageSize.value));
  detailPage.value = Math.max(1, Math.min(max, value));
  loadDetail();
}

const startIndex = () => (detailPage.value - 1) * detailPageSize.value + 1;
const detailTotalPages = () => Math.max(1, Math.ceil(detailTotal.value / detailPageSize.value));
</script>

<template>
  <div class="h-full flex flex-col gap-4 overflow-hidden">
    <!-- 上区域 -->
    <div class="bg-white rounded-lg shadow flex flex-col max-h-[50%] overflow-hidden">
      <div class="p-4 shrink-0 border-b">
        <h2 class="text-lg font-semibold text-slate-800 mb-3">车辆收费查询</h2>
        <div class="flex items-end gap-3">
          <div>
            <label class="block text-xs text-slate-500 mb-1">车牌号</label>
            <input
              v-model="pendingPlate"
              type="text"
              placeholder="请输入车牌号"
              class="px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 w-64"
              @keydown.enter="search"
            />
          </div>
          <button
            @click="search"
            class="px-5 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition"
          >
            搜索
          </button>
          <button
            @click="exportExcel"
            :disabled="!plate || monthlyStats.length === 0 || exporting"
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            {{ exporting ? "导出中..." : "导出 Excel" }}
          </button>
          <button
            @click="filtersExpanded = !filtersExpanded"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            {{ filtersExpanded ? "收起筛选" : "更多筛选" }}
          </button>
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
        正在查询...
      </div>

      <div v-else-if="plate && monthlyStats.length === 0" class="flex-1 flex items-center justify-center text-slate-400 text-sm">
        未找到该车牌的收费记录
      </div>

      <template v-else-if="monthlyStats.length > 0">
        <div class="px-4 pt-2 shrink-0">
          <p class="text-sm text-slate-500">车牌号：{{ plate }}，共 {{ monthlyStats.length }} 个月</p>
        </div>

        <div class="px-4 pt-2 shrink-0">
          <table class="w-full text-sm table-fixed">
            <thead class="bg-slate-100 text-slate-600">
              <tr>
                <th class="text-left px-4 py-2 w-28">月份</th>
                <th class="text-right px-4 py-2 w-24">记录数</th>
                <th class="text-right px-4 py-2 w-28">应收金额</th>
                <th class="text-right px-4 py-2 w-28">实收金额</th>
                <th class="text-right px-4 py-2 w-28">未收金额</th>
                <th class="px-4 py-2 w-20"></th>
              </tr>
            </thead>
          </table>
        </div>

        <div class="flex-1 overflow-auto px-4 pb-4">
          <table class="w-full text-sm table-fixed">
            <tbody class="divide-y">
              <tr v-for="item in monthlyStats" :key="item.month" class="hover:bg-slate-50">
                <td class="px-4 py-2 w-28 font-medium">{{ item.month }}</td>
                <td class="text-right px-4 py-2 w-24">{{ item.count.toLocaleString() }}</td>
                <td class="text-right px-4 py-2 w-28">{{ formatMoney(item.due) }}</td>
                <td class="text-right px-4 py-2 w-28">{{ formatMoney(item.paid) }}</td>
                <td class="text-right px-4 py-2 w-28">{{ formatMoney(item.unpaid) }}</td>
                <td class="px-4 py-2 w-20">
                  <button
                    @click="showDetail(item.month)"
                    class="px-3 py-1 bg-blue-50 text-blue-600 hover:bg-blue-100 rounded text-xs font-medium transition"
                  >
                    详情
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </div>

    <!-- 下区域 -->
    <div class="bg-white rounded-lg shadow flex flex-col flex-1 min-h-0 overflow-hidden">
      <div class="p-4 border-b">
        <h2 class="text-lg font-semibold text-slate-800">
          收费明细
          <span v-if="detailMonth" class="text-sm font-normal text-slate-500 ml-2">
            {{ plate }} · {{ detailMonth }}
          </span>
        </h2>
      </div>

      <div v-if="!detailMonth" class="flex-1 flex items-center justify-center text-slate-400 text-sm">
        请点击上方“详情”按钮查看对应月份的收费记录
      </div>

      <div v-else-if="detailLoading" class="flex-1 flex items-center justify-center text-slate-500 text-sm">
        正在加载明细...
      </div>

      <template v-else>
        <div class="overflow-auto flex-1">
          <table class="min-w-full text-sm text-left">
            <thead class="bg-slate-100 text-slate-600 sticky top-0">
              <tr>
                <th class="px-4 py-3 font-medium">序号</th>
                <th class="px-4 py-3 font-medium">车牌号</th>
                <th class="px-4 py-3 font-medium">进场时间</th>
                <th class="px-4 py-3 font-medium">出场时间</th>
                <th class="px-4 py-3 font-medium">停车时长</th>
                <th class="px-4 py-3 font-medium text-right">应收金额</th>
                <th class="px-4 py-3 font-medium text-right">实收金额</th>
                <th class="px-4 py-3 font-medium">放行方式</th>
                <th class="px-4 py-3 font-medium">来源文件</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr v-for="(record, idx) in detailRecords" :key="record.id" class="hover:bg-slate-50">
                <td class="px-4 py-3 text-slate-500">{{ startIndex() + idx }}</td>
                <td class="px-4 py-3 font-medium">{{ record.plate }}</td>
                <td class="px-4 py-3 whitespace-nowrap">{{ record.entryTime }}</td>
                <td class="px-4 py-3 whitespace-nowrap">{{ record.exitTime }}</td>
                <td class="px-4 py-3">{{ record.duration }}</td>
                <td class="px-4 py-3 text-right">{{ record.dueAmount.toFixed(2) }}</td>
                <td class="px-4 py-3 text-right">{{ record.paidAmount.toFixed(2) }}</td>
                <td class="px-4 py-3">{{ record.releaseMethod }}</td>
                <td class="px-4 py-3 text-slate-500">{{ record.sourceFile }}</td>
              </tr>
              <tr v-if="detailRecords.length === 0">
                <td colspan="9" class="px-4 py-12 text-center text-slate-400">暂无记录</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="p-4 border-t flex items-center justify-between shrink-0">
          <span class="text-sm text-slate-500">
            第 {{ detailPage }} / {{ detailTotalPages() }} 页，共 {{ detailTotal.toLocaleString() }} 条
          </span>
          <div class="flex items-center gap-2">
            <button
              @click="setDetailPage(1)"
              :disabled="detailPage === 1"
              class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
            >
              首页
            </button>
            <button
              @click="setDetailPage(detailPage - 1)"
              :disabled="detailPage === 1"
              class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
            >
              上一页
            </button>
            <button
              @click="setDetailPage(detailPage + 1)"
              :disabled="detailPage === detailTotalPages()"
              class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
            >
              下一页
            </button>
            <button
              @click="setDetailPage(detailTotalPages())"
              :disabled="detailPage === detailTotalPages()"
              class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
            >
              末页
            </button>
          </div>
        </div>
      </template>
    </div>

    <!-- 右下角提示 -->
    <transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="translate-y-2 opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-2 opacity-0"
    >
      <div
        v-if="toast.show"
        class="fixed bottom-4 right-4 z-50 px-4 py-3 rounded-lg shadow-lg text-sm font-medium max-w-xs"
        :class="toast.type === 'success' ? 'bg-emerald-600 text-white' : 'bg-red-600 text-white'"
      >
        {{ toast.message }}
      </div>
    </transition>
  </div>
</template>
