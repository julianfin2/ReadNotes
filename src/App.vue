<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AppSidebar from "./components/AppSidebar.vue";
import ExcerptCapture from "./components/ExcerptCapture.vue";
import ExcerptList from "./components/ExcerptList.vue";
import TopicWorkspace from "./components/TopicWorkspace.vue";
import type { Excerpt, ExcerptFilters } from "./types/excerpt";
import type { Tag } from "./types/tag";

type ViewKey = "excerpts" | "topics" | "tags" | "timeline";

const activeView = ref<ViewKey>("excerpts");
const excerpts = ref<Excerpt[]>([]);
const tags = ref<Tag[]>([]);
const databasePath = ref("");
const errorMessage = ref("");
const isSaving = ref(false);

onMounted(async () => {
  await Promise.all([loadDatabasePath(), loadExcerpts(), loadTags()]);
});

async function loadDatabasePath() {
  databasePath.value = await invoke<string>("get_database_path");
}

async function loadExcerpts(filters?: ExcerptFilters) {
  excerpts.value = await invoke<Excerpt[]>("list_excerpts", {
    input: filters ? toExcerptQuery(filters) : null,
  });
}

async function loadTags() {
  tags.value = await invoke<Tag[]>("list_tags");
}

async function createExcerpt(input: {
  quote: string;
  reflection: string;
  location: string;
  importance: number;
  tagNames: string[];
}) {
  errorMessage.value = "";
  isSaving.value = true;

  try {
    await invoke<Excerpt>("create_excerpt", { input });
    await Promise.all([loadExcerpts(), loadTags()]);
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    isSaving.value = false;
  }
}

function toExcerptQuery(filters: ExcerptFilters) {
  return {
    search: filters.search || null,
    tagName: filters.tagName || null,
    status: filters.status || null,
    minImportance: filters.minImportance || null,
    sortBy: filters.sortBy,
    sortDirection: filters.sortDirection,
  };
}
</script>

<template>
  <main class="app-shell">
    <AppSidebar :active-view="activeView" @select-view="activeView = $event" />

    <template v-if="activeView === 'excerpts'">
      <ExcerptCapture
        :database-path="databasePath"
        :error-message="errorMessage"
        :is-saving="isSaving"
        @create-excerpt="createExcerpt"
      />
      <ExcerptList :excerpts="excerpts" :tags="tags" @apply-filters="loadExcerpts" />
    </template>

    <TopicWorkspace v-else-if="activeView === 'topics'" :excerpts="excerpts" />

    <section v-else class="workspace-panel">
      <p class="empty-state">这个视图还没有实现。</p>
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
select,
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
h2,
h3 {
  margin: 0;
  letter-spacing: 0;
}

h1 {
  font-size: 1.65rem;
}

h2 {
  font-size: 1.25rem;
}

h3 {
  font-size: 1rem;
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
.library-panel,
.topic-panel,
.workspace-panel {
  padding: 28px;
}

.capture-panel {
  border-right: 1px solid #d9d3c7;
  background: #fbf8f1;
}

.library-panel,
.topic-panel,
.workspace-panel {
  background: #f4f1ea;
}

.topic-panel {
  border-right: 1px solid #d9d3c7;
  background: #fbf8f1;
}

.section-heading {
  margin-bottom: 20px;
}

form,
.excerpt-list,
.stack {
  display: grid;
  gap: 16px;
}

.filter-bar {
  display: grid;
  grid-template-columns: minmax(180px, 1.4fr) repeat(5, minmax(110px, 1fr));
  gap: 12px;
  align-items: end;
  margin-bottom: 18px;
}

.filter-bar .primary-action,
.filter-bar .secondary-action {
  min-height: 40px;
}

label {
  display: grid;
  gap: 7px;
  color: #405055;
  font-size: 0.9rem;
  font-weight: 700;
}

textarea,
input,
select {
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

input,
select {
  min-height: 40px;
  padding: 0 12px;
}

textarea:focus,
input:focus,
select:focus {
  border-color: #8a6552;
}

.field-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 96px;
  gap: 12px;
}

.primary-action,
.secondary-action {
  min-height: 42px;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 700;
}

.primary-action {
  background: #2e6f62;
  color: white;
}

.secondary-action {
  border: 1px solid #d6cfc2;
  background: #fffdf9;
  color: #2d3a3f;
}

.primary-action:disabled,
.secondary-action:disabled {
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

.excerpt-card,
.topic-card {
  display: grid;
  gap: 12px;
  padding: 18px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

.topic-list,
.node-list {
  display: grid;
  gap: 8px;
  margin-top: 18px;
}

.topic-selector,
.node-selector {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  min-height: 40px;
  padding: 8px 10px;
  border: 1px solid #d6cfc2;
  border-radius: 6px;
  background: #fffdf9;
  color: #2d3a3f;
  cursor: pointer;
  text-align: left;
}

.topic-selector.active,
.node-selector.active {
  border-color: #2e6f62;
  background: #e8eee6;
}

.topic-selector small {
  color: #7a817f;
}

.topic-workspace-grid {
  display: grid;
  grid-template-columns: minmax(260px, 340px) minmax(0, 1fr);
  gap: 18px;
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

.tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag-pill {
  max-width: 100%;
  padding: 3px 8px;
  border-radius: 999px;
  background: #e8eee6;
  color: #2e6f62;
  font-size: 0.78rem;
  font-weight: 700;
  overflow-wrap: anywhere;
}

footer,
.meta-row {
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

  .topic-panel {
    border-right: 0;
    border-bottom: 1px solid #d9d3c7;
  }

  .topic-workspace-grid {
    grid-template-columns: 1fr;
  }

  .filter-bar {
    grid-template-columns: 1fr;
  }
}
</style>
