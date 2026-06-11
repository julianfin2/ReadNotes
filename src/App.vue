<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

type Excerpt = {
  id: string;
  quote: string;
  reflection?: string | null;
  sourceWorkId?: string | null;
  location?: string | null;
  importance: number;
  status: "inbox" | "processed" | "archived";
  createdAt: string;
  updatedAt: string;
};

const excerpts = ref<Excerpt[]>([]);
const quote = ref("");
const reflection = ref("");
const location = ref("");
const importance = ref(3);
const databasePath = ref("");
const errorMessage = ref("");
const isSaving = ref(false);

onMounted(async () => {
  await Promise.all([loadDatabasePath(), loadExcerpts()]);
});

async function loadDatabasePath() {
  databasePath.value = await invoke<string>("get_database_path");
}

async function loadExcerpts() {
  excerpts.value = await invoke<Excerpt[]>("list_excerpts");
}

async function createExcerpt() {
  errorMessage.value = "";
  isSaving.value = true;

  try {
    await invoke<Excerpt>("create_excerpt", {
      input: {
        quote: quote.value,
        reflection: reflection.value,
        location: location.value,
        importance: importance.value,
      },
    });

    quote.value = "";
    reflection.value = "";
    location.value = "";
    importance.value = 3;
    await loadExcerpts();
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <main class="app-shell">
    <aside class="sidebar">
      <div>
        <p class="eyebrow">ReadNotes</p>
        <h1>摘抄库</h1>
      </div>

      <nav>
        <button class="nav-item active">摘抄</button>
        <button class="nav-item" disabled>主题</button>
        <button class="nav-item" disabled>标签</button>
        <button class="nav-item" disabled>时间线</button>
      </nav>
    </aside>

    <section class="capture-panel">
      <div class="section-heading">
        <p class="eyebrow">Quick capture</p>
        <h2>新增摘抄</h2>
      </div>

      <form @submit.prevent="createExcerpt">
        <label>
          原文
          <textarea v-model="quote" rows="7" placeholder="输入摘抄原文" />
        </label>

        <label>
          初始理解
          <textarea v-model="reflection" rows="5" placeholder="写下此刻的理解" />
        </label>

        <div class="field-row">
          <label>
            位置
            <input v-model="location" placeholder="页码、章节，可选" />
          </label>

          <label>
            重要性
            <input v-model.number="importance" max="5" min="1" type="number" />
          </label>
        </div>

        <button class="primary-action" :disabled="isSaving" type="submit">
          {{ isSaving ? "保存中" : "保存摘抄" }}
        </button>

        <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
      </form>

      <p class="database-path">Database: {{ databasePath }}</p>
    </section>

    <section class="library-panel">
      <div class="section-heading">
        <p class="eyebrow">Library</p>
        <h2>{{ excerpts.length }} 条摘抄</h2>
      </div>

      <div class="excerpt-list">
        <article v-for="excerpt in excerpts" :key="excerpt.id" class="excerpt-card">
          <blockquote>{{ excerpt.quote }}</blockquote>
          <p v-if="excerpt.reflection" class="reflection">{{ excerpt.reflection }}</p>
          <footer>
            <span>重要性 {{ excerpt.importance }}</span>
            <span>{{ excerpt.status }}</span>
            <span>{{ new Date(excerpt.createdAt).toLocaleString() }}</span>
          </footer>
        </article>

        <p v-if="excerpts.length === 0" class="empty-state">还没有摘抄。</p>
      </div>
    </section>
  </main>
</template>

<style>
:root {
  color: #1d2528;
  background: #f4f1ea;
  font-family:
    Inter, "Microsoft YaHei", "PingFang SC", system-ui, -apple-system, sans-serif;
  font-size: 16px;
  font-synthesis: none;
  line-height: 1.5;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
}

button,
input,
textarea {
  font: inherit;
}

button {
  border: 0;
}

.app-shell {
  display: grid;
  min-height: 100vh;
  grid-template-columns: 220px minmax(320px, 440px) minmax(0, 1fr);
}

.sidebar {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding: 28px 20px;
  border-right: 1px solid #d9d3c7;
  background: #253238;
  color: #f9f5ed;
}

.eyebrow {
  margin: 0 0 6px;
  color: #8a6552;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
}

.sidebar .eyebrow {
  color: #d6b39d;
}

h1,
h2 {
  margin: 0;
  letter-spacing: 0;
}

h1 {
  font-size: 1.65rem;
}

h2 {
  font-size: 1.25rem;
}

nav {
  display: grid;
  gap: 6px;
}

.nav-item {
  min-height: 38px;
  padding: 0 12px;
  border-radius: 6px;
  background: transparent;
  color: #cfd8d6;
  text-align: left;
}

.nav-item.active {
  background: #e0c3a8;
  color: #1d2528;
}

.nav-item:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.capture-panel,
.library-panel {
  padding: 28px;
}

.capture-panel {
  border-right: 1px solid #d9d3c7;
  background: #fbf8f1;
}

.section-heading {
  margin-bottom: 20px;
}

form,
.excerpt-list {
  display: grid;
  gap: 16px;
}

label {
  display: grid;
  gap: 7px;
  color: #405055;
  font-size: 0.9rem;
  font-weight: 700;
}

textarea,
input {
  width: 100%;
  border: 1px solid #d6cfc2;
  border-radius: 6px;
  background: #fffdf9;
  color: #1d2528;
  outline: none;
}

textarea {
  resize: vertical;
  padding: 10px 12px;
}

input {
  min-height: 40px;
  padding: 0 12px;
}

textarea:focus,
input:focus {
  border-color: #8a6552;
}

.field-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 96px;
  gap: 12px;
}

.primary-action {
  min-height: 42px;
  border-radius: 6px;
  background: #2e6f62;
  color: white;
  cursor: pointer;
  font-weight: 700;
}

.primary-action:disabled {
  cursor: wait;
  opacity: 0.7;
}

.error-message {
  margin: 0;
  color: #a23b32;
}

.database-path {
  margin: 24px 0 0;
  overflow-wrap: anywhere;
  color: #6e7678;
  font-size: 0.78rem;
}

.library-panel {
  background: #f4f1ea;
}

.excerpt-card {
  display: grid;
  gap: 12px;
  padding: 18px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

blockquote {
  margin: 0;
  padding-left: 14px;
  border-left: 3px solid #2e6f62;
  color: #1d2528;
}

.reflection {
  margin: 0;
  color: #49585d;
}

footer {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  color: #7a817f;
  font-size: 0.82rem;
}

.empty-state {
  margin: 0;
  color: #6e7678;
}

@media (max-width: 900px) {
  .app-shell {
    grid-template-columns: 1fr;
  }

  .sidebar {
    border-right: 0;
  }

  .capture-panel {
    border-right: 0;
    border-bottom: 1px solid #d9d3c7;
  }
}
</style>
