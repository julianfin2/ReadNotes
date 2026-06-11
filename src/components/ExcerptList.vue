<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import BaseModal from "./BaseModal.vue";
import type { Excerpt, ExcerptFilters, UpdateExcerptInput } from "../types/excerpt";
import type { Tag } from "../types/tag";

defineProps<{
  excerpts: Excerpt[];
  tags: Tag[];
  isSaving: boolean;
}>();

const emit = defineEmits<{
  applyFilters: [filters: ExcerptFilters];
  archiveExcerpt: [id: string];
  createExcerpt: [
    input: {
      quote: string;
      reflection: string;
      bookTitle: string;
      chapterTitle: string;
      location: string;
      importance: number;
      tagNames: string[];
    },
  ];
  deleteExcerpt: [id: string];
  updateExcerpt: [input: UpdateExcerptInput];
}>();

const createModalOpen = ref(false);
const editModalOpen = ref(false);
const filterModalOpen = ref(false);
const editingId = ref("");

const filters = reactive<ExcerptFilters>({
  search: "",
  tagName: "",
  status: "",
  minImportance: null,
  sortBy: "createdAt",
  sortDirection: "desc",
});

const createDraft = reactive({
  quote: "",
  reflection: "",
  bookTitle: "",
  chapterTitle: "",
  location: "",
  importance: 3,
  tagInput: "",
});

const editDraft = reactive<UpdateExcerptInput & { tagInput: string }>({
  id: "",
  quote: "",
  reflection: "",
  sourceWorkId: null,
  bookTitle: "",
  chapterTitle: "",
  location: "",
  importance: 3,
  status: "inbox",
  tagNames: [],
  tagInput: "",
});

const activeFilterCount = computed(() => {
  return [
    filters.search,
    filters.tagName,
    filters.status,
    filters.minImportance,
    filters.sortBy !== "createdAt" ? filters.sortBy : "",
    filters.sortDirection !== "desc" ? filters.sortDirection : "",
  ].filter(Boolean).length;
});

function submitCreate() {
  emit("createExcerpt", {
    quote: createDraft.quote,
    reflection: createDraft.reflection,
    bookTitle: createDraft.bookTitle,
    chapterTitle: createDraft.chapterTitle,
    location: createDraft.location,
    importance: createDraft.importance,
    tagNames: parseTagInput(createDraft.tagInput),
  });
  resetCreateDraft();
  createModalOpen.value = false;
}

function startEditing(excerpt: Excerpt) {
  editingId.value = excerpt.id;
  editDraft.id = excerpt.id;
  editDraft.quote = excerpt.quote;
  editDraft.reflection = excerpt.reflection || "";
  editDraft.sourceWorkId = excerpt.sourceWorkId || null;
  editDraft.bookTitle = excerpt.bookTitle || "";
  editDraft.chapterTitle = excerpt.chapterTitle || "";
  editDraft.location = excerpt.location || "";
  editDraft.importance = excerpt.importance;
  editDraft.status = excerpt.status;
  editDraft.tagNames = excerpt.tags.map((tag) => tag.name);
  editDraft.tagInput = excerpt.tags.map((tag) => `#${tag.name}`).join(" ");
  editModalOpen.value = true;
}

function submitEdit() {
  emit("updateExcerpt", {
    id: editDraft.id,
    quote: editDraft.quote,
    reflection: editDraft.reflection,
    sourceWorkId: editDraft.sourceWorkId,
    bookTitle: editDraft.bookTitle,
    chapterTitle: editDraft.chapterTitle,
    location: editDraft.location,
    importance: editDraft.importance,
    status: editDraft.status,
    tagNames: parseTagInput(editDraft.tagInput),
  });
  editModalOpen.value = false;
  editingId.value = "";
}

function applyFilters() {
  emit("applyFilters", { ...filters });
  filterModalOpen.value = false;
}

function resetFilters() {
  filters.search = "";
  filters.tagName = "";
  filters.status = "";
  filters.minImportance = null;
  filters.sortBy = "createdAt";
  filters.sortDirection = "desc";
  applyFilters();
}

function resetCreateDraft() {
  createDraft.quote = "";
  createDraft.reflection = "";
  createDraft.bookTitle = "";
  createDraft.chapterTitle = "";
  createDraft.location = "";
  createDraft.importance = 3;
  createDraft.tagInput = "";
}

