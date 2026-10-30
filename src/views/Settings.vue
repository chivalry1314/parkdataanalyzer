<script setup lang="ts">
import { onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import { getDbPath, setDbPath } from "../api";

const dbPath = ref("");
const migrateData = ref(true);
const dbStatus = ref("");

onMounted(async () => {
  try {
    dbPath.value = await getDbPath();
  } catch (err) {
    dbPath.value = `获取路径失败：${err instanceof Error ? err.message : String(err)}`;
  }
});

const changeDbPath = async () => {
  try {
    const path = await save({
      title: "选择新的数据库存放位置",
      filters: [{ name: "SQLite", extensions: ["db"] }],
      defaultPath: "parkingsyslog.db",
    });
    if (!path) return;

    dbStatus.value = "正在切换数据库路径...";
    await setDbPath(path as string, migrateData.value);
    dbStatus.value = "数据库路径已切换，即将自动重启...";
    setTimeout(() => relaunch(), 800);
  } catch (err) {
    dbStatus.value = `切换失败：${err instanceof Error ? err.message : String(err)}`;
  }
};
</script>

<template>
  <div class="max-w-3xl space-y-6">
    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">数据存储位置</h3>
      <p class="text-sm text-slate-500 mb-4">
        数据库文件当前路径如下。你可以直接复制该文件进行备份，也可以修改存储位置。
        修改路径时可以选择是否迁移现有数据；若不迁移，则在新位置创建一个空数据库。
      </p>
      <div class="bg-slate-100 rounded-lg px-4 py-3 text-sm text-slate-700 break-all font-mono mb-4">
        {{ dbPath }}
      </div>
      <div class="flex flex-wrap items-center gap-4">
        <label class="flex items-center gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            v-model="migrateData"
            class="w-4 h-4"
          />
          迁移现有数据到新路径
        </label>
        <button
          @click="changeDbPath"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          修改数据库路径
        </button>
      </div>
      <div v-if="dbStatus" class="mt-3 text-sm" :class="dbStatus.startsWith('切换失败') ? 'text-red-600' : 'text-green-600'">
        {{ dbStatus }}
      </div>
    </div>
  </div>
</template>
