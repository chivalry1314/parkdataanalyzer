<script setup lang="ts">
import { ref, watch } from "vue";
import type { MonthlyRentalFilters, MonthlyRentalVehicle } from "../types";

const props = defineProps<{
  records: MonthlyRentalVehicle[];
  total: number;
  page: number;
  pageSize: number;
  filters: MonthlyRentalFilters;
  loading?: boolean;
  cleaning?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:page", value: number): void;
  (e: "update:pageSize", value: number): void;
  (e: "search", value: MonthlyRentalFilters): void;
  (e: "reset"): void;
  (e: "clean-addresses"): void;
  (e: "import-zip"): void;
  (e: "save", value: MonthlyRentalVehicle): void;
}>();

const showModal = ref(false);
const draft = ref<MonthlyRentalVehicle | null>(null);
const saving = ref(false);

const expanded = ref(false);
const pendingFilters = ref<MonthlyRentalFilters>({ ...props.filters });

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

function startEdit(record: MonthlyRentalVehicle) {
  draft.value = { ...record };
  showModal.value = true;
}

function closeModal() {
  showModal.value = false;
  draft.value = null;
  saving.value = false;
}

function saveEdit() {
  if (!draft.value) return;
  saving.value = true;
  emit("save", { ...draft.value });
}

defineExpose({ closeModal });
</script>

<template>
  <div class="bg-white rounded-lg shadow flex flex-col h-full">
    <div class="p-4 border-b flex flex-col gap-3">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h2 class="text-lg font-semibold text-slate-800">月租车列表</h2>
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
            placeholder="如：月租车"
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
            <th class="px-4 py-3 font-medium">车位组</th>
            <th class="px-4 py-3 font-medium text-right">车位数量</th>
            <th class="px-4 py-3 font-medium">来源文件</th>
            <th class="px-4 py-3 font-medium">操作</th>
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
            <td class="px-4 py-3">{{ record.parkingGroup || "--" }}</td>
            <td class="px-4 py-3 text-right">{{ record.parkingSpaceCount }}</td>
            <td class="px-4 py-3 text-slate-500">{{ record.sourceFile }}</td>
            <td class="px-4 py-3">
              <button
                @click="startEdit(record)"
                class="px-3 py-1 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded text-xs font-medium transition"
              >
                编辑
              </button>
            </td>
          </tr>
          <tr v-if="records.length === 0">
            <td colspan="12" class="px-4 py-12 text-center text-slate-400">
              暂无记录
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div
      v-if="showModal && draft"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      @click.self="closeModal"
    >
      <div class="bg-white rounded-lg shadow-xl w-full max-w-lg mx-4 flex flex-col max-h-[90vh]">
        <div class="p-4 border-b flex items-center justify-between">
          <h3 class="text-lg font-semibold text-slate-800">编辑月租车</h3>
          <button
            @click="closeModal"
            class="text-slate-400 hover:text-slate-600"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
        <div class="p-4 overflow-auto space-y-4">
          <div>
            <label class="block text-xs text-slate-500 mb-1">车牌号</label>
            <input
              v-model="draft.plate"
              type="text"
              class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-xs text-slate-500 mb-1">车辆类型</label>
            <input
              v-model="draft.vehicleType"
              type="text"
              class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-xs text-slate-500 mb-1">车主电话</label>
            <input
              v-model="draft.ownerPhone"
              type="text"
              class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-xs text-slate-500 mb-1">车主姓名</label>
            <input
              v-model="draft.ownerName"
              type="text"
              class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-xs text-slate-500 mb-1">车主地址</label>
            <input
              v-model="draft.ownerAddress"
              type="text"
              class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-xs text-slate-500 mb-1">开始有效时间</label>
              <input
                v-model="draft.startDate"
                type="date"
                class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="block text-xs text-slate-500 mb-1">结束有效时间</label>
              <input
                v-model="draft.endDate"
                type="date"
                class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-xs text-slate-500 mb-1">车位组</label>
              <input
                v-model="draft.parkingGroup"
                type="text"
                class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="block text-xs text-slate-500 mb-1">车位数量</label>
              <input
                v-model.number="draft.parkingSpaceCount"
                type="number"
                class="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
        </div>
        <div class="p-4 border-t flex justify-end gap-3">
          <button
            @click="closeModal"
            :disabled="saving"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 disabled:opacity-50 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            取消
          </button>
          <button
            @click="saveEdit"
            :disabled="saving"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            {{ saving ? "保存中..." : "保存" }}
          </button>
        </div>
      </div>
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
