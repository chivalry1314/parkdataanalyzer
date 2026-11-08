<script setup lang="ts">
import { computed, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import type {
  BuildingChargeItem,
  BuildingChargePlateDetail,
  BuildingChargesFilters,
  Paginated,
  ParkingRecord,
  VehicleMonthlyItem,
} from "../types";
import {
  exportBuildingCharges,
  exportVehicleCharges,
  getBuildingChargeDetail,
  listVehicleRecords,
} from "../api";

defineProps<{
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: "search", value: BuildingChargesFilters): void;
}>();

const filters = ref<BuildingChargesFilters>({
  building: "",
  unit: "",
  room: "",
  entryTimeStart: "",
  entryTimeEnd: "",
  exitTimeStart: "",
  exitTimeEnd: "",
  page: 1,
  perPage: 20,
});

const items = ref<BuildingChargeItem[]>([]);
const total = ref(0);
const page = ref(1);
const perPage = ref(20);
const detailItem = ref<BuildingChargeItem | null>(null);
const plateDetails = ref<BuildingChargePlateDetail[]>([]);
const detailLoading = ref(false);
const detailError = ref("");
const selectedPlate = ref("");

const recordsPlate = ref("");
const recordsMonth = ref("");
const records = ref<Paginated<ParkingRecord> | null>(null);
const recordsPage = ref(1);
const recordsPerPage = ref(20);
const recordsLoading = ref(false);
const recordsError = ref("");

const exporting = ref(false);
const exportError = ref("");

const toast = ref<{ show: boolean; message: string; type: "success" | "error" }>({
  show: false,
  message: "",
  type: "success",
});
let toastTimer: ReturnType<typeof setTimeout> | null = null;

const view = ref<"list" | "plates" | "detail" | "records">("list");

const selectedPlateDetail = computed<BuildingChargePlateDetail | null>(() => {
  return plateDetails.value.find((d) => d.plate === selectedPlate.value) ?? null;
});

function doSearch() {
  if (!filters.value.building.trim()) {
    showToast("请输入楼栋号", "error");
    return;
  } else if (!filters.value.unit.trim()) {
    showToast("请输入单元号", "error");
    return;
  }
  emit("search", { ...filters.value });
}

function showToast(message: string, type: "success" | "error" = "success") {
  toast.value = { show: true, message, type };
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.value.show = false;
  }, 3000);
}

function resetSearch() {
  filters.value = {
    building: "",
    unit: "",
    room: "",
    entryTimeStart: "",
    entryTimeEnd: "",
    exitTimeStart: "",
    exitTimeEnd: "",
    page: 1,
    perPage: perPage.value,
  };
  emit("search", { ...filters.value });
}

function onPageSizeChange(e: Event) {
  perPage.value = Number((e.target as HTMLSelectElement).value);
  filters.value.page = 1;
  filters.value.perPage = perPage.value;
  emit("search", { ...filters.value });
}

function setPage(value: number) {
  const max = Math.max(1, Math.ceil(total.value / perPage.value));
  const target = Math.max(1, Math.min(max, value));
  filters.value.page = target;
  emit("search", { ...filters.value });
}

function setItems(data: Paginated<BuildingChargeItem>) {
  items.value = data.items;
  total.value = data.total;
  page.value = data.page;
  perPage.value = data.perPage;
  view.value = "list";
  detailItem.value = null;
  plateDetails.value = [];
  detailError.value = "";
  selectedPlate.value = "";
  records.value = null;
  recordsError.value = "";
}

async function showDetail(item: BuildingChargeItem) {
  detailItem.value = item;
  view.value = "plates";
  detailLoading.value = true;
  detailError.value = "";
  plateDetails.value = [];
  selectedPlate.value = "";
  try {
    const result = await getBuildingChargeDetail({
      building: item.building,
      unit: item.unit,
      room: item.room,
      entryTimeStart: filters.value.entryTimeStart,
      entryTimeEnd: filters.value.entryTimeEnd,
      exitTimeStart: filters.value.exitTimeStart,
      exitTimeEnd: filters.value.exitTimeEnd,
    });
    plateDetails.value = result;
  } catch (err) {
    detailError.value = err instanceof Error ? err.message : String(err);
    plateDetails.value = [];
  } finally {
    detailLoading.value = false;
  }
}

