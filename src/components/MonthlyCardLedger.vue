<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import type { MonthlyCardLedger, MonthlyCardLedgerFilters } from "../types";

const props = defineProps<{
  records: MonthlyCardLedger[];
  total: number;
  page: number;
  pageSize: number;
  filters: MonthlyCardLedgerFilters;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:page", value: number): void;
  (e: "update:pageSize", value: number): void;
  (e: "search", value: MonthlyCardLedgerFilters): void;
  (e: "reset"): void;
  (e: "import-file", filePath: string, data: Uint8Array): void;
  (e: "clear"): void;
}>();

const expanded = ref(false);
const pendingFilters = ref<MonthlyCardLedgerFilters>({ ...props.filters });

watch(
  () => props.filters,
  (val) => {
    pendingFilters.value = { ...val };
  },
  { deep: true }
);

function doSearch() {
  emit("search", { ...pendingFilters.value });
}

function resetSearch() {
  pendingFilters.value = {
    plate: "",
    ownerName: "",
    roomNumber: "",
    processDateStart: "",
    processDateEnd: "",
  };
  emit("reset");
}

function onPageSizeChange(e: Event) {
  emit("update:pageSize", Number((e.target as HTMLSelectElement).value));
}

function setPage(value: number) {
  const max = totalPages();
  emit("update:page", Math.max(1, Math.min(max, value)));
}

const totalPages = () => Math.max(1, Math.ceil(props.total / props.pageSize));
const startIndex = () => (props.page - 1) * props.pageSize + 1;

const totalStats = computed(() => {
  return props.records.reduce(
    (acc, item) => {
      acc.rent += item.rent;
      acc.managementFee += item.managementFee;
      acc.total += item.rent + item.managementFee;
      return acc;
    },
    { rent: 0, managementFee: 0, total: 0 }
  );
});

async function importFile() {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Excel 文件", extensions: ["xlsx"] }],
    defaultPath: "E:/shenyewuye/project/深业泰蓉府停车信息登记表--每日更新.xlsx",
  });
  if (!selected) return;
  const path = Array.isArray(selected) ? selected[0] : selected;
  const data = await readFile(path);
  emit("import-file", path, data);
}

async function clearAll() {
  const ok = await confirm("确定要清空所有月卡台账记录吗？此操作不可恢复。", {
    title: "请确认",
    kind: "warning",
  });
  if (!ok) return;
  emit("clear");
}

function formatMoney(n: number) {
  return `¥${n.toFixed(2)}`;
}

function formatBool(v: boolean) {
  return v ? "是" : "否";
}
</script>

