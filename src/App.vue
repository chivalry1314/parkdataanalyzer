<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { confirm, message, open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import FilterStats from "./components/FilterStats.vue";
import RecordList from "./components/RecordList.vue";
import StatsPanel from "./components/StatsPanel.vue";
import VehicleCharges from "./components/VehicleCharges.vue";
import Onboarding from "./views/Onboarding.vue";
import Settings from "./views/Settings.vue";
import { parseZipFile } from "./utils/parser";
import {
  clearRecords,
  getDbPath,
  getFilteredStats,
  listRecords,
  saveRecords,
} from "./api";
import type { ParkingRecord, RecordFilters, StatsSummary } from "./types";

const loading = ref(false);
const progress = ref(0);
const progressText = ref("准备中...");
const messageText = ref("");
const messageType = ref<"info" | "error">("info");
const activeMenu = ref<"records" | "stats" | "vehicle" | "settings">("records");
const menuCollapsed = ref(false);
const dbReady = ref<boolean | null>(null);

const menuItems = [
  { key: "records", label: "记录列表", short: "列" },
  { key: "vehicle", label: "车辆收费", short: "车" },
  { key: "stats", label: "统计分析", short: "统" },
  { key: "settings", label: "设置", short: "设" },
] as const;

const records = ref<ParkingRecord[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(20);
const filters = ref<RecordFilters>({
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
});
const appliedFilters = ref<RecordFilters>({ ...filters.value });
const filterStats = ref<StatsSummary | null>(null);
const statsRefreshKey = ref(0);
const filePath = ref("");

async function selectFile() {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "压缩文件", extensions: ["zip"] }],
    });
    if (!selected) return;
    const path = Array.isArray(selected) ? selected[0] : selected;
    filePath.value = path;
    const data = await readFile(path);
    await handleFileLoad(data, path);
  } catch (err) {
    handleError(err instanceof Error ? err.message : String(err));
  }
}

function formatDateTime(value: string): string {
  if (!value) return "";
  return value.replace("T", " ").replace(/\+.*$/, "").padEnd(19, ":00").slice(0, 19);
}

function filterParams(filters: RecordFilters) {
  return {
    plate: filters.plate || undefined,
    ownerName: filters.ownerName || undefined,
    vehicleType: filters.vehicleType || undefined,
    chargeType: filters.chargeType || undefined,
    releaseMethod: filters.releaseMethod || undefined,
    paymentMethod: filters.paymentMethod || undefined,
    entryTimeStart: formatDateTime(filters.entryTimeStart) || undefined,
    entryTimeEnd: formatDateTime(filters.entryTimeEnd) || undefined,
    exitTimeStart: formatDateTime(filters.exitTimeStart) || undefined,
    exitTimeEnd: formatDateTime(filters.exitTimeEnd) || undefined,
  };
}

async function loadRecords() {
  try {
    const params = filterParams(appliedFilters.value);
    const [result, stats] = await Promise.all([
      listRecords({
        ...params,
        page: page.value,
        perPage: pageSize.value,
      }),
      getFilteredStats(params),
    ]);
    records.value = result.items;
    total.value = result.total;
    filterStats.value = stats;
  } catch (err) {
    showError(`加载记录失败：${err instanceof Error ? err.message : String(err)}`);
  }
}

async function handleSearch(newFilters: RecordFilters) {
  appliedFilters.value = { ...newFilters };
  filters.value = { ...newFilters };
  page.value = 1;
  await loadRecords();
}

