<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from "vue";
import { open, save, confirm } from "@tauri-apps/plugin-dialog";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import * as XLSX from "xlsx";
import { checkMonthlyCardLedgerXlsxFile } from "../utils/parser";
import type {
  MonthlyCardLedger,
  MonthlyCardLedgerCheckResult,
  MonthlyCardLedgerFilters,
} from "../types";

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
const importMenuOpen = ref(false);
const importMenuRef = ref<HTMLDivElement | null>(null);
const pendingFilters = ref<MonthlyCardLedgerFilters>({ ...props.filters });

function handleDocumentClick(e: MouseEvent) {
  if (importMenuRef.value && !importMenuRef.value.contains(e.target as Node)) {
    importMenuOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener("click", handleDocumentClick);
});

onBeforeUnmount(() => {
  document.removeEventListener("click", handleDocumentClick);
});

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

const checkModalOpen = ref(false);
const checkFile = ref<File | null>(null);
const checkFileInput = ref<HTMLInputElement | null>(null);
const checkResult = ref<MonthlyCardLedgerCheckResult | null>(null);
const checking = ref(false);
const checkProgress = ref(0);

function openCheckModal() {
  checkModalOpen.value = true;
  checkFile.value = null;
  checkResult.value = null;
  checkProgress.value = 0;
  if (checkFileInput.value) checkFileInput.value.value = "";
}

function closeCheckModal() {
  checkModalOpen.value = false;
}

function triggerCheckFileSelect() {
  checkFileInput.value?.click();
}

function handleCheckFileChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    checkFile.value = file;
    checkResult.value = null;
  }
}

function readCheckFile(file: File): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const arrayBuffer = reader.result as ArrayBuffer;
      resolve(new Uint8Array(arrayBuffer));
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

async function runCheck() {
  if (!checkFile.value) return;
  checking.value = true;
  checkProgress.value = 0;
  try {
    const data = await readCheckFile(checkFile.value);
    checkResult.value = await checkMonthlyCardLedgerXlsxFile(
      data,
      checkFile.value.name,
      (current, total) => {
        checkProgress.value = total > 0 ? Math.round((current / total) * 100) : 0;
      }
    );
  } catch (err) {
    checkResult.value = {
      total: 0,
      valid: 0,
      invalid: 0,
      issues: [
        {
          rowIndex: 0,
          plate: "",
          field: "文件读取",
          issue: err instanceof Error ? err.message : String(err),
          value: "",
        },
      ],
    };
  } finally {
    checking.value = false;
    checkProgress.value = 0;
  }
}