function parseTagInput(value: string) {
  return value
    .split(/[\s,，#]+/)
    .map((tag) => tag.trim())
    .filter(Boolean);
}
</script>

<template>
  <section class="page-panel library-panel">
    <header class="page-header">
      <div>
        <p class="eyebrow">Library</p>
        <h2>摘抄库</h2>
        <p class="subtle-text">{{ excerpts.length }} 条摘抄</p>
      </div>

      <div class="toolbar">
        <button class="secondary-action" type="button" @click="filterModalOpen = true">
          筛选{{ activeFilterCount ? ` (${activeFilterCount})` : "" }}
        </button>
        <button class="primary-action" type="button" @click="createModalOpen = true">
          新增摘抄
        </button>
      </div>
    </header>

    <div class="excerpt-list">
      <article v-for="excerpt in excerpts" :key="excerpt.id" class="excerpt-card">
        <blockquote>{{ excerpt.quote }}</blockquote>
        <p v-if="excerpt.bookTitle || excerpt.chapterTitle" class="source-line">
          <span v-if="excerpt.bookTitle">《{{ excerpt.bookTitle }}》</span>
          <span v-if="excerpt.bookTitle && excerpt.chapterTitle"> / </span>
          <span v-if="excerpt.chapterTitle">{{ excerpt.chapterTitle }}</span>
        </p>
        <p v-if="excerpt.reflection" class="reflection">{{ excerpt.reflection }}</p>
        <div v-if="excerpt.tags.length > 0" class="tag-row">
          <span v-for="tag in excerpt.tags" :key="tag.id" class="tag-pill">
            #{{ tag.name }}
          </span>
        </div>
        <footer>
          <span>重要性 {{ excerpt.importance }}</span>
          <span>{{ excerpt.status }}</span>
          <span>{{ new Date(excerpt.createdAt).toLocaleString() }}</span>
        </footer>

        <div class="action-row">
          <button class="secondary-action" type="button" @click="startEditing(excerpt)">
            编辑
          </button>
          <button
            class="secondary-action"
            type="button"
            :disabled="excerpt.status === 'archived'"
            @click="$emit('archiveExcerpt', excerpt.id)"
          >
            归档
          </button>
          <button class="danger-action" type="button" @click="$emit('deleteExcerpt', excerpt.id)">
            删除
          </button>
        </div>
      </article>

      <p v-if="excerpts.length === 0" class="empty-state">还没有摘抄。</p>
    </div>
  </section>

  <BaseModal :open="createModalOpen" title="新增摘抄" @close="createModalOpen = false">
    <form class="modal-form" @submit.prevent="submitCreate">
      <label>
        原文
        <textarea v-model="createDraft.quote" rows="7" placeholder="输入摘抄原文" />
      </label>
      <label>
        初始理解
        <textarea v-model="createDraft.reflection" rows="5" placeholder="写下此刻的理解" />
      </label>
      <div class="source-grid">
        <label>
          书籍名
          <input v-model="createDraft.bookTitle" placeholder="例如：置身事内" />
        </label>
        <label>
          章节名
          <input v-model="createDraft.chapterTitle" placeholder="例如：地方政府的权力与事务" />
        </label>
      </div>
      <label>
        标签
        <input v-model="createDraft.tagInput" placeholder="例如：人性 写作素材 #焦虑" />
      </label>
      <div class="edit-grid">
        <label>
          位置
          <input v-model="createDraft.location" placeholder="页码、章节，可选" />
        </label>
        <label>
          重要性
          <input v-model.number="createDraft.importance" max="5" min="1" type="number" />
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="createModalOpen = false">取消</button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="editModalOpen" title="编辑摘抄" @close="editModalOpen = false">
    <form class="modal-form" @submit.prevent="submitEdit">
      <label>
        原文
        <textarea v-model="editDraft.quote" rows="7" />
      </label>
      <label>
        初始理解
        <textarea v-model="editDraft.reflection" rows="5" />
      </label>
      <div class="source-grid">
        <label>
          书籍名
          <input v-model="editDraft.bookTitle" />
        </label>
        <label>
          章节名
          <input v-model="editDraft.chapterTitle" />
        </label>
      </div>
      <label>
        标签
        <input v-model="editDraft.tagInput" />
      </label>
      <div class="edit-grid">
        <label>
          位置
          <input v-model="editDraft.location" />
        </label>
        <label>
          重要性
          <input v-model.number="editDraft.importance" max="5" min="1" type="number" />
        </label>
        <label>
          状态
          <select v-model="editDraft.status">
            <option value="inbox">inbox</option>
            <option value="processed">processed</option>
            <option value="archived">archived</option>
          </select>
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="editModalOpen = false">取消</button>
        <button class="primary-action" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="filterModalOpen" title="筛选摘抄" @close="filterModalOpen = false">
    <form class="modal-form" @submit.prevent="applyFilters">
      <label>
        搜索
        <input v-model="filters.search" placeholder="搜索原文、理解、书籍或章节" />
      </label>
      <label>
        标签
        <select v-model="filters.tagName">
          <option value="">全部标签</option>
          <option v-for="tag in tags" :key="tag.id" :value="tag.name">#{{ tag.name }}</option>
        </select>
      </label>
      <div class="edit-grid">
        <label>
          状态
          <select v-model="filters.status">
            <option value="">全部状态</option>
            <option value="inbox">inbox</option>
            <option value="processed">processed</option>
            <option value="archived">archived</option>
          </select>
        </label>
        <label>
          最低重要性
          <input v-model.number="filters.minImportance" max="5" min="1" type="number" />
        </label>
        <label>
          排序
          <select v-model="filters.sortBy">
            <option value="createdAt">创建时间</option>
            <option value="updatedAt">更新时间</option>
            <option value="importance">重要性</option>
          </select>
        </label>
      </div>
      <label>
        方向
        <select v-model="filters.sortDirection">
          <option value="desc">降序</option>
          <option value="asc">升序</option>
        </select>
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="resetFilters">清空</button>
        <button class="primary-action" type="submit">应用</button>
      </div>
    </form>
  </BaseModal>
</template>
