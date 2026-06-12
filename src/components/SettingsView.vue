<script setup lang="ts">
import { ref, watch } from "vue";
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

const createPath = ref("");
const switchPath = ref("");

watch(
  () => props.databaseInfo.currentPath,
  (path) => {
    switchPath.value = path;
  },
  { immediate: true },
);

function submitCreate() {
  const path = createPath.value.trim();
  if (!path) {
    return;
  }

  emit("createDatabase", path);
}

function submitSwitch() {
  const path = switchPath.value.trim();
  if (!path || path === props.databaseInfo.currentPath) {
    return;
  }

  emit("switchDatabase", path);
}
</script>

<template>
  <section class="page-panel workspace-panel desktop-view settings-page">
    <header class="page-header list-toolbar-header">
      <div class="page-title-block">
        <h2>设置</h2>
        <p class="subtle-text">管理数据库文件位置</p>
      </div>
    </header>

    <div class="settings-layout">
      <section class="settings-card">
        <div class="card-header">
          <div>
            <h3>当前数据库</h3>
            <p class="subtle-text">
              {{ databaseInfo.usingDefault ? "正在使用默认数据库" : "正在使用自定义数据库" }}
            </p>
          </div>
          <button
            class="secondary-action"
            :disabled="isSaving || databaseInfo.usingDefault"
            type="button"
            @click="emit('useDefaultDatabase')"
          >
            回到默认
          </button>
        </div>

        <label>
          当前路径
          <input :value="databaseInfo.currentPath" readonly />
        </label>
        <label>
          默认路径
          <input :value="databaseInfo.defaultPath" readonly />
        </label>
      </section>

      <section class="settings-card">
        <div>
          <h3>创建新数据库</h3>
          <p class="subtle-text">输入一个还不存在的 SQLite 数据库文件绝对路径。</p>
        </div>

        <form class="settings-form" @submit.prevent="submitCreate">
          <label>
            新数据库路径
            <input
              v-model="createPath"
              placeholder="例如：D:\\ReadNotes\\my-notes.sqlite"
            />
          </label>
          <div class="settings-actions">
            <button
              class="primary-action"
              :disabled="isSaving || !createPath.trim()"
              type="submit"
            >
              创建并切换
            </button>
          </div>
        </form>
      </section>

      <section class="settings-card">
        <div>
          <h3>切换数据库</h3>
          <p class="subtle-text">输入一个已经存在的数据库文件绝对路径。</p>
        </div>

        <form class="settings-form" @submit.prevent="submitSwitch">
          <label>
            数据库路径
            <input
              v-model="switchPath"
              placeholder="例如：D:\\ReadNotes\\archive.sqlite"
            />
          </label>
          <div class="settings-actions">
            <button
              class="primary-action"
              :disabled="
                isSaving ||
                !switchPath.trim() ||
                switchPath.trim() === databaseInfo.currentPath
              "
              type="submit"
            >
              切换数据库
            </button>
          </div>
        </form>
      </section>
    </div>
  </section>
</template>