async function downloadCheckResult() {
  if (!checkResult.value) return;
  const issues = checkResult.value.issues || [];
  try {
    const filePath = await save({
      filters: [{ name: "Excel 文件", extensions: ["xlsx"] }],
      defaultPath: `月卡台账导入检查结果_${new Date().toISOString().slice(0, 10)}.xlsx`,
    });
    if (!filePath) return;

    const rows = issues.map((issue) => ({
      "Excel行号": issue.rowIndex || "",
      "车牌号": issue.plate,
      "字段": issue.field,
      "问题描述": issue.issue,
      "当前值": issue.value,
    }));

    const worksheet = XLSX.utils.json_to_sheet(rows);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "检查结果");
    const data = XLSX.write(workbook, { bookType: "xlsx", type: "array" });

    await writeFile(filePath, new Uint8Array(data));
  } catch (err) {
    alert(err instanceof Error ? err.message : String(err));
  }
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
          <div ref="importMenuRef" class="relative">
            <button
              @click.stop="importMenuOpen = !importMenuOpen"
              :disabled="loading"
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition flex items-center gap-2"
            >
              导入
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                  clip-rule="evenodd"
                />
              </svg>
            </button>
            <div
              v-if="importMenuOpen"
              @click.stop
              class="absolute right-0 mt-1 w-36 bg-white border rounded-lg shadow-lg py-1 z-20"
            >
              <button
                @click="importFile(); importMenuOpen = false"
                :disabled="loading"
                class="w-full text-left px-4 py-2 text-sm text-slate-700 hover:bg-slate-100 disabled:opacity-50"
              >
                导入 Excel
              </button>
              <button
                @click="openCheckModal(); importMenuOpen = false"
                class="w-full text-left px-4 py-2 text-sm text-slate-700 hover:bg-slate-100"
              >
                导入检查
              </button>
            </div>
          </div>
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
            v-for="(record, idx) in (records || [])"
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
          <tr v-if="(records || []).length === 0">
            <td colspan="17" class="px-4 py-12 text-center text-slate-400">
              暂无记录，请点击“导入”选择“导入 Excel”导入月卡台账
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

  <div
    v-if="checkModalOpen"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
    @click.self="closeCheckModal"
  >
    <div class="bg-white rounded-lg shadow-xl w-full max-w-5xl max-h-[90vh] flex flex-col">
      <div class="p-4 border-b flex items-center justify-between">
        <h3 class="text-lg font-semibold text-slate-800">月卡台账导入检查</h3>
        <button
          @click="closeCheckModal"
          class="text-slate-400 hover:text-slate-600"
          aria-label="关闭"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <div class="p-4 space-y-4 overflow-auto">
        <div class="flex items-center gap-3">
          <input
            ref="checkFileInput"
            type="file"
            accept=".xlsx"
            class="hidden"
            @change="handleCheckFileChange"
          />
          <button
            @click="triggerCheckFileSelect"
            class="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg text-sm font-medium transition"
          >
            选择 Excel 文件
          </button>
          <span class="text-sm text-slate-600">
            {{ checkFile ? checkFile.name : "未选择文件" }}
          </span>
        </div>

        <div class="flex items-center gap-3">
          <button
            @click="runCheck"
            :disabled="!checkFile || checking"
            class="px-5 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition"
          >
            {{ checking ? "检查中..." : "开始检查" }}
          </button>
        </div>

        <div v-if="checking" class="space-y-1">
          <div class="flex items-center justify-between text-xs text-slate-600">
            <span>正在检查数据...</span>
            <span>{{ checkProgress }}%</span>
          </div>
          <div class="w-full bg-slate-200 rounded-full h-2 overflow-hidden">
            <div
              class="bg-blue-600 h-2 rounded-full transition-all duration-200"
              :style="{ width: `${checkProgress}%` }"
            ></div>
          </div>
        </div>

        <div v-if="checkResult" class="space-y-3">
          <div class="grid grid-cols-3 gap-3">
            <div class="bg-slate-50 rounded-lg p-3 text-center">
              <p class="text-xs text-slate-500">总记录数</p>
              <p class="text-xl font-bold text-slate-800">{{ checkResult.total }}</p>
            </div>
            <div class="bg-emerald-50 rounded-lg p-3 text-center">
              <p class="text-xs text-emerald-600">通过</p>
              <p class="text-xl font-bold text-emerald-700">{{ checkResult.valid }}</p>
            </div>
            <div class="bg-rose-50 rounded-lg p-3 text-center">
              <p class="text-xs text-rose-600">异常</p>
              <p class="text-xl font-bold text-rose-700">{{ checkResult.invalid }}</p>
            </div>
          </div>

          <div class="border rounded-lg overflow-hidden">
            <div class="max-h-80 overflow-auto">
              <table class="min-w-full text-sm text-left">
                <thead class="bg-slate-100 text-slate-600 sticky top-0">
                  <tr>
                    <th class="px-4 py-2 font-medium">Excel 行号</th>
                    <th class="px-4 py-2 font-medium">车牌号</th>
                    <th class="px-4 py-2 font-medium">字段</th>
                    <th class="px-4 py-2 font-medium">问题描述</th>
                    <th class="px-4 py-2 font-medium">当前值</th>
                  </tr>
                </thead>
                <tbody class="divide-y">
                  <tr
                    v-for="(issue, idx) in (checkResult.issues || [])"
                    :key="idx"
                    class="hover:bg-slate-50"
                  >
                    <td class="px-4 py-2">{{ issue.rowIndex || "-" }}</td>
                    <td class="px-4 py-2">{{ issue.plate || "-" }}</td>
                    <td class="px-4 py-2">{{ issue.field }}</td>
                    <td class="px-4 py-2 text-rose-600">{{ issue.issue }}</td>
                    <td class="px-4 py-2 text-slate-500">{{ issue.value || "-" }}</td>
                  </tr>
                  <tr v-if="(checkResult.issues || []).length === 0">
                    <td colspan="5" class="px-4 py-8 text-center text-slate-400">
                      未发现问题
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div class="flex justify-end">
            <button
              @click="downloadCheckResult"
              :disabled="(checkResult.issues || []).length === 0"
              class="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition flex items-center gap-2"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 11.586V3a1 1 0 112 0v8.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z"
                  clip-rule="evenodd"
                />
              </svg>
              下载检查结果
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
