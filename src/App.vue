<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { confirm, message, open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import BuildingCharges from "./components/BuildingCharges.vue";
import FilterStats from "./components/FilterStats.vue";
import FreeCars from "./components/FreeCars.vue";
import ImportProgress from "./components/ImportProgress.vue";
import MonthlyCardLedgerComponent from "./components/MonthlyCardLedger.vue";
import MonthlyRentals from "./components/MonthlyRentals.vue";
import RecordList from "./components/RecordList.vue";
import StatsPanel from "./components/StatsPanel.vue";
import VehicleCharges from "./components/VehicleCharges.vue";
import Onboarding from "./views/Onboarding.vue";
import Settings from "./views/Settings.vue";
import { parseFreeCarZipFile, parseMonthlyCardLedgerXlsxFile, parseMonthlyRentalZipFile, parseZipFile } from "./utils/parser";
import {
  cleanFreeCarAddresses,
  cleanMonthlyRentalAddresses,
  clearFreeCars,
  clearMonthlyCardLedgers,
  clearMonthlyRentals,
  clearRecords,
  getBuildingCharges,
  getDbPath,
  getFilteredStats,
  getSetting,
  listFreeCars,
  setSetting,
  listMonthlyCardLedgers,
  listMonthlyRentals,
  listRecords,
  saveFreeCars,
  saveMonthlyCardLedgers,
  saveMonthlyRentals,
  saveRecords,
  updateFreeCar,
  updateMonthlyRental,
} from "./api";
import type {
  BuildingChargesFilters,
  FreeCar,
  FreeCarFilters,
  MonthlyCardLedger,
  MonthlyCardLedgerFilters,
  MonthlyRentalFilters,
  MonthlyRentalVehicle,
  ParkingRecord,
  RecordFilters,
  StatsSummary,
} from "./types";

const loading = ref(false);
const cleaningAddresses = ref(false);
const progress = ref(0);
const progressText = ref("准备中...");
const messageText = ref("");
const messageType = ref<"info" | "error">("info");
const activeMenu = ref<"records" | "stats" | "vehicle" | "monthly" | "free" | "building" | "settings" | "monthlyCardLedger">("records");
const menuCollapsed = ref(false);
const dbReady = ref<boolean | null>(null);
const dbPath = ref("");
const lastImportTime = ref("");
const currentTime = ref(formatCurrentDateTime(new Date()));
let currentTimeTimer: ReturnType<typeof setInterval> | null = null;

function formatCurrentDateTime(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

async function updateLastImportTime() {
  const now = formatCurrentDateTime(new Date());
  lastImportTime.value = now;
  try {
    await setSetting("lastImportTime", now);
  } catch {
    // ignore setting save errors
  }
}

const menuItems = [
  { key: "records", label: "记录列表", short: "列" },
  { key: "monthly", label: "月租车", short: "月" },
  { key: "free", label: "免费车", short: "免" },
  { key: "monthlyCardLedger", label: "月卡台账", short: "卡" },
  { key: "building", label: "楼栋收费", short: "栋" },
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

const monthlyRecords = ref<MonthlyRentalVehicle[]>([]);
const monthlyTotal = ref(0);
const monthlyPage = ref(1);
const monthlyPageSize = ref(20);
const monthlyFilters = ref<MonthlyRentalFilters>({
  plate: "",
  ownerName: "",
  vehicleType: "",
  startDateStart: "",
  startDateEnd: "",
  endDateStart: "",
  endDateEnd: "",
});
const appliedMonthlyFilters = ref<MonthlyRentalFilters>({ ...monthlyFilters.value });

const freeCarRecords = ref<FreeCar[]>([]);
const freeCarTotal = ref(0);
const freeCarPage = ref(1);
const freeCarPageSize = ref(20);
const freeCarFilters = ref<FreeCarFilters>({
  plate: "",
  ownerName: "",
  vehicleType: "",
  startDateStart: "",
  startDateEnd: "",
  endDateStart: "",
  endDateEnd: "",
});
const appliedFreeCarFilters = ref<FreeCarFilters>({ ...freeCarFilters.value });

const monthlyCardLedgerRecords = ref<MonthlyCardLedger[]>([]);
const monthlyCardLedgerTotal = ref(0);
const monthlyCardLedgerPage = ref(1);
const monthlyCardLedgerPageSize = ref(20);
const monthlyCardLedgerFilters = ref<MonthlyCardLedgerFilters>({
  plate: "",
  ownerName: "",
  roomNumber: "",
  processDateStart: "",
  processDateEnd: "",
});
const appliedMonthlyCardLedgerFilters = ref<MonthlyCardLedgerFilters>({ ...monthlyCardLedgerFilters.value });

const buildingChargesRef = ref<InstanceType<typeof BuildingCharges> | null>(null);
const monthlyRentalsRef = ref<InstanceType<typeof MonthlyRentals> | null>(null);
const freeCarsRef = ref<InstanceType<typeof FreeCars> | null>(null);

const monthlyEditable = ref(false);
const freeCarEditable = ref(false);

function parseBoolSetting(value: string | null, defaultValue: boolean): boolean {
  if (value === null || value === undefined) return defaultValue;
  return value === "1" || value.toLowerCase() === "true";
}

async function loadEditSettings() {
  try {
    const [monthly, freeCar] = await Promise.all([
      getSetting("monthly_rental_editable"),
      getSetting("free_car_editable"),
    ]);
    monthlyEditable.value = parseBoolSetting(monthly, false);
    freeCarEditable.value = parseBoolSetting(freeCar, false);
  } catch (err) {
    showError(`加载编辑设置失败：${err instanceof Error ? err.message : String(err)}`);
  }
}
const buildingChargesLoading = ref(false);
const buildingChargesFilters = ref<BuildingChargesFilters>({
  building: "",
  unit: "",
  room: "",
  entryTimeStart: "",
  entryTimeEnd: "",
  exitTimeStart: "",
  exitTimeEnd: "",
});

async function selectFile(type: "records" | "monthly" | "free") {
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
    await handleFileLoad(data, path, type);
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

function monthlyFilterParams(filters: MonthlyRentalFilters) {
  return {
    plate: filters.plate || undefined,
    ownerName: filters.ownerName || undefined,
    vehicleType: filters.vehicleType || undefined,
    startDateStart: filters.startDateStart || undefined,
    startDateEnd: filters.startDateEnd || undefined,
    endDateStart: filters.endDateStart || undefined,
    endDateEnd: filters.endDateEnd || undefined,
  };
}

async function loadMonthlyRentals() {
  try {
    const result = await listMonthlyRentals({
      ...monthlyFilterParams(appliedMonthlyFilters.value),
      page: monthlyPage.value,
      perPage: monthlyPageSize.value,
    });
    monthlyRecords.value = result.items;
    monthlyTotal.value = result.total;
  } catch (err) {
    showError(`加载月租车记录失败：${err instanceof Error ? err.message : String(err)}`);
  }
}

async function handleMonthlySearch(newFilters: MonthlyRentalFilters) {
  appliedMonthlyFilters.value = { ...newFilters };
  monthlyFilters.value = { ...newFilters };
  monthlyPage.value = 1;
  await loadMonthlyRentals();
}

async function handleMonthlyReset() {
  appliedMonthlyFilters.value = {
    plate: "",
    ownerName: "",
    vehicleType: "",
    startDateStart: "",
    startDateEnd: "",
    endDateStart: "",
    endDateEnd: "",
  };
  monthlyFilters.value = { ...appliedMonthlyFilters.value };
  monthlyPage.value = 1;
  await loadMonthlyRentals();
}

async function handleCleanMonthlyRentalAddresses() {
  cleaningAddresses.value = true;
  try {
    const count = await cleanMonthlyRentalAddresses();
    showInfo(`已整理 ${count} 条月租车地址信息。`);
    await message(`已整理 ${count} 条月租车地址信息。`, { title: "整理完成", kind: "info" });
    await loadMonthlyRentals();
  } catch (err) {
    const text = `地址整理失败：${err instanceof Error ? err.message : String(err)}`;
    showError(text);
    await message(text, { title: "整理失败", kind: "error" });
  } finally {
    cleaningAddresses.value = false;
  }
}

function freeCarFilterParams(filters: FreeCarFilters) {
  return {
    plate: filters.plate || undefined,
    ownerName: filters.ownerName || undefined,
    vehicleType: filters.vehicleType || undefined,
    startDateStart: filters.startDateStart || undefined,
    startDateEnd: filters.startDateEnd || undefined,
    endDateStart: filters.endDateStart || undefined,
    endDateEnd: filters.endDateEnd || undefined,
  };
}

async function loadFreeCars() {
  try {
    const result = await listFreeCars({
      ...freeCarFilterParams(appliedFreeCarFilters.value),
      page: freeCarPage.value,
      perPage: freeCarPageSize.value,
    });
    freeCarRecords.value = result.items;
    freeCarTotal.value = result.total;
  } catch (err) {
    showError(`加载免费车记录失败：${err instanceof Error ? err.message : String(err)}`);
  }
}

async function handleFreeCarSearch(newFilters: FreeCarFilters) {
  appliedFreeCarFilters.value = { ...newFilters };
  freeCarFilters.value = { ...newFilters };
  freeCarPage.value = 1;
  await loadFreeCars();
}

async function handleSaveFreeCar(record: FreeCar) {
  try {
    await updateFreeCar(record);
    showInfo("免费车记录已保存");
    await loadFreeCars();
    freeCarsRef.value?.closeModal();
  } catch (err) {
    showError(`保存失败：${err instanceof Error ? err.message : String(err)}`);
  }
}

async function loadMonthlyCardLedgers() {
  try {
    const result = await listMonthlyCardLedgers({
      plate: appliedMonthlyCardLedgerFilters.value.plate || undefined,
      ownerName: appliedMonthlyCardLedgerFilters.value.ownerName || undefined,
      roomNumber: appliedMonthlyCardLedgerFilters.value.roomNumber || undefined,
      processDateStart: appliedMonthlyCardLedgerFilters.value.processDateStart || undefined,
      processDateEnd: appliedMonthlyCardLedgerFilters.value.processDateEnd || undefined,
      page: monthlyCardLedgerPage.value,
      perPage: monthlyCardLedgerPageSize.value,
    });
    monthlyCardLedgerRecords.value = result.items;
    monthlyCardLedgerTotal.value = result.total;
  } catch (err) {
    showError(`加载月卡台账失败：${err instanceof Error ? err.message : String(err)}`);
  }
}

async function handleMonthlyCardLedgerSearch(newFilters: MonthlyCardLedgerFilters) {
  appliedMonthlyCardLedgerFilters.value = { ...newFilters };
  monthlyCardLedgerFilters.value = { ...newFilters };
  monthlyCardLedgerPage.value = 1;
  await loadMonthlyCardLedgers();
}

async function handleMonthlyCardLedgerReset() {
  appliedMonthlyCardLedgerFilters.value = {
    plate: "",
    ownerName: "",
    roomNumber: "",
    processDateStart: "",
    processDateEnd: "",
  };
  monthlyCardLedgerFilters.value = { ...appliedMonthlyCardLedgerFilters.value };
  monthlyCardLedgerPage.value = 1;
  await loadMonthlyCardLedgers();
}

async function handleImportMonthlyCardLedger(filePath: string, data: Uint8Array) {
  loading.value = true;
  try {
    const fileName = filePath.split(/[\\/]/).pop() || filePath;
    const result = parseMonthlyCardLedgerXlsxFile(data, fileName);
    if (result.records.length === 0 && result.errors.length === 0) {
      showError("未在文件中找到有效的月卡台账记录。");
      return;
    }
    await saveMonthlyCardLedgers({ records: result.records, replace: true });
    showInfo(`成功导入 ${result.records.length.toLocaleString()} 条月卡台账记录。`);
    if (result.errors.length > 0) {
      showError(result.errors.join("；"));
    }
    monthlyCardLedgerPage.value = 1;
    appliedMonthlyCardLedgerFilters.value = {
      plate: "",
      ownerName: "",
      roomNumber: "",
      processDateStart: "",
      processDateEnd: "",
    };
    monthlyCardLedgerFilters.value = { ...appliedMonthlyCardLedgerFilters.value };
    await loadMonthlyCardLedgers();
    await updateLastImportTime();
  } catch (err) {
    showError(`导入失败：${err instanceof Error ? err.message : String(err)}`);
  } finally {
    loading.value = false;
  }
}

async function handleClearMonthlyCardLedgers() {
  try {
    const count = await clearMonthlyCardLedgers();
    showInfo(`已清空 ${count.toLocaleString()} 条月卡台账记录。`);
    monthlyCardLedgerPage.value = 1;
    await loadMonthlyCardLedgers();
  } catch (err) {
    showError(`清空失败：${err instanceof Error ? err.message : String(err)}`);
  }
}

async function handleSaveMonthlyRental(record: MonthlyRentalVehicle) {
  try {
    await updateMonthlyRental(record);
    showInfo("月租车记录已保存");
    await loadMonthlyRentals();
    monthlyRentalsRef.value?.closeModal();
  } catch (err) {
    showError(`保存失败：${err instanceof Error ? err.message : String(err)}`);
  }
}

async function handleFreeCarReset() {
  appliedFreeCarFilters.value = {
    plate: "",
    ownerName: "",
    vehicleType: "",
    startDateStart: "",
    startDateEnd: "",
    endDateStart: "",
    endDateEnd: "",
  };
  freeCarFilters.value = { ...appliedFreeCarFilters.value };
  freeCarPage.value = 1;
  await loadFreeCars();
}

async function handleCleanFreeCarAddresses() {
  cleaningAddresses.value = true;
  try {
    const count = await cleanFreeCarAddresses();
    showInfo(`已整理 ${count} 条免费车地址信息。`);
    await message(`已整理 ${count} 条免费车地址信息。`, { title: "整理完成", kind: "info" });
    await loadFreeCars();
  } catch (err) {
    const text = `地址整理失败：${err instanceof Error ? err.message : String(err)}`;
    showError(text);
    await message(text, { title: "整理失败", kind: "error" });
  } finally {
    cleaningAddresses.value = false;
  }
}

async function handleBuildingChargesSearch(filters: BuildingChargesFilters) {
  buildingChargesFilters.value = { ...filters };
  buildingChargesLoading.value = true;
  try {
    const items = await getBuildingCharges(filters);
    buildingChargesRef.value?.setItems(items);
  } catch (err) {
    showError(`加载楼栋收费数据失败：${err instanceof Error ? err.message : String(err)}`);
  } finally {
    buildingChargesLoading.value = false;
  }
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

async function handleFileLoad(data: Uint8Array, _path: string, type: "records" | "monthly" | "free") {
  loading.value = true;
  progress.value = 0;
  progressText.value = "正在解压并解析 ZIP 文件...";
  messageText.value = "";

  try {
    if (type === "monthly") {
      const result = await parseMonthlyRentalZipFile(data, (current, totalFiles, fileName) => {
        progress.value = totalFiles > 0 ? Math.round((current / totalFiles) * 50) : 0;
        progressText.value = `正在解析 ${fileName} (${current}/${totalFiles})`;
      });

      if (result.records.length === 0 && result.errors.length === 0) {
        showError("未在压缩包中找到有效的月租车 Excel 文件。");
        return;
      }

      progress.value = 55;
      progressText.value = `正在保存 ${result.records.length.toLocaleString()} 条月租车记录到数据库...`;
      await saveMonthlyRentals({ records: result.records, replace: true });

      progress.value = 75;
      progressText.value = "正在整理月租车地址信息...";
      const cleanedMonthly = await cleanMonthlyRentalAddresses();

      progress.value = 100;
      progressText.value = "处理完成";

      const info = `成功处理 ${result.records.length.toLocaleString()} 条月租车记录，整理地址 ${cleanedMonthly} 条。`;
      showInfo(info);
      await message(info, { title: "处理完成", kind: "info" });

      if (result.errors.length > 0) {
        showError(result.errors.join("；"));
      }

      monthlyPage.value = 1;
      appliedMonthlyFilters.value = {
        plate: "",
        ownerName: "",
        vehicleType: "",
        startDateStart: "",
        startDateEnd: "",
        endDateStart: "",
        endDateEnd: "",
      };
      monthlyFilters.value = { ...appliedMonthlyFilters.value };
      await loadMonthlyRentals();
    } else if (type === "free") {
      const result = await parseFreeCarZipFile(data, (current, totalFiles, fileName) => {
        progress.value = totalFiles > 0 ? Math.round((current / totalFiles) * 50) : 0;
        progressText.value = `正在解析 ${fileName} (${current}/${totalFiles})`;
      });

      if (result.records.length === 0 && result.errors.length === 0) {
        showError("未在压缩包中找到有效的免费车 Excel 文件。");
        return;
      }

      progress.value = 55;
      progressText.value = `正在保存 ${result.records.length.toLocaleString()} 条免费车记录到数据库...`;
      await saveFreeCars({ records: result.records, replace: true });

      progress.value = 75;
      progressText.value = "正在整理免费车地址信息...";
      const cleanedFree = await cleanFreeCarAddresses();

      progress.value = 100;
      progressText.value = "处理完成";

      const info = `成功处理 ${result.records.length.toLocaleString()} 条免费车记录，整理地址 ${cleanedFree} 条。`;
      showInfo(info);
      await message(info, { title: "处理完成", kind: "info" });

      if (result.errors.length > 0) {
        showError(result.errors.join("；"));
      }

      freeCarPage.value = 1;
      appliedFreeCarFilters.value = {
        plate: "",
        ownerName: "",
        vehicleType: "",
        startDateStart: "",
        startDateEnd: "",
        endDateStart: "",
        endDateEnd: "",
      };
      freeCarFilters.value = { ...appliedFreeCarFilters.value };
      await loadFreeCars();
    } else {
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
    }

    await updateLastImportTime();
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
  let confirmText = "确定要清空数据库中的所有停车记录吗？此操作不可恢复。";
  if (activeMenu.value === "monthly") {
    confirmText = "确定要清空数据库中的所有月租车记录吗？此操作不可恢复。";
  } else if (activeMenu.value === "free") {
    confirmText = "确定要清空数据库中的所有免费车记录吗？此操作不可恢复。";
  } else if (activeMenu.value === "monthlyCardLedger") {
    confirmText = "确定要清空数据库中的所有月卡台账记录吗？此操作不可恢复。";
  }
  const ok = await confirm(confirmText, {
    title: "请确认",
    kind: "warning",
  });
  if (!ok) return;
  loading.value = true;
  progressText.value = "正在清空数据库...";
  try {
    if (activeMenu.value === "monthly") {
      await clearMonthlyRentals();
      showInfo("月租车记录已清空。");
      monthlyPage.value = 1;
      appliedMonthlyFilters.value = {
        plate: "",
        ownerName: "",
        vehicleType: "",
        startDateStart: "",
        startDateEnd: "",
        endDateStart: "",
        endDateEnd: "",
      };
      monthlyFilters.value = { ...appliedMonthlyFilters.value };
      await loadMonthlyRentals();
    } else if (activeMenu.value === "free") {
      await clearFreeCars();
      showInfo("免费车记录已清空。");
      freeCarPage.value = 1;
      appliedFreeCarFilters.value = {
        plate: "",
        ownerName: "",
        vehicleType: "",
        startDateStart: "",
        startDateEnd: "",
        endDateStart: "",
        endDateEnd: "",
      };
      freeCarFilters.value = { ...appliedFreeCarFilters.value };
      await loadFreeCars();
    } else if (activeMenu.value === "monthlyCardLedger") {
      await clearMonthlyCardLedgers();
      showInfo("月卡台账记录已清空。");
      monthlyCardLedgerPage.value = 1;
      appliedMonthlyCardLedgerFilters.value = {
        plate: "",
        ownerName: "",
        roomNumber: "",
        processDateStart: "",
        processDateEnd: "",
      };
      monthlyCardLedgerFilters.value = { ...appliedMonthlyCardLedgerFilters.value };
      await loadMonthlyCardLedgers();
    } else {
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
    }
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

watch(monthlyPage, () => {
  loadMonthlyRentals();
});

watch(freeCarPage, () => {
  loadFreeCars();
});

watch(monthlyCardLedgerPage, () => {
  loadMonthlyCardLedgers();
});

watch(activeMenu, (value) => {
  if (value === "monthly" || value === "free" || value === "settings") {
    loadEditSettings();
  }
});

onMounted(async () => {
  try {
    dbPath.value = await getDbPath();
    dbReady.value = true;
    lastImportTime.value = (await getSetting("lastImportTime")) ?? "";
    await Promise.all([
      loadRecords(),
      loadMonthlyRentals(),
      loadFreeCars(),
      loadMonthlyCardLedgers(),
      loadEditSettings(),
    ]);
  } catch {
    dbReady.value = false;
  }
  currentTimeTimer = setInterval(() => {
    currentTime.value = formatCurrentDateTime(new Date());
  }, 1000);
});

onBeforeUnmount(() => {
  if (currentTimeTimer) {
    clearInterval(currentTimeTimer);
    currentTimeTimer = null;
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
        <p class="text-slate-400 text-xs">
          解析 ZIP 压缩包中的进出场记录表，支持查看与统计
        </p>
        <p v-if="dbPath" class="text-slate-500 text-xs">{{ dbPath }}</p>
      </div>
      <div class="flex-1 flex justify-center px-4">
        <div class="bg-amber-500/15 border border-amber-500/40 rounded-lg px-4 py-2 text-center">
          <div class="flex items-center justify-center gap-2 text-amber-300 text-sm font-medium">
            <span class="inline-block w-2 h-2 rounded-full bg-amber-400 animate-pulse"></span>
            <span>请使用最新数据</span>
          </div>
          <div class="text-amber-200 text-xs mt-1 space-x-3">
            <span>数据更新时间：{{ lastImportTime || "暂无导入记录" }}</span>
            <span>当前时间：{{ currentTime }}</span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <button
          @click="handleClear"
          :disabled="loading"
          class="px-4 py-2 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded-lg text-sm font-medium transition"
        >
          清空数据库
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

        <ImportProgress
          v-if="activeMenu === 'monthly' || activeMenu === 'free'"
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
            :loading="loading"
            @search="handleSearch"
            @reset="handleReset"
            @update:page-size="(v) => { pageSize = v; page = 1; loadRecords(); }"
            @import-zip="selectFile('records')"
          />
          <MonthlyRentals
            v-else-if="activeMenu === 'monthly'"
            ref="monthlyRentalsRef"
            v-model:page="monthlyPage"
            :page-size="monthlyPageSize"
            :filters="monthlyFilters"
            :records="monthlyRecords"
            :total="monthlyTotal"
            :loading="loading"
            :cleaning="cleaningAddresses"
            :editable="monthlyEditable"
            @search="handleMonthlySearch"
            @reset="handleMonthlyReset"
            @save="handleSaveMonthlyRental"
            @update:page-size="(v) => { monthlyPageSize = v; monthlyPage = 1; loadMonthlyRentals(); }"
            @clean-addresses="handleCleanMonthlyRentalAddresses"
            @import-zip="selectFile('monthly')"
          />
          <FreeCars
            v-else-if="activeMenu === 'free'"
            ref="freeCarsRef"
            v-model:page="freeCarPage"
            :page-size="freeCarPageSize"
            :filters="freeCarFilters"
            :records="freeCarRecords"
            :total="freeCarTotal"
            :loading="loading"
            :cleaning="cleaningAddresses"
            :editable="freeCarEditable"
            @search="handleFreeCarSearch"
            @reset="handleFreeCarReset"
            @save="handleSaveFreeCar"
            @update:page-size="(v) => { freeCarPageSize = v; freeCarPage = 1; loadFreeCars(); }"
            @clean-addresses="handleCleanFreeCarAddresses"
            @import-zip="selectFile('free')"
          />
          <MonthlyCardLedgerComponent
            v-else-if="activeMenu === 'monthlyCardLedger'"
            v-model:page="monthlyCardLedgerPage"
            :page-size="monthlyCardLedgerPageSize"
            :filters="monthlyCardLedgerFilters"
            :records="monthlyCardLedgerRecords"
            :total="monthlyCardLedgerTotal"
            :loading="loading"
            @search="handleMonthlyCardLedgerSearch"
            @reset="handleMonthlyCardLedgerReset"
            @import-file="handleImportMonthlyCardLedger"
            @clear="handleClearMonthlyCardLedgers"
            @update:page-size="(v) => { monthlyCardLedgerPageSize = v; monthlyCardLedgerPage = 1; loadMonthlyCardLedgers(); }"
          />
          <BuildingCharges
            v-else-if="activeMenu === 'building'"
            ref="buildingChargesRef"
            :loading="buildingChargesLoading"
            @search="handleBuildingChargesSearch"
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
