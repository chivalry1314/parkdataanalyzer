<script setup lang="ts">
import { ref, watch } from "vue";
import type { ParkingRecord, RecordFilters } from "../types";

const props = defineProps<{
  records: ParkingRecord[];
  total: number;
  page: number;
  pageSize: number;
  filters: RecordFilters;
}>();

const emit = defineEmits<{
  (e: "update:page", value: number): void;
  (e: "update:pageSize", value: number): void;
  (e: "search", value: RecordFilters): void;
  (e: "reset"): void;
}>();

const expanded = ref(false);

const pendingFilters = ref<RecordFilters>({ ...props.filters });

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
    vehicleType: "",
    chargeType: "",
    releaseMethod: "",
    paymentMethod: "",
    entryTimeStart: "",
    entryTimeEnd: "",
    exitTimeStart: "",
    exitTimeEnd: "",
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
</script>

<template>
  <div class="bg-white rounded-lg shadow flex flex-col h-full">
    <div class="p-4 border-b flex flex-col gap-3">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h2 class="text-lg font-semibold text-slate-800">停车收费记录</h2>
          <p class="text-sm text-slate-500">共 {{ total.toLocaleString() }} 条记录</p>
        </div>
        <div class="flex items-center gap-3">
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
          <label class="block text-xs text-slate-500 mb-1">车辆类型</label>
          <input
            v-model="pendingFilters.vehicleType"
            type="text"
            placeholder="如：临时车、月租车"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">收费车型</label>
          <input
            v-model="pendingFilters.chargeType"
            type="text"
            placeholder="如：小型车、新能源车"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">放行方式</label>
          <input
            v-model="pendingFilters.releaseMethod"
            type="text"
            placeholder="如：自动放行、免费放行"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">缴费方式</label>
          <input
            v-model="pendingFilters.paymentMethod"
            type="text"
            placeholder="输入缴费方式"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">进场时间 起</label>
          <input
            v-model="pendingFilters.entryTimeStart"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">进场时间 止</label>
          <input
            v-model="pendingFilters.entryTimeEnd"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">出场时间 起</label>
          <input
            v-model="pendingFilters.exitTimeStart"
            type="datetime-local"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">出场时间 止</label>
          <input
            v-model="pendingFilters.exitTimeEnd"
            type="datetime-local"
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

    <div class="overflow-auto flex-1">
      <table class="min-w-full text-sm text-left">
        <thead class="bg-slate-100 text-slate-600 sticky top-0">
          <tr>
            <th class="px-4 py-3 font-medium">序号</th>
            <th class="px-4 py-3 font-medium">车牌号</th>
            <th class="px-4 py-3 font-medium">车辆类型</th>
            <th class="px-4 py-3 font-medium">收费车型</th>
            <th class="px-4 py-3 font-medium">进场时间</th>
            <th class="px-4 py-3 font-medium">出场时间</th>
            <th class="px-4 py-3 font-medium">停车时长</th>
            <th class="px-4 py-3 font-medium text-right">应收金额</th>
            <th class="px-4 py-3 font-medium text-right">实收金额</th>
            <th class="px-4 py-3 font-medium">缴费方式</th>
            <th class="px-4 py-3 font-medium">放行方式</th>
            <th class="px-4 py-3 font-medium">车主姓名</th>
            <th class="px-4 py-3 font-medium">来源文件</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr
            v-for="(record, idx) in records"
            :key="record.id"
            class="hover:bg-slate-50"
          >
            <td class="px-4 py-3 text-slate-500">{{ startIndex() + idx }}</td>
            <td class="px-4 py-3 font-medium">{{ record.plate }}</td>
            <td class="px-4 py-3">{{ record.vehicleType }}</td>
            <td class="px-4 py-3">{{ record.chargeType }}</td>
            <td class="px-4 py-3 whitespace-nowrap">{{ record.entryTime }}</td>
            <td class="px-4 py-3 whitespace-nowrap">{{ record.exitTime }}</td>
            <td class="px-4 py-3">{{ record.duration }}</td>
            <td class="px-4 py-3 text-right">{{ record.dueAmount.toFixed(2) }}</td>
            <td class="px-4 py-3 text-right">{{ record.paidAmount.toFixed(2) }}</td>
            <td class="px-4 py-3">{{ record.paymentMethod || "--" }}</td>
            <td class="px-4 py-3">{{ record.releaseMethod }}</td>
            <td class="px-4 py-3">{{ record.ownerName || "--" }}</td>
            <td class="px-4 py-3 text-slate-500">{{ record.sourceFile }}</td>
          </tr>
          <tr v-if="records.length === 0">
            <td colspan="13" class="px-4 py-12 text-center text-slate-400">
              暂无记录
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
