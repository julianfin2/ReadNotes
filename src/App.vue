<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AppSidebar from "./components/AppSidebar.vue";
import ExcerptList from "./components/ExcerptList.vue";
import ManagementView from "./components/ManagementView.vue";
import TopicWorkspace from "./components/TopicWorkspace.vue";
import type { Book } from "./types/book";
import type { Excerpt, ExcerptFilters, UpdateExcerptInput } from "./types/excerpt";
import type { Tag } from "./types/tag";

type ViewKey = "excerpts" | "topics" | "management";

const activeView = ref<ViewKey>("excerpts");
const excerpts = ref<Excerpt[]>([]);
const tags = ref<Tag[]>([]);
const books = ref<Book[]>([]);
const databasePath = ref("");
const errorMessage = ref("");
const isSaving = ref(false);
const lastFilters = ref<ExcerptFilters | undefined>();

onMounted(async () => {
  await Promise.all([loadDatabasePath(), loadExcerpts(), loadTags(), loadBooks()]);
});

async function loadDatabasePath() {
  databasePath.value = await invoke<string>("get_database_path");
}

async function loadExcerpts(filters?: ExcerptFilters) {
  lastFilters.value = filters;
  excerpts.value = await invoke<Excerpt[]>("list_excerpts", {
    input: filters ? toExcerptQuery(filters) : null,
  });
}

async function updateExcerpt(input: UpdateExcerptInput) {
  errorMessage.value = "";
  isSaving.value = true;

  try {
    await invoke<Excerpt>("update_excerpt", { input });
    await Promise.all([loadExcerpts(lastFilters.value), loadTags(), loadBooks()]);
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    isSaving.value = false;
  }
}

async function deleteExcerpt(id: string) {
  errorMessage.value = "";

  try {
    await invoke("delete_excerpt", { id });
    await Promise.all([loadExcerpts(lastFilters.value), loadTags()]);
  } catch (error) {
    errorMessage.value = String(error);
  }
}

async function loadTags() {
  tags.value = await invoke<Tag[]>("list_tags");
}

async function loadBooks() {
  books.value = await invoke<Book[]>("list_books");
}

async function createExcerpt(input: {
  quote: string;
  reflection: string;
  bookTitle: string;
  chapterTitle: string;
  tagNames: string[];
}) {
  errorMessage.value = "";
  isSaving.value = true;

  try {
    await invoke<Excerpt>("create_excerpt", { input });
    await Promise.all([loadExcerpts(), loadTags(), loadBooks()]);
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
    sortBy: filters.sortBy,
    sortDirection: filters.sortDirection,
  };
}
</script>

<template>
  <main class="app-shell">
    <AppSidebar :active-view="activeView" @select-view="activeView = $event" />

    <section class="app-main-frame">
      <template v-if="activeView === 'excerpts'">
      <p v-if="errorMessage" class="app-error">{{ errorMessage }}</p>
      <ExcerptList
        :excerpts="excerpts"
        :is-saving="isSaving"
        :books="books"
        :tags="tags"
        @apply-filters="loadExcerpts"
        @create-excerpt="createExcerpt"
        @delete-excerpt="deleteExcerpt"
        @update-excerpt="updateExcerpt"
      />
      </template>

      <TopicWorkspace v-else-if="activeView === 'topics'" :excerpts="excerpts" />

      <ManagementView v-else-if="activeView === 'management'" @books-changed="loadBooks" />

      <section v-else class="workspace-panel">
        <p class="empty-state">这个视图还没有实现。</p>
      </section>
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
  overflow: hidden;
}

* {
  scrollbar-color: rgba(37, 50, 56, 0.42) transparent;
  scrollbar-width: thin;
}

*::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

*::-webkit-scrollbar-track {
  background: transparent;
}

*::-webkit-scrollbar-thumb {
  border: 3px solid transparent;
  border-radius: 999px;
  background: rgba(37, 50, 56, 0.42);
  background-clip: content-box;
}

