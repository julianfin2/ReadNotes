<script setup lang="ts">
import { open, save } from "@tauri-apps/plugin-dialog";
import { Database, FolderOpen, RotateCcw } from "@lucide/vue";
import type { DatabaseInfo, DatabaseStartupIssue } from "../types/database";

const props = defineProps<{
  databaseInfo: DatabaseInfo;
  issue: DatabaseStartupIssue;
  isSaving: boolean;
  errorMessage: string;
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
  <Teleport to="body">
    <div class="database-recovery-backdrop">
      <section class="database-recovery-panel" role="dialog" aria-modal="true" aria-label="数据库不可用">
        <header>
          <h2>数据库不可用</h2>
          <p>启动时保存的数据库路径无法使用。请选择一个处理方式后继续。</p>
        </header>

        <div class="database-recovery-detail">
          <span>保存路径</span>
          <strong>{{ issue.configuredPath }}</strong>
          <span>原因</span>
          <strong>{{ issue.reason }}</strong>
        </div>

        <p v-if="errorMessage" class="database-recovery-error">{{ errorMessage }}</p>

        <div class="database-recovery-actions">
          <button class="primary-action" :disabled="isSaving" type="button" @click="chooseNewDatabase">
            <Database aria-hidden="true" />
            新建
          </button>
          <button class="secondary-action" :disabled="isSaving" type="button" @click="chooseExistingDatabase">
            <FolderOpen aria-hidden="true" />
            切换
          </button>
          <button class="secondary-action" :disabled="isSaving" type="button" @click="emit('useDefaultDatabase')">
            <RotateCcw aria-hidden="true" />
            默认
          </button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
