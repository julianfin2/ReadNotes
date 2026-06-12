<script setup lang="ts">
import { open, save } from "@tauri-apps/plugin-dialog";
import type { DatabaseInfo } from "../types/database";

const props = defineProps<{
  databaseInfo: DatabaseInfo;
  isSaving: boolean;
}>();

const emit = defineEmits<{
  createDatabase: [path: string];
  switchDatabase: [path: string];
  useDefaultDatabase: [];
}>();

const databaseFilters = [
  {
    name: "SQLite 数据库",
    extensions: ["sqlite", "sqlite3", "db"],
  },
];

async function chooseNewDatabase() {
  const path = await save({
    title: "新建数据库",
    defaultPath: props.databaseInfo.defaultPath,
    filters: databaseFilters,
  });

  if (path) {
    emit("createDatabase", path);
  }
}

async function chooseExistingDatabase() {
  const path = await open({
    title: "切换数据库",
    multiple: false,
    filters: databaseFilters,
  });

  if (typeof path === "string") {
    emit("switchDatabase", path);
  }
}
</script>

<template>
  <section class="page-panel workspace-panel desktop-view settings-page">
    <header class="page-header list-toolbar-header">
      <div class="page-title-block">
        <h2>设置</h2>
        <p class="subtle-text">管理当前使用的数据库文件</p>
      </div>
    </header>

    <section class="settings-card database-settings-card">
      <label>
        当前数据库路径
        <input :value="databaseInfo.currentPath" readonly />
      </label>

      <div class="settings-actions database-actions">
        <button
          class="primary-action"
          :disabled="isSaving"
          type="button"
          @click="chooseNewDatabase"
        >
          新建
        </button>
        <button
          class="secondary-action"
          :disabled="isSaving"
          type="button"
          @click="chooseExistingDatabase"
        >
          切换
        </button>
        <button
          class="secondary-action"
          :disabled="isSaving || databaseInfo.usingDefault"
          type="button"
          @click="emit('useDefaultDatabase')"
        >
          默认
        </button>
      </div>
    </section>
  </section>
</template>