async function exportExcel() {
  if (items.value.length === 0) {
    exportError.value = "当前没有可导出的楼栋收费数据";
    return;
  }

  const fileDate = new Date().toISOString().slice(0, 10);
  const defaultName = `楼栋收费统计_${fileDate}.xlsx`;
  const filePath = await save({
    filters: [{ name: "Excel 文件", extensions: ["xlsx"] }],
    defaultPath: defaultName,
  });
  if (!filePath) return;

  exporting.value = true;
  exportError.value = "";
  try {
    await exportBuildingCharges(filePath, { ...filters.value });
    showToast("导出成功", "success");
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    exportError.value = `导出失败：${msg}`;
    showToast(`导出失败：${msg}`, "error");
  } finally {
    exporting.value = false;
  }
}

async function exportVehicleExcel() {
  if (!selectedPlate.value) {
    exportError.value = "当前没有可导出的车辆收费数据";
    return;
  }

  const fileDate = new Date().toISOString().slice(0, 10);
  const defaultName = `车辆收费统计_${selectedPlate.value}_${fileDate}.xlsx`;
  const filePath = await save({
    filters: [{ name: "Excel 文件", extensions: ["xlsx"] }],
    defaultPath: defaultName,
  });
  if (!filePath) return;

  exporting.value = true;
  exportError.value = "";
  try {
    await exportVehicleCharges(filePath, {
      plate: selectedPlate.value,
      entryTimeStart: filters.value.entryTimeStart,
      entryTimeEnd: filters.value.entryTimeEnd,
      exitTimeStart: filters.value.exitTimeStart,
      exitTimeEnd: filters.value.exitTimeEnd,
    });
    showToast("导出成功", "success");
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    exportError.value = `导出失败：${msg}`;
    showToast(`导出失败：${msg}`, "error");
  } finally {
    exporting.value = false;
  }
}

function showPlateMonthly(plate: string) {
  selectedPlate.value = plate;
  view.value = "detail";
}

async function showRecords(plate: string, month: string) {
  recordsPlate.value = plate;
  recordsMonth.value = month;
  recordsPage.value = 1;
  view.value = "records";
  await loadRecords();
}

async function loadRecords() {
  recordsLoading.value = true;
  recordsError.value = "";
  try {
    const result = await listVehicleRecords(recordsPlate.value, recordsMonth.value, recordsPage.value, recordsPerPage.value, {
      entryTimeStart: filters.value.entryTimeStart,
      entryTimeEnd: filters.value.entryTimeEnd,
      exitTimeStart: filters.value.exitTimeStart,
      exitTimeEnd: filters.value.exitTimeEnd,
    });
    records.value = result;
  } catch (err) {
    recordsError.value = err instanceof Error ? err.message : String(err);
    records.value = null;
  } finally {
    recordsLoading.value = false;
  }
}

function backToList() {
  view.value = "list";
  detailItem.value = null;
  plateDetails.value = [];
  selectedPlate.value = "";
  records.value = null;
}

function backToPlates() {
  view.value = "plates";
  selectedPlate.value = "";
  records.value = null;
}

function backToDetail() {
  view.value = "detail";
  records.value = null;
}

function onRecordsPageChange(page: number) {
  recordsPage.value = page;
  loadRecords();
}

function onRecordsPageSizeChange(e: Event) {
  recordsPerPage.value = Number((e.target as HTMLSelectElement).value);
  recordsPage.value = 1;
  loadRecords();
}

function plateSummary(detail: BuildingChargePlateDetail) {
  return detail.items.reduce(
    (acc, item) => {
      acc.count += item.count;
      acc.due += item.due;
      acc.paid += item.paid;
      acc.unpaid += item.unpaid;
      return acc;
    },
    { count: 0, due: 0, paid: 0, unpaid: 0 }
  );
}

function detailTotal(items: VehicleMonthlyItem[]) {
  return items.reduce(
    (acc, item) => {
      acc.count += item.count;
      acc.due += item.due;
      acc.paid += item.paid;
      acc.unpaid += item.unpaid;
      return acc;
    },
    { count: 0, due: 0, paid: 0, unpaid: 0 }
  );
}

const recordsSummary = computed(() => {
  const items = records.value?.items ?? [];
  return items.reduce(
    (acc, record) => {
      acc.count += 1;
      acc.due += record.dueAmount;
      acc.paid += record.paidAmount;
      acc.unpaid += record.dueAmount - record.paidAmount;
      return acc;
    },
    { count: 0, due: 0, paid: 0, unpaid: 0 }
  );
});

defineExpose({ setItems });
</script>