async function handleReset() {
  appliedFilters.value = {
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
  filters.value = { ...appliedFilters.value };
  page.value = 1;
  await loadRecords();
}

function showInfo(text: string) {
  messageText.value = text;
  messageType.value = "info";
}

function showError(text: string) {
  messageText.value = text;
  messageType.value = "error";
}

function closeMessage() {
  messageText.value = "";
}

async function handleFileLoad(data: Uint8Array, _path: string) {
  loading.value = true;
  progress.value = 0;
  progressText.value = "正在解压并解析 ZIP 文件...";
  messageText.value = "";

  try {
    const result = await parseZipFile(data, (current, totalFiles, fileName) => {
      progress.value = totalFiles > 0 ? Math.round((current / totalFiles) * 50) : 0;
      progressText.value = `正在解析 ${fileName} (${current}/${totalFiles})`;
    });

    if (result.records.length === 0 && result.errors.length === 0) {
      showError("未在压缩包中找到有效的 Excel 记录文件。");
      return;
    }

    progress.value = 55;
    progressText.value = `正在保存 ${result.records.length.toLocaleString()} 条记录到数据库...`;
    await saveRecords({ records: result.records, replace: true });

    progress.value = 100;
    progressText.value = "处理完成";

    const info = `成功处理 ${result.records.length.toLocaleString()} 条停车收费记录。`;
    showInfo(info);
    await message(info, { title: "处理完成", kind: "info" });

    if (result.errors.length > 0) {
      showError(result.errors.join("；"));
    }

    page.value = 1;
    appliedFilters.value = {
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
    filters.value = { ...appliedFilters.value };
    await loadRecords();
    statsRefreshKey.value++;
  } catch (err) {
    const text = `处理失败：${err instanceof Error ? err.message : String(err)}`;
    showError(text);
    await message(text, { title: "处理失败", kind: "error" });
  } finally {
    loading.value = false;
    progress.value = 0;
    progressText.value = "";
  }
}

function handleError(text: string) {
  message(text, { title: "错误", kind: "error" });
}

async function handleClear() {
  const ok = await confirm("确定要清空数据库中的所有停车记录吗？此操作不可恢复。", {
    title: "请确认",
    kind: "warning",
  });
  if (!ok) return;
  loading.value = true;
  progressText.value = "正在清空数据库...";
  try {
    await clearRecords();
    showInfo("数据库已清空。");
    page.value = 1;
    appliedFilters.value = {
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
    filters.value = { ...appliedFilters.value };
    await loadRecords();
    statsRefreshKey.value++;
  } catch (err) {
    showError(`清空失败：${err instanceof Error ? err.message : String(err)}`);
  } finally {
    loading.value = false;
    progressText.value = "";
  }
}

watch(page, () => {
  loadRecords();
});

onMounted(async () => {
  try {
    await getDbPath();
    dbReady.value = true;
    await loadRecords();
  } catch {
    dbReady.value = false;
  }
});
</script>

<template>
  <div v-if="dbReady === null" class="h-screen flex items-center justify-center bg-slate-100">
    <div class="text-sm text-slate-500">正在加载...</div>
  </div>
  <Onboarding v-else-if="!dbReady" />
  <div v-else class="h-screen flex flex-col overflow-hidden">
    <header class="bg-slate-900 text-white px-6 py-3 flex items-center justify-between shrink-0">
      <div>
        <h1 class="text-lg font-bold">停车收费记录解析工具</h1>
        <p class="text-slate-400 text-xs">解析 ZIP 压缩包中的进出场记录表，支持查看与统计</p>
      </div>
      <div class="flex items-center gap-3">
        <button
          @click="handleClear"
          :disabled="loading"
          class="px-4 py-2 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded-lg text-sm font-medium transition"
        >
          清空数据库
        </button>
        <button
          @click="selectFile"
          :disabled="loading"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 rounded-lg text-sm font-medium transition"
        >
          选择 ZIP 文件
        </button>
      </div>
    </header>

    <div class="flex-1 flex overflow-hidden">
      <aside
        :class="[
          'bg-slate-800 text-white flex flex-col shrink-0 transition-all duration-300',
          menuCollapsed ? 'w-16' : 'w-48',
        ]"
      >
        <div
          :class="[
            'flex items-center py-3 border-b border-slate-700',
            menuCollapsed ? 'justify-center px-2' : 'justify-between px-4',
          ]"
        >
          <span v-if="!menuCollapsed" class="font-semibold text-sm">菜单</span>
          <button
            @click="menuCollapsed = !menuCollapsed"
            class="p-1.5 rounded-md text-slate-400 hover:text-white hover:bg-slate-700 transition"
            :title="menuCollapsed ? '展开菜单' : '收起菜单'"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 transition-transform duration-300"
              :class="menuCollapsed ? 'rotate-180' : ''"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z"
                clip-rule="evenodd"
              />
            </svg>
          </button>
        </div>
        <nav class="flex-1 p-3 space-y-1">
          <button
            v-for="item in menuItems"
            :key="item.key"
            @click="activeMenu = item.key"
            :class="[
              'w-full rounded-lg text-sm font-medium transition flex items-center',
              activeMenu === item.key
                ? 'bg-blue-600 text-white'
                : 'text-slate-300 hover:bg-slate-700',
              menuCollapsed ? 'justify-center px-2 py-3' : 'text-left px-4 py-3',
            ]"
            :title="item.label"
          >
            <span v-if="menuCollapsed" class="text-base">{{ item.short }}</span>
            <span v-else>{{ item.label }}</span>
          </button>
        </nav>
      </aside>

      <main class="flex-1 overflow-hidden flex flex-col p-4 gap-3 bg-slate-50">
        <FilterStats
          v-if="activeMenu === 'records'"
          :stats="filterStats"
          :loading="loading"
          :progress="progress"
          :progress-text="progressText"
        />

        <div
          v-if="messageText"
          :class="[
            'rounded-lg px-3 py-2 text-xs flex items-start justify-between gap-2',
            messageType === 'error'
              ? 'bg-amber-50 border border-amber-200 text-amber-800'
              : 'bg-blue-50 border border-blue-200 text-blue-800',
          ]"
        >
          <span>{{ messageText }}</span>
          <button
            @click="closeMessage"
            class="shrink-0 leading-none hover:opacity-70"
            aria-label="关闭"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
              <path
                fill-rule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clip-rule="evenodd"
              />
            </svg>
          </button>
        </div>

        <div class="flex-1 overflow-hidden min-h-0">
          <RecordList
            v-if="activeMenu === 'records'"
            v-model:page="page"
            :page-size="pageSize"
            :filters="filters"
            :records="records"
            :total="total"
            @search="handleSearch"
            @reset="handleReset"
            @update:page-size="(v) => { pageSize = v; page = 1; loadRecords(); }"
          />
          <div v-else-if="activeMenu === 'stats'" class="h-full overflow-auto pr-2 pb-2">
            <StatsPanel :refresh-key="statsRefreshKey" />
          </div>
          <Settings v-else-if="activeMenu === 'settings'" />
          <VehicleCharges v-else />
        </div>
      </main>
    </div>
  </div>
</template>
