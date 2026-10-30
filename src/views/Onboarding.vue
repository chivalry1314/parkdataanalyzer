<script setup lang="ts">
import { ref } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import { getDefaultDbPath, setDbPath } from "../api";

const status = ref("");
const loading = ref(false);

const finishWith = async (path: string, migrate: boolean) => {
  loading.value = true;
  try {
    status.value = "正在初始化数据库...";
    await setDbPath(path, migrate);
    status.value = "初始化完成，即将重启应用...";
    setTimeout(() => relaunch(), 800);
  } catch (err) {
    status.value = `初始化失败：${err instanceof Error ? err.message : String(err)}`;
    loading.value = false;
  }
};

const chooseNewLocation = async () => {
  try {
    const path = await save({
      title: "选择数据库存放位置",
      filters: [{ name: "SQLite", extensions: ["db"] }],
      defaultPath: "parkingsyslog.db",
    });
    if (!path) return;
    await finishWith(path as string, false);
  } catch (err) {
    status.value = `选择失败：${err instanceof Error ? err.message : String(err)}`;
  }
};

const chooseExistingDb = async () => {
  try {
    const path = await open({
      title: "选择已有的数据库文件",
      filters: [{ name: "SQLite", extensions: ["db"] }],
      multiple: false,
    });
    if (!path) return;
    await finishWith(path as string, false);
  } catch (err) {
    status.value = `选择失败：${err instanceof Error ? err.message : String(err)}`;
  }
};

const useDefaultLocation = async () => {
  try {
    const path = await getDefaultDbPath();
    await finishWith(path, false);
  } catch (err) {
    status.value = `使用默认路径失败：${err instanceof Error ? err.message : String(err)}`;
  }
};
</script>

<template>
  <div class="h-screen flex items-center justify-center bg-slate-100">
    <div class="bg-white rounded-xl shadow-sm border p-8 max-w-md w-full mx-4">
      <h1 class="text-xl font-bold text-slate-800">欢迎使用停车收费记录解析工具</h1>
      <p class="text-sm text-slate-500 mt-2">
        首次使用需要选择数据库文件的存放位置。所有停车收费记录都会保存在这个文件中。
      </p>
      <ul class="list-disc list-inside text-sm text-slate-500 mt-3 space-y-1">
        <li>建议选择便于备份的目录，例如 D 盘或文档目录。</li>
        <li>请勿选择系统目录（如 Program Files），可能没有写入权限。</li>
        <li>之后也可以在“设置”页面修改存放位置。</li>
      </ul>

      <div class="mt-6 space-y-3">
        <button
          @click="chooseNewLocation"
          :disabled="loading"
          class="w-full px-5 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-60"
        >
          选择存放位置（新建数据库）
        </button>
        <button
          @click="chooseExistingDb"
          :disabled="loading"
          class="w-full px-5 py-2.5 bg-white border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50 transition disabled:opacity-60"
        >
          选择已有的数据库文件
        </button>
        <button
          @click="useDefaultLocation"
          :disabled="loading"
          class="w-full px-5 py-2 text-sm text-slate-500 hover:text-slate-700 transition disabled:opacity-60"
        >
          使用默认位置
        </button>
      </div>

      <p v-if="status" class="mt-4 text-sm" :class="status.startsWith('初始化失败') ? 'text-red-600' : 'text-green-600'">
        {{ status }}
      </p>
    </div>
  </div>
</template>