<template>
  <div class="bg-white rounded-lg shadow flex flex-col h-full">
    <div class="p-4 border-b flex flex-col gap-3">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h2 class="text-lg font-semibold text-slate-800">楼栋收费</h2>
          <p class="text-sm text-slate-500">按楼栋单元房间汇总停车收费</p>
        </div>
        <div v-if="view !== 'list'" class="flex items-center gap-3">
          <button
            v-if="view === 'records'"
            @click="backToDetail"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            返回月份
          </button>
          <button
            v-if="view === 'detail'"
            @click="backToPlates"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            返回车辆
          </button>
          <button
            @click="backToList"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            返回列表
          </button>
        </div>
      </div>

      <div
        v-if="view === 'list'"
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3"
      >
        <div>
          <label class="block text-xs text-slate-500 mb-1">楼栋号（必填）</label>
          <input
            v-model="filters.building"
            type="text"
            placeholder="如：10"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">单元号（必填）</label>
          <input
            v-model="filters.unit"
            type="text"
            placeholder="如：2"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">房间号（可为空）</label>
          <input
            v-model="filters.room"
            type="text"
            placeholder="如：1103"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">进场时间 起</label>
          <input
            v-model="filters.entryTimeStart"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">进场时间 止</label>
          <input
            v-model="filters.entryTimeEnd"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">出场时间 起</label>
          <input
            v-model="filters.exitTimeStart"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">出场时间 止</label>
          <input
            v-model="filters.exitTimeEnd"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div class="flex items-end gap-3">
          <button
            @click="doSearch"
            :disabled="loading"
            class="px-5 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            搜索
          </button>
          <button
            @click="resetSearch"
            class="px-5 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            重置
          </button>
          <button
            @click="exportExcel"
            :disabled="exporting || items.length === 0"
            class="px-5 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            {{ exporting ? "导出中..." : "导出 Excel" }}
          </button>
        </div>
      </div>

      <div v-if="exportError" class="text-sm text-amber-700 bg-amber-50 border border-amber-200 rounded-lg px-3 py-2">
        {{ exportError }}
      </div>
    </div>

    <div class="overflow-auto flex-1 p-4">
      <!-- List View -->
      <div v-if="view === 'list'">
        <div v-if="loading" class="text-center py-12 text-slate-500">
          <div class="inline-block w-8 h-8 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin mb-3"></div>
          <p>正在加载楼栋收费数据...</p>
        </div>

        <template v-else>
          <table class="min-w-full text-sm text-left">
            <thead class="bg-slate-100 text-slate-600 sticky top-0">
              <tr>
                <th class="px-4 py-3 font-medium">楼栋</th>
                <th class="px-4 py-3 font-medium">单元</th>
                <th class="px-4 py-3 font-medium">房间</th>
                <th class="px-4 py-3 font-medium text-right">车牌数</th>
                <th class="px-4 py-3 font-medium text-right">记录数</th>
                <th class="px-4 py-3 font-medium text-right">应收金额</th>
                <th class="px-4 py-3 font-medium text-right">实收金额</th>
                <th class="px-4 py-3 font-medium text-right">欠费金额</th>
                <th class="px-4 py-3 font-medium">操作</th>
              </tr>
            </thead>
            <tbody class="divide-y">
            <tr
              v-for="item in items"
              :key="`${item.building}-${item.unit}-${item.room}`"
              class="hover:bg-slate-50 cursor-pointer"
              @click="showDetail(item)"
            >
              <td class="px-4 py-3">{{ item.building || "--" }}</td>
              <td class="px-4 py-3">{{ item.unit || "--" }}</td>
              <td class="px-4 py-3">{{ item.room || "--" }}</td>
              <td class="px-4 py-3 text-right">{{ item.plateCount }}</td>
              <td class="px-4 py-3 text-right">{{ item.totalCount }}</td>
              <td class="px-4 py-3 text-right">{{ (item.totalDue).toFixed(2) }}</td>
              <td class="px-4 py-3 text-right">{{ (item.totalPaid).toFixed(2) }}</td>
              <td class="px-4 py-3 text-right">{{ (item.totalUnpaid).toFixed(2) }}</td>
              <td class="px-4 py-3">
                <button
                  @click.stop="showDetail(item)"
                  class="text-blue-600 hover:text-blue-800 text-sm font-medium"
                >
                  详情
                </button>
              </td>
            </tr>
            <tr v-if="items.length === 0">
              <td colspan="9" class="px-4 py-12 text-center text-slate-400">
                暂无数据，请输入楼栋号、单元号进行搜索
              </td>
            </tr>
          </tbody>
        </table>

        <div class="flex items-center justify-between pt-4">
          <span class="text-sm text-slate-500">
            第 {{ page }} 页，共 {{ total }} 条
          </span>
          <div class="flex items-center gap-2">
            <select
              :value="perPage"
              @change="onPageSizeChange"
              class="px-3 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option :value="20">20 条/页</option>
              <option :value="50">50 条/页</option>
              <option :value="100">100 条/页</option>
            </select>
            <button
              @click="setPage(page - 1)"
              :disabled="page === 1"
              class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
            >
              上一页
            </button>
            <button
              @click="setPage(page + 1)"
              :disabled="page * perPage >= total"
              class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
            >
              下一页
            </button>
          </div>
        </div>
      </template>
      </div>

      <!-- Plate List View -->
      <div v-else-if="view === 'plates' && detailItem">
        <div class="mb-4">
          <h3 class="text-lg font-semibold text-slate-800">
            {{ detailItem.building }}-{{ detailItem.unit }}{{ detailItem.room ? `-${detailItem.room}` : "" }}
          </h3>
          <p class="text-sm text-slate-500">
            车牌数 {{ detailItem.plateCount }}，记录数 {{ detailItem.totalCount }}，
            应收 {{ (detailItem.totalDue).toFixed(2) }}，
            实收 {{ (detailItem.totalPaid).toFixed(2) }}，
            欠费 {{ (detailItem.totalUnpaid).toFixed(2) }}
          </p>
        </div>

        <div v-if="detailLoading" class="text-sm text-slate-500 py-8 text-center">正在加载...</div>

        <div v-else-if="detailError" class="mb-4 rounded-lg px-4 py-3 bg-amber-50 border border-amber-200 text-amber-800 text-sm">
          加载失败：{{ detailError }}
        </div>

        <table v-if="!detailLoading && !detailError" class="min-w-full text-sm text-left">
          <thead class="bg-slate-100 text-slate-600 sticky top-0">
            <tr>
              <th class="px-4 py-3 font-medium">车牌号</th>
              <th class="px-4 py-3 font-medium text-right">记录数</th>
              <th class="px-4 py-3 font-medium text-right">应收金额</th>
              <th class="px-4 py-3 font-medium text-right">实收金额</th>
              <th class="px-4 py-3 font-medium text-right">欠费金额</th>
              <th class="px-4 py-3 font-medium">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y">
            <tr
              v-for="detail in plateDetails"
              :key="detail.plate"
              class="hover:bg-slate-50 cursor-pointer"
              @click="showPlateMonthly(detail.plate)"
            >
              <td class="px-4 py-3 font-medium">{{ detail.plate }}</td>
              <td class="px-4 py-3 text-right">{{ plateSummary(detail).count }}</td>
              <td class="px-4 py-3 text-right">{{ (plateSummary(detail).due).toFixed(2) }}</td>
              <td class="px-4 py-3 text-right">{{ (plateSummary(detail).paid).toFixed(2) }}</td>
              <td class="px-4 py-3 text-right">{{ (plateSummary(detail).unpaid).toFixed(2) }}</td>
              <td class="px-4 py-3">
                <button
                  @click.stop="showPlateMonthly(detail.plate)"
                  class="text-blue-600 hover:text-blue-800 text-sm font-medium"
                >
                  详情
                </button>
              </td>
            </tr>
            <tr v-if="plateDetails.length === 0">
              <td colspan="6" class="px-4 py-12 text-center text-slate-400">
                该楼栋单元房间下没有关联的车辆停车记录
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Monthly Detail View -->
      <div v-else-if="view === 'detail' && detailItem && selectedPlateDetail">
        <div class="mb-4 flex items-start justify-between gap-3">
          <div>
            <h3 class="text-lg font-semibold text-slate-800">
              {{ detailItem.building }}-{{ detailItem.unit }}{{ detailItem.room ? `-${detailItem.room}` : "" }}
              / {{ selectedPlateDetail.plate }}
            </h3>
            <p class="text-sm text-slate-500">
              记录数 {{ detailTotal(selectedPlateDetail.items).count }}，
              应收 {{ (detailTotal(selectedPlateDetail.items).due).toFixed(2) }}，
              实收 {{ (detailTotal(selectedPlateDetail.items).paid).toFixed(2) }}，
              欠费 {{ (detailTotal(selectedPlateDetail.items).unpaid).toFixed(2) }}
            </p>
          </div>
          <button
            @click="exportVehicleExcel"
            :disabled="exporting"
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition shrink-0"
          >
            {{ exporting ? "导出中..." : "导出 Excel" }}
          </button>
        </div>

        <table class="min-w-full text-sm text-left">
          <thead class="bg-slate-100 text-slate-600 sticky top-0">
            <tr>
              <th class="px-4 py-3 font-medium">月份</th>
              <th class="px-4 py-3 font-medium text-right">记录数</th>
              <th class="px-4 py-3 font-medium text-right">应收金额</th>
              <th class="px-4 py-3 font-medium text-right">实收金额</th>
              <th class="px-4 py-3 font-medium text-right">欠费金额</th>
              <th class="px-4 py-3 font-medium">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y">
            <tr
              v-for="item in selectedPlateDetail.items"
              :key="item.month"
              class="hover:bg-slate-50 cursor-pointer"
              @click="showRecords(selectedPlateDetail.plate, item.month)"
            >
              <td class="px-4 py-3">{{ item.month }}</td>
              <td class="px-4 py-3 text-right">{{ item.count }}</td>
              <td class="px-4 py-3 text-right">{{ (item.due).toFixed(2) }}</td>
              <td class="px-4 py-3 text-right">{{ (item.paid).toFixed(2) }}</td>
              <td class="px-4 py-3 text-right">{{ (item.unpaid).toFixed(2) }}</td>
              <td class="px-4 py-3">
                <button
                  @click.stop="showRecords(selectedPlateDetail.plate, item.month)"
                  class="text-blue-600 hover:text-blue-800 text-sm font-medium"
                >
                  详情
                </button>
              </td>
            </tr>
            <tr v-if="selectedPlateDetail.items.length === 0">
              <td colspan="6" class="px-4 py-12 text-center text-slate-400">
                该车牌无停车记录
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Records View -->
      <div v-else-if="view === 'records'">
        <div class="mb-4">
          <h3 class="text-lg font-semibold text-slate-800">
            {{ recordsPlate }} - {{ recordsMonth }}
          </h3>
          <p class="text-sm text-slate-500">
            共 {{ records?.total ?? 0 }} 条记录，
            应收 {{ recordsSummary.due.toFixed(2) }}，
            实收 {{ recordsSummary.paid.toFixed(2) }}，
            欠费 {{ recordsSummary.unpaid.toFixed(2) }}
          </p>
        </div>

        <div v-if="recordsLoading" class="text-sm text-slate-500 py-8 text-center">正在加载...</div>

        <div v-else-if="recordsError" class="mb-4 rounded-lg px-4 py-3 bg-amber-50 border border-amber-200 text-amber-800 text-sm">
          加载失败：{{ recordsError }}
        </div>

        <div v-else-if="records" class="space-y-3">
          <table class="min-w-full text-sm text-left">
            <thead class="bg-slate-100 text-slate-600 sticky top-0">
              <tr>
                <th class="px-4 py-3 font-medium">车牌号</th>
                <th class="px-4 py-3 font-medium">进场时间</th>
                <th class="px-4 py-3 font-medium">出场时间</th>
                <th class="px-4 py-3 font-medium">停车时长</th>
                <th class="px-4 py-3 font-medium text-right">应收金额</th>
                <th class="px-4 py-3 font-medium text-right">实收金额</th>
                <th class="px-4 py-3 font-medium">放行方式</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr
                v-for="record in records.items"
                :key="record.id"
                class="hover:bg-slate-50"
              >
                <td class="px-4 py-3 font-medium">{{ record.plate }}</td>
                <td class="px-4 py-3 whitespace-nowrap">{{ record.entryTime }}</td>
                <td class="px-4 py-3 whitespace-nowrap">{{ record.exitTime }}</td>
                <td class="px-4 py-3">{{ record.duration }}</td>
                <td class="px-4 py-3 text-right">{{ (record.dueAmount).toFixed(2) }}</td>
                <td class="px-4 py-3 text-right">{{ (record.paidAmount).toFixed(2) }}</td>
                <td class="px-4 py-3">{{ record.releaseMethod }}</td>
              </tr>
              <tr v-if="records.items.length === 0">
                <td colspan="7" class="px-4 py-12 text-center text-slate-400">
                  暂无记录
                </td>
              </tr>
            </tbody>
          </table>

          <div class="flex items-center justify-between pt-2">
            <span class="text-sm text-slate-500">
              第 {{ recordsPage }} 页，共 {{ records.total }} 条
            </span>
            <div class="flex items-center gap-2">
              <select
                :value="recordsPerPage"
                @change="onRecordsPageSizeChange"
                class="px-3 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option :value="20">20 条/页</option>
                <option :value="50">50 条/页</option>
                <option :value="100">100 条/页</option>
              </select>
              <button
                @click="onRecordsPageChange(recordsPage - 1)"
                :disabled="recordsPage === 1"
                class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
              >
                上一页
              </button>
              <button
                @click="onRecordsPageChange(recordsPage + 1)"
                :disabled="recordsPage * recordsPerPage >= records.total"
                class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
              >
                下一页
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

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