<template>
  <div class="bg-white rounded-lg shadow flex flex-col h-full">
    <div class="p-4 border-b flex flex-col gap-3">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h2 class="text-lg font-semibold text-slate-800">月卡台账</h2>
          <p class="text-sm text-slate-500">共 {{ total.toLocaleString() }} 条记录</p>
        </div>
        <div class="flex items-center gap-3">
          <button
            @click="importFile"
            :disabled="loading"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            导入 Excel
          </button>
          <button
            @click="clearAll"
            :disabled="loading || total === 0"
            class="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            清空
          </button>
          <button
            @click="expanded = !expanded"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            {{ expanded ? "收起筛选" : "展开筛选" }}
          </button>
          <select
            :value="pageSize"
            @change="onPageSizeChange"
            class="px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option :value="20">20 条/页</option>
            <option :value="50">50 条/页</option>
            <option :value="100">100 条/页</option>
          </select>
        </div>
      </div>

      <div
        v-show="expanded"
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 pt-2 border-t"
      >
        <div>
          <label class="block text-xs text-slate-500 mb-1">车牌号</label>
          <input
            v-model="pendingFilters.plate"
            type="text"
            placeholder="输入车牌号"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">车主姓名</label>
          <input
            v-model="pendingFilters.ownerName"
            type="text"
            placeholder="输入车主姓名"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">车主房号</label>
          <input
            v-model="pendingFilters.roomNumber"
            type="text"
            placeholder="输入车主房号"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">办理日期 起</label>
          <input
            v-model="pendingFilters.processDateStart"
            type="date"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">办理日期 止</label>
          <input
            v-model="pendingFilters.processDateEnd"
            type="date"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div class="flex items-end gap-3 lg:col-span-3">
          <button
            @click="doSearch"
            class="px-5 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition"
          >
            搜索
          </button>
          <button
            @click="resetSearch"
            class="px-5 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            重置
          </button>
        </div>
      </div>
    </div>

    <div class="px-4 py-2 bg-slate-50 border-b text-sm text-slate-600 flex flex-wrap gap-4">
      <span>租金合计：{{ formatMoney(totalStats.rent) }}</span>
      <span>管理费合计：{{ formatMoney(totalStats.managementFee) }}</span>
      <span>费用总计：{{ formatMoney(totalStats.total) }}</span>
    </div>

    <div class="overflow-auto flex-1">
      <table class="min-w-full text-sm text-left">
        <thead class="bg-slate-100 text-slate-600 sticky top-0">
          <tr>
            <th class="px-4 py-3 font-medium">序号</th>
            <th class="px-4 py-3 font-medium">办理日期</th>
            <th class="px-4 py-3 font-medium">车主房号</th>
            <th class="px-4 py-3 font-medium">车主姓名</th>
            <th class="px-4 py-3 font-medium">车主电话</th>
            <th class="px-4 py-3 font-medium">车牌号</th>
            <th class="px-4 py-3 font-medium">充费时间段</th>
            <th class="px-4 py-3 font-medium text-right">缴费月数</th>
            <th class="px-4 py-3 font-medium text-right">租金</th>
            <th class="px-4 py-3 font-medium text-right">管理费</th>
            <th class="px-4 py-3 font-medium">授权开始</th>
            <th class="px-4 py-3 font-medium">授权结束</th>
            <th class="px-4 py-3 font-medium text-right">授权月数</th>
            <th class="px-4 py-3 font-medium">租金赠送</th>
            <th class="px-4 py-3 font-medium">物业欠费</th>
            <th class="px-4 py-3 font-medium">经办人</th>
            <th class="px-4 py-3 font-medium">备注</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr
            v-for="(record, idx) in records"
            :key="record.id"
            class="hover:bg-slate-50"
          >
            <td class="px-4 py-3 text-slate-500">{{ startIndex() + idx }}</td>
            <td class="px-4 py-3">{{ record.processDate }}</td>
            <td class="px-4 py-3">{{ record.roomNumber || "--" }}</td>
            <td class="px-4 py-3">{{ record.ownerName || "--" }}</td>
            <td class="px-4 py-3">{{ record.ownerPhone || "--" }}</td>
            <td class="px-4 py-3 font-medium">{{ record.plate }}</td>
            <td class="px-4 py-3">{{ record.paymentPeriod }}</td>
            <td class="px-4 py-3 text-right">{{ record.paymentMonths }}</td>
            <td class="px-4 py-3 text-right">{{ formatMoney(record.rent) }}</td>
            <td class="px-4 py-3 text-right">{{ formatMoney(record.managementFee) }}</td>
            <td class="px-4 py-3">{{ record.actualStartDate }}</td>
            <td class="px-4 py-3">{{ record.actualEndDate }}</td>
            <td class="px-4 py-3 text-right">{{ record.actualMonths }}</td>
            <td class="px-4 py-3">{{ formatBool(record.rentGifted) }}</td>
            <td class="px-4 py-3">{{ formatBool(record.propertyFeeOwed) }}</td>
            <td class="px-4 py-3">{{ record.operator || "--" }}</td>
            <td class="px-4 py-3 text-slate-500">{{ record.remark || "--" }}</td>
          </tr>
          <tr v-if="records.length === 0">
            <td colspan="17" class="px-4 py-12 text-center text-slate-400">
              暂无记录，请点击“导入 Excel”导入月卡台账
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="p-4 border-t flex items-center justify-between">
      <span class="text-sm text-slate-500">
        第 {{ page }} / {{ totalPages() }} 页，共 {{ total.toLocaleString() }} 条
      </span>
      <div class="flex items-center gap-2">
        <button
          @click="setPage(1)"
          :disabled="page === 1"
          class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
        >
          首页
        </button>
        <button
          @click="setPage(page - 1)"
          :disabled="page === 1"
          class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
        >
          上一页
        </button>
        <button
          @click="setPage(page + 1)"
          :disabled="page === totalPages()"
          class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
        >
          下一页
        </button>
        <button
          @click="setPage(totalPages())"
          :disabled="page === totalPages()"
          class="px-3 py-1.5 border rounded hover:bg-slate-50 disabled:opacity-40 text-sm"
        >
          末页
        </button>
      </div>
    </div>
  </div>
</template>
