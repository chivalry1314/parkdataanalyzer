<script setup lang="ts">
import { ref, watch } from "vue";
import type { FreeCar, FreeCarFilters } from "../types";

const props = defineProps<{
  records: FreeCar[];
  total: number;
  page: number;
  pageSize: number;
  filters: FreeCarFilters;
  loading?: boolean;
  cleaning?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:page", value: number): void;
  (e: "update:pageSize", value: number): void;
  (e: "search", value: FreeCarFilters): void;
  (e: "reset"): void;
  (e: "clean-addresses"): void;
  (e: "import-zip"): void;
}>();

const expanded = ref(false);
const pendingFilters = ref<FreeCarFilters>({ ...props.filters });

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
    startDateStart: "",
    startDateEnd: "",
    endDateStart: "",
    endDateEnd: "",
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
          <h2 class="text-lg font-semibold text-slate-800">免费车列表</h2>
          <p class="text-sm text-slate-500">共 {{ total.toLocaleString() }} 条记录</p>
        </div>
        <div class="flex items-center gap-3">
          <button
            @click="$emit('import-zip')"
            :disabled="loading"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            导入 ZIP
          </button>
          <button
            @click="$emit('clean-addresses')"
            :disabled="cleaning"
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            {{ cleaning ? "正在整理..." : "地址填补" }}
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
          <label class="block text-xs text-slate-500 mb-1">车辆类型</label>
          <input
            v-model="pendingFilters.vehicleType"
            type="text"
            placeholder="如：免费车"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">开始有效时间 起</label>
          <input
            v-model="pendingFilters.startDateStart"
            type="date"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">开始有效时间 止</label>
          <input
            v-model="pendingFilters.startDateEnd"
            type="date"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">结束有效时间 起</label>
          <input
            v-model="pendingFilters.endDateStart"
            type="date"
            class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-xs text-slate-500 mb-1">结束有效时间 止</label>
          <input
            v-model="pendingFilters.endDateEnd"
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

    <div class="overflow-auto flex-1">
      <table class="min-w-full text-sm text-left">
        <thead class="bg-slate-100 text-slate-600 sticky top-0">
          <tr>
            <th class="px-4 py-3 font-medium">序号</th>
            <th class="px-4 py-3 font-medium">车牌号</th>
            <th class="px-4 py-3 font-medium">车辆类型</th>
            <th class="px-4 py-3 font-medium">车主电话</th>
            <th class="px-4 py-3 font-medium">车主姓名</th>
            <th class="px-4 py-3 font-medium">车主地址</th>
            <th class="px-4 py-3 font-medium">开始有效时间</th>
            <th class="px-4 py-3 font-medium">结束有效时间</th>
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
            <td class="px-4 py-3">{{ record.ownerPhone || "--" }}</td>
            <td class="px-4 py-3">{{ record.ownerName || "--" }}</td>
            <td class="px-4 py-3">{{ record.ownerAddress || "--" }}</td>
            <td class="px-4 py-3 whitespace-nowrap">{{ record.startDate }}</td>
            <td class="px-4 py-3 whitespace-nowrap">{{ record.endDate }}</td>
            <td class="px-4 py-3 text-slate-500">{{ record.sourceFile }}</td>
          </tr>
          <tr v-if="records.length === 0">
            <td colspan="9" class="px-4 py-12 text-center text-slate-400">
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