*::-webkit-scrollbar-thumb:hover {
  background: rgba(37, 50, 56, 0.58);
  background-clip: content-box;
}

*::-webkit-scrollbar-button,
*::-webkit-scrollbar-corner {
  display: none;
  width: 0;
  height: 0;
  background: transparent;
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
  height: 100vh;
  min-height: 0;
  grid-template-columns: 232px minmax(0, 1fr);
  overflow: hidden;
}

.app-main-frame {
  display: grid;
  grid-template-columns: minmax(260px, 300px) minmax(0, 1fr);
  gap: 28px;
  min-width: 0;
  min-height: 0;
  padding: 28px;
  overflow: hidden;
  background: #f4f1ea;
}

.sidebar {
  display: flex;
  flex-direction: column;
  gap: 32px;
  padding: 28px 20px;
  border-right: 1px solid #d9d3c7;
  background: #253238;
  color: #f9f5ed;
  overflow: hidden;
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

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.sidebar-brand-icon {
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  border-radius: 8px;
}

.sidebar-version {
  margin: auto 0 0;
  color: rgba(249, 245, 237, 0.58);
  font-size: 0.78rem;
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

.management-page > .tag-page,
.management-page > .book-manager {
  flex: 1 1 auto;
  min-height: 0;
}

.management-tabs {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fbf8f1;
}

.management-tab {
  min-height: 34px;
  padding: 0 14px;
  border-radius: 999px;
  background: transparent;
  color: #2d3a3f;
  cursor: pointer;
  font-weight: 700;
}

.management-tab.active {
  background: #e8eee6;
  color: #2e6f62;
}

.library-panel,
.topic-panel,
.workspace-panel {
  padding: 0;
  min-width: 0;
  min-height: 0;
}

.library-panel,
.topic-panel,
.workspace-panel {
  background: #f4f1ea;
}

.page-panel {
  grid-column: 1 / -1;
  min-width: 0;
}

.desktop-view {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.desktop-view > .split-workspace,
.desktop-view > .topic-workspace-grid,
.desktop-view > .excerpt-list,
.desktop-view > .table-page,
.desktop-view > .editor-page,
.desktop-view > .reader-page,
.desktop-view > .topic-workspace {
  flex: 1 1 auto;
  min-height: 0;
}

.desktop-side-pane {
  display: flex;
  flex-direction: column;
  align-self: start;
  max-height: 100%;
  padding: 16px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fbf8f1;
  overflow: hidden;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 22px;
}

.list-toolbar-header {
  align-items: center;
  margin-bottom: 18px;
}

.page-title-block {
  min-width: 0;
}

.desktop-toolbar {
  margin-bottom: 16px;
}

.toolbar,
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.topic-toolbar {
  align-items: center;
}

.list-toolbar {
  align-items: center;
  flex: 1 1 auto;
}

.toolbar-search {
  width: min(420px, 42vw);
}

.custom-select {
  position: relative;
  flex: 0 0 auto;
}

.custom-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  min-height: 40px;
  padding: 0 12px 0 14px;
  border: 1px solid #d6cfc2;
  border-radius: 6px;
  background: #fffdf9;
  color: #1d2528;
  cursor: pointer;
  font-size: 0.92rem;
  font-weight: 500;
  text-align: left;
}

.custom-select-trigger:hover,
.custom-select-trigger:focus-visible {
  border-color: #8a6552;
  outline: none;
}

.custom-select-trigger span:first-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-select-caret {
  width: 8px;
  height: 8px;
  border-right: 2px solid #6e4e40;
  border-bottom: 2px solid #6e4e40;
  transform: translateY(-2px) rotate(45deg);
}

.custom-select-menu {
  position: fixed;
  z-index: 30;
  display: grid;
  gap: 4px;
  padding: 6px;
  border: 1px solid #d6cfc2;
  border-radius: 8px;
  background: #fffdf9;
  box-shadow: 0 12px 28px rgba(38, 35, 30, 0.16);
  overflow: auto;
}

.custom-select-option {
  min-height: 32px;
  padding: 0 10px;
  border-radius: 6px;
  background: transparent;
  color: #1d2528;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  overflow: hidden;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-select-option:hover,
.custom-select-option.active {
  background: #e8eee6;
  color: #2e6f62;
}

.editable-combobox {
  position: relative;
}

.editable-combobox-menu {
  position: fixed;
  z-index: 40;
  display: grid;
  gap: 4px;
  padding: 6px;
  border: 1px solid #d6cfc2;
  border-radius: 8px;
  background: #fffdf9;
  box-shadow: 0 12px 28px rgba(38, 35, 30, 0.16);
  overflow: auto;
}

.editable-combobox-option {
  min-height: 32px;
  padding: 0 10px;
  border-radius: 6px;
  background: transparent;
  color: #1d2528;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  text-align: left;
}

.editable-combobox-option:hover {
  background: #e8eee6;
  color: #2e6f62;
}

.filter-chip-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin: -6px 0 14px;
}

.filter-chip {
  max-width: 260px;
  padding: 3px 8px;
  border-radius: 999px;
  background: #e8eee6;
  color: #2e6f62;
  font-size: 0.78rem;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.text-action {
  min-height: 26px;
  padding: 0;
  background: transparent;
  color: #6e4e40;
  cursor: pointer;
  font-size: 0.82rem;
  font-weight: 700;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.inline-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.subtle-text {
  margin: 4px 0 0;
  color: #6e7678;
  font-size: 0.9rem;
}

.section-heading {
  margin-bottom: 20px;
}

form,
.excerpt-list,
.stack {
  display: grid;
  gap: 16px;
  align-content: start;
  min-height: 0;
}

.split-workspace {
  display: grid;
  grid-template-columns: minmax(280px, 340px) minmax(0, 1fr);
  gap: 16px;
  min-height: 0;
  overflow: hidden;
}

.library-workbench .split-workspace {
  grid-template-columns: minmax(300px, 340px) minmax(520px, 1fr);
}

.list-pane,
.detail-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
  overflow: hidden;
}

.list-scroll,
.detail-scroll {
  display: grid;
  align-content: start;
  grid-auto-rows: max-content;
  gap: 8px;
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
  padding: 12px;
}

.detail-scroll {
  gap: 14px;
  padding: 20px;
}

.excerpt-detail-pane {
  border-color: transparent;
  background: transparent;
}

.excerpt-detail-pane .detail-scroll {
  padding: 0 0 0 2px;
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 18px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

.reading-body {
  display: grid;
  gap: 16px;
  width: 100%;
  max-width: none;
  padding: 20px 22px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

.empty-detail {
  display: grid;
  place-items: center;
}

.excerpt-list-item {
  display: grid;
  flex: 0 0 auto;
  gap: 5px;
  width: 100%;
  min-height: 72px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: #1d2528;
  cursor: pointer;
  text-align: left;
}

.excerpt-list-item:hover,
.excerpt-list-item.active {
  border-color: #bfd0c8;
  background: #e8eee6;
}

.table-page {
  min-height: 0;
  overflow: hidden;
}

.tag-table-page {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}

.excerpt-table,
.topic-table,
.tag-table {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
  overflow: hidden;
}

.excerpt-table-body,
.topic-table-body,
.tag-table-body {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
}

.excerpt-table-head,
.excerpt-table-row,
.topic-table-head,
.topic-table-row,
.tag-table-head,
.tag-table-row {
  display: grid;
  gap: 14px;
  align-items: center;
}

.excerpt-table-head,
.topic-table-head,
.tag-table-head {
  z-index: 1;
  min-height: 42px;
  padding: 0 16px;
  border-bottom: 1px solid #ded7ca;
  background: #fbf8f1;
  color: #6e7678;
  font-size: 0.78rem;
  font-weight: 700;
}

.excerpt-table-head,
.excerpt-table-row {
  grid-template-columns: minmax(300px, 2fr) minmax(190px, 1fr) minmax(150px, 0.62fr) 120px 150px;
}

.topic-table-head,
.topic-table-row {
  grid-template-columns: minmax(260px, 1.3fr) minmax(300px, 1.7fr) 120px 120px 210px;
}

.tag-table-head,
.tag-table-row {
  grid-template-columns: minmax(180px, 1.2fr) minmax(130px, 0.8fr) minmax(130px, 0.8fr) 90px 120px 150px;
}

.excerpt-table-row,
.topic-table-row,
.tag-table-row {
  width: 100%;
  min-height: 76px;
  padding: 12px 16px;
  border-bottom: 1px solid #eee7da;
  background: transparent;
  color: #1d2528;
  text-align: left;
}

.excerpt-table-row,
.topic-table-row {
  cursor: pointer;
}

.excerpt-table-row:hover,
.topic-table-row:hover {
  background: #f6f2ea;
}

.excerpt-table-row:focus-visible,
.topic-table-row:focus-visible {
  outline: 2px solid #2e6f62;
  outline-offset: -2px;
}

.excerpt-table-row:last-of-type,
.topic-table-row:last-of-type,
.tag-table-row:last-of-type {
  border-bottom: 0;
}

.table-quote,
.table-source,
.table-tags {
  min-width: 0;
}

.table-quote {
  display: grid;
  gap: 4px;
}

.table-quote strong,
.table-quote small,
.table-source {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.table-quote small {
  color: #6e7678;
  font-size: 0.82rem;
}

.table-source {
  color: #4c5b60;
  font-size: 0.88rem;
}

.table-tags,
.row-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.table-tags {
  overflow: hidden;
  white-space: nowrap;
}

.table-tag,
.table-tag-more {
  flex: 0 0 auto;
  max-width: 78px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #2e6f62;
  font-size: 0.78rem;
  font-weight: 700;
}

.table-tag {
  padding: 1px 6px;
  border-radius: 5px;
  background: #e8eee6;
}

.table-tag-more {
  color: #6e7678;
}

.tag-name-cell {
  min-width: 0;
  overflow: hidden;
  color: #1f6658;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-color-swatch {
  display: inline-block;
  width: 18px;
  height: 18px;
  margin-right: 8px;
  border: 1px solid rgba(37, 50, 56, 0.12);
  border-radius: 999px;
  vertical-align: middle;
}

.subtle-inline {
  color: #6e7678;
  font-size: 0.82rem;
}

.table-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.topic-status {
  justify-self: start;
  padding: 2px 8px;
  border-radius: 999px;
  background: #e8eee6;
  color: #2e6f62;
  font-size: 0.78rem;
  font-weight: 700;
}

.row-actions {
  justify-content: flex-end;
}

.table-empty {
  padding: 18px;
}

.editor-page {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
}

.editor-surface {
  display: grid;
  align-content: start;
  gap: 16px;
  flex: 1 1 auto;
  min-height: 0;
  padding: 20px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
  overflow: auto;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  flex: 0 0 auto;
}

.reader-page {
  min-height: 0;
  overflow: hidden;
}

.reader-surface {
  display: grid;
  align-content: start;
  gap: 22px;
  height: 100%;
  min-height: 0;
  padding: 24px 28px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
  overflow: auto;
}

.reader-meta {
  display: grid;
  gap: 6px;
  padding-bottom: 14px;
  border-bottom: 1px solid #eee7da;
}

.reader-section {
  display: grid;
  gap: 10px;
}

.item-title {
  display: -webkit-box;
  overflow: hidden;
  color: #1d2528;
  font-weight: 700;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.item-meta {
  overflow: hidden;
  color: #6e7678;
  font-size: 0.8rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modal-form {
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
  min-height: 34px;
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
.secondary-action,
.danger-action {
  min-height: 40px;
  padding: 0 14px;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
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

.danger-action {
  border: 1px solid #d9a29c;
  background: #fff8f6;
  color: #a23b32;
}

.primary-action:disabled,
.secondary-action:disabled,
.danger-action:disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.error-message {
  margin: 0;
  color: #a23b32;
}

.app-error {
  position: fixed;
  right: 20px;
  bottom: 20px;
  z-index: 2;
  max-width: min(420px, calc(100vw - 40px));
  margin: 0;
  padding: 10px 12px;
  border: 1px solid #d9a29c;
  border-radius: 6px;
  background: #fff8f6;
  color: #a23b32;
  box-shadow: 0 8px 18px rgba(38, 35, 30, 0.12);
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgba(29, 37, 40, 0.46);
}

.modal-panel {
  width: min(760px, 100%);
  max-height: min(860px, calc(100vh - 48px));
  overflow: hidden;
  border: 1px solid #d6cfc2;
  border-radius: 8px;
  background: #fbf8f1;
  box-shadow: 0 24px 70px rgba(24, 25, 23, 0.24);
}

.modal-panel:has(.collect-excerpt-form) {
  display: flex;
  flex-direction: column;
  width: min(980px, calc(100vw - 48px));
  height: min(760px, calc(100vh - 48px));
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 18px 20px;
  border-bottom: 1px solid #ded7ca;
}

.modal-body {
  max-height: calc(100vh - 138px);
  overflow: auto;
  padding: 20px;
}

.modal-panel:has(.collect-excerpt-form) .modal-body {
  flex: 1 1 auto;
  min-height: 0;
  max-height: none;
  overflow: hidden;
}

.icon-button {
  width: 34px;
  height: 34px;
  border-radius: 6px;
  background: #efe8da;
  color: #2d3a3f;
  cursor: pointer;
  font-size: 1.35rem;
  line-height: 1;
}

.database-path {
  margin: 24px 0 0;
  overflow-wrap: anywhere;
  color: #6e7678;
  font-size: 0.78rem;
}

.excerpt-card {
  display: grid;
  gap: 12px;
  padding: 18px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

.edit-form {
  display: grid;
  gap: 12px;
}

.edit-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 96px 140px;
  gap: 12px;
}

.source-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.source-line {
  margin: 0;
  color: #6f6257;
  font-size: 0.88rem;
  font-weight: 700;
}

.action-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.action-row .primary-action,
.action-row .secondary-action,
.action-row .danger-action {
  min-height: 32px;
  padding: 0 12px;
}

.topic-list,
.node-list {
  display: grid;
  align-content: start;
  grid-auto-rows: max-content;
  gap: 8px;
  margin-top: 18px;
}

.desktop-side-pane .topic-list {
  flex: 0 1 auto;
  min-height: 0;
  overflow: auto;
  padding-right: 2px;
}

.topic-selector,
.node-selector {
  display: flex;
  align-items: center;
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

.topic-selector-block,
.node-editor {
  display: grid;
  gap: 10px;
}

.topic-selector-block {
  align-content: start;
}

.plain-selector {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  min-height: 32px;
  padding: 0;
  background: transparent;
  color: inherit;
  cursor: pointer;
  text-align: left;
}

.topic-selector.active,
.node-selector.active {
  border-color: #2e6f62;
  background: #e8eee6;
}

.topic-selector:focus-visible,
.node-selector:focus-visible {
  outline: 2px solid #2e6f62;
  outline-offset: 2px;
}

.topic-selector small {
  color: #7a817f;
}

.topic-workspace-grid {
  display: grid;
  grid-template-columns: minmax(280px, 320px) minmax(0, 1fr);
  gap: 18px;
  align-items: stretch;
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}

.topic-workspace-grid > .stack,
.workspace-panel > .excerpt-list {
  overflow: auto;
  padding-right: 2px;
}

.topic-workspace {
  display: flex;
  flex-direction: column;
  gap: 14px;
  flex: 1 1 auto;
  min-height: 0;
}

.topic-tabs-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  flex: 0 0 auto;
  padding: 10px 12px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

.topic-tabs {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  overflow: auto;
}

.topic-tab {
  flex: 0 0 auto;
  min-height: 34px;
  padding: 0 12px;
  border: 1px solid transparent;
  border-radius: 999px;
  background: transparent;
  color: #2d3a3f;
  cursor: pointer;
  font-weight: 700;
}

.topic-tab:hover,
.topic-tab.active {
  border-color: #bfd0c8;
  background: #e8eee6;
  color: #2e6f62;
}

.topic-tab-actions {
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
}

.topic-node-summary {
  flex: 0 0 auto;
  margin: -4px 0 0;
  color: #6e7678;
  font-size: 0.88rem;
}

.topic-material-pane {
  display: flex;
  min-height: 0;
  overflow: hidden;
}

.context-section {
  display: grid;
  gap: 12px;
  padding: 16px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

.material-context-section {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
}

.material-list-scroll {
  display: grid;
  align-content: start;
  grid-auto-rows: max-content;
  gap: 8px;
  flex: 1 1 auto;
  min-height: 0;
  margin-right: -10px;
  overflow: auto;
  padding-right: 10px;
}

.context-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-top: 4px;
}

.context-actions .subtle-text {
  flex-basis: 100%;
}

.context-caption {
  margin: 0;
  color: #6e7678;
  font-size: 0.82rem;
  font-weight: 700;
}

.topic-detail-pane .reading-body {
  width: 100%;
  max-width: none;
}

.document-detail-pane {
  display: block;
}

.document-detail-pane .detail-document,
.document-detail-pane .edit-document {
  display: flex;
  flex-direction: column;
  gap: 10px;
  height: 100%;
  min-height: 0;
}

.document-detail-pane .document-header {
  flex: 0 0 auto;
}

.document-detail-pane .document-scroll {
  display: block;
  flex: 1 1 auto;
  height: auto;
  min-height: 0;
  padding: 0;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
  overflow: hidden;
}

.document-detail-pane .document-body,
.document-detail-pane .inline-editor-body {
  width: 100%;
  max-width: none;
  height: 100%;
  min-height: 100%;
  padding: 24px 26px;
}

.document-detail-pane .document-body {
  align-content: start;
  align-items: start;
  border: 0;
  border-radius: 0;
  background: transparent;
  overflow: auto;
}

.document-detail-pane .document-body blockquote,
.document-detail-pane .document-body .reflection,
.document-detail-pane .document-body .tag-row {
  align-self: start;
  width: 100%;
  max-width: none;
}

.document-detail-pane .inline-editor-body {
  display: grid;
  align-content: start;
  gap: 16px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
}

.document-detail-pane .edit-document .inline-editor-body {
  height: 100%;
  min-height: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  overflow: auto;
}

.document-detail-pane .inline-editor-body textarea {
  min-height: 140px;
}

.document-detail-pane.is-editing .document-header,
.document-detail-pane.is-editing .inline-editor-body {
  border-color: #bfd0c8;
  background: #fbfdf9;
}

.document-detail-pane.is-editing .document-header {
  align-items: center;
}

.document-detail-pane.is-editing .document-header .action-row {
  align-items: center;
}

.document-detail-pane.is-editing .document-scroll {
  border-color: #bfd0c8;
  background: #fbfdf9;
}

.document-detail-pane.is-editing .edit-document .inline-editor-body {
  background: transparent;
}

.topic-excerpt-editor .readonly-excerpt-preview {
  display: grid;
  gap: 10px;
  padding-bottom: 2px;
}

.collect-excerpt-form {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 14px;
  height: 100%;
  min-height: 0;
}

.excerpt-picker {
  display: grid;
  grid-template-columns: minmax(260px, 0.9fr) minmax(280px, 1fr);
  gap: 12px;
  min-height: 0;
}

.excerpt-picker-list,
.excerpt-picker-preview {
  min-height: 0;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
  overflow: auto;
}

.excerpt-picker-list {
  display: grid;
  align-content: start;
  grid-auto-rows: max-content;
  gap: 8px;
  padding: 10px;
}

.excerpt-picker-item {
  display: grid;
  flex: 0 0 auto;
  gap: 6px;
  width: 100%;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: #1d2528;
  cursor: pointer;
  text-align: left;
}

.excerpt-picker-item:hover,
.excerpt-picker-item.active {
  border-color: #bfd0c8;
  background: #e8eee6;
}

.excerpt-picker-preview {
  display: grid;
  align-content: start;
  gap: 12px;
  padding: 14px;
}

.excerpt-picker-selected {
  display: grid;
  gap: 10px;
  padding-bottom: 2px;
}

.excerpt-picker-preview textarea {
  min-height: 110px;
}

.compact-tags {
  max-height: 28px;
  overflow: hidden;
}

.excerpt-picker-empty {
  padding: 8px 2px;
}

.book-manager {
  display: grid;
  grid-template-columns: minmax(280px, 340px) minmax(0, 1fr);
  gap: 18px;
  overflow: hidden;
}

.book-list-pane,
.book-detail-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 16px;
  border: 1px solid #ded7ca;
  border-radius: 8px;
  background: #fffdf9;
  overflow: hidden;
}

.book-list-scroll,
.chapter-list {
  display: grid;
  align-content: start;
  grid-auto-rows: max-content;
  gap: 8px;
  flex: 1 1 auto;
  min-height: 0;
  margin-top: 14px;
  overflow: auto;
}

.book-list-scroll {
  margin-right: -10px;
  padding-right: 10px;
}

.book-list-item {
  display: grid;
  gap: 5px;
  width: 100%;
  min-height: 64px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: #1d2528;
  cursor: pointer;
  text-align: left;
}

.book-list-item:hover,
.book-list-item.active {
  border-color: #bfd0c8;
  background: #e8eee6;
}

.book-detail-header,
.chapter-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.book-detail-header {
  flex: 0 0 auto;
  padding-bottom: 16px;
  border-bottom: 1px solid #ded7ca;
}

.chapter-section {
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding-top: 16px;
}

.chapter-row {
  min-height: 60px;
  padding: 10px 12px;
  border: 1px solid #ded7ca;
  border-radius: 6px;
  background: #fbf8f1;
}

.topic-empty-state {
  gap: 12px;
  min-height: 240px;
}

.tag-empty-state {
  gap: 12px;
  min-height: 240px;
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
  body {
    overflow: auto;
  }

  .app-shell {
    height: auto;
    min-height: 100vh;
    grid-template-columns: 1fr;
    overflow: visible;
  }

  .app-main-frame {
    grid-template-columns: 1fr;
    padding: 20px;
    overflow: visible;
  }

  .page-panel {
    grid-column: auto;
  }

  .sidebar {
    border-right: 0;
  }

  .topic-panel {
    border-right: 0;
    border-bottom: 1px solid #d9d3c7;
  }

  .page-header,
  .toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .topic-workspace-grid {
    grid-template-columns: 1fr;
    overflow: visible;
  }

  .desktop-view,
  .desktop-side-pane,
  .split-workspace {
    overflow: visible;
  }

  .split-workspace {
    grid-template-columns: 1fr;
  }

  .list-pane,
  .detail-pane,
  .desktop-side-pane .topic-list,
  .topic-workspace-grid > .stack,
  .workspace-panel > .excerpt-list {
    overflow: visible;
  }

  .filter-bar {
    grid-template-columns: 1fr;
  }

  .excerpt-table-head,
  .topic-table-head {
    display: none;
  }

  .excerpt-table-row,
  .topic-table-row {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .row-actions {
    justify-content: flex-start;
  }

  .edit-grid,
  .source-grid {
    grid-template-columns: 1fr;
  }
}
</style>
