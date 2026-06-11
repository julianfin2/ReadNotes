<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import BaseModal from "./BaseModal.vue";
import type { Excerpt, ExcerptFilters, UpdateExcerptInput } from "../types/excerpt";
import type { Tag } from "../types/tag";

const props = defineProps<{
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
const filterModalOpen = ref(false);
const deleteModalOpen = ref(false);
const editingId = ref("");
const selectedExcerptId = ref("");
const deletingExcerptId = ref("");

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

const activeFilterLabels = computed(() => {
  const labels: string[] = [];

  if (filters.search) {
    labels.push(`搜索：${filters.search}`);
  }
  if (filters.tagName) {
    labels.push(`标签：#${filters.tagName}`);
  }
  if (filters.status) {
    labels.push(`状态：${filters.status}`);
  }
  if (filters.minImportance) {
    labels.push(`重要性 >= ${filters.minImportance}`);
  }
  if (filters.sortBy !== "createdAt") {
    labels.push(`排序：${filters.sortBy}`);
  }
  if (filters.sortDirection !== "desc") {
    labels.push("升序");
  }

  return labels;
});

const selectedExcerpt = computed(() => {
  return props.excerpts.find((excerpt) => excerpt.id === selectedExcerptId.value) || null;
});

const isEditingSelected = computed(() => {
  return Boolean(selectedExcerpt.value && selectedExcerpt.value.id === editingId.value);
});

const isEditDirty = computed(() => {
  const excerpt = selectedExcerpt.value;

  if (!excerpt || editDraft.id !== excerpt.id) {
    return false;
  }

  const excerptTags = excerpt.tags.map((tag) => tag.name).sort();
  const draftTags = parseTagInput(editDraft.tagInput).sort();

  return (
    editDraft.quote !== excerpt.quote ||
    editDraft.reflection !== normalizeOptionalText(excerpt.reflection) ||
    editDraft.sourceWorkId !== (excerpt.sourceWorkId || null) ||
    editDraft.bookTitle !== normalizeOptionalText(excerpt.bookTitle) ||
    editDraft.chapterTitle !== normalizeOptionalText(excerpt.chapterTitle) ||
    editDraft.location !== normalizeOptionalText(excerpt.location) ||
    editDraft.importance !== excerpt.importance ||
    editDraft.status !== excerpt.status ||
    excerptTags.join("\n") !== draftTags.join("\n")
  );
});

const canSaveEdit = computed(() => {
  return editDraft.quote.trim().length > 0 && isEditDirty.value && !props.isSaving;
});

watch(
  () => props.excerpts,
  (excerpts) => {
    if (excerpts.length === 0) {
      selectedExcerptId.value = "";
      return;
    }

    if (!excerpts.some((excerpt) => excerpt.id === selectedExcerptId.value)) {
      selectedExcerptId.value = excerpts[0].id;
    }

    if (editingId.value && !excerpts.some((excerpt) => excerpt.id === editingId.value)) {
      editingId.value = "";
    }
  },
  { immediate: true },
);

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
}

function selectExcerpt(id: string) {
  if (id === selectedExcerptId.value) {
    return;
  }

  if (!discardEditing()) {
    return;
  }

  selectedExcerptId.value = id;
}

function requestDeleteExcerpt(id: string) {
  deletingExcerptId.value = id;
  deleteModalOpen.value = true;
}

function confirmDeleteExcerpt() {
  if (!deletingExcerptId.value) {
    return;
  }

  emit("deleteExcerpt", deletingExcerptId.value);
  deletingExcerptId.value = "";
  deleteModalOpen.value = false;
}

function cancelDeleteExcerpt() {
  deletingExcerptId.value = "";
  deleteModalOpen.value = false;
}

function submitEdit() {
  if (!editDraft.id || !canSaveEdit.value) {
    return;
  }

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
  editingId.value = "";
}

function cancelEditing() {
  discardEditing();
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

function normalizeOptionalText(value: string | null | undefined) {
  return value || "";
}

function discardEditing() {
  if (!editingId.value) {
    return true;
  }

  if (isEditDirty.value && !window.confirm("当前摘抄有未保存修改，确定放弃吗？")) {
    return false;
  }

  editingId.value = "";
  return true;
}
</script>

<template>
  <section class="page-panel desktop-view library-workbench">
    <header class="page-header desktop-toolbar">
      <div>
        <p class="eyebrow">Library</p>
        <h2>摘抄库</h2>
        <p class="subtle-text">{{ props.excerpts.length }} 条摘抄</p>
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

    <div v-if="activeFilterLabels.length > 0" class="filter-chip-row">
      <span v-for="label in activeFilterLabels" :key="label" class="filter-chip">
        {{ label }}
      </span>
      <button class="text-action" type="button" @click="resetFilters">清空筛选</button>
    </div>

    <div class="split-workspace">
      <aside class="list-pane">
        <div class="list-scroll">
          <button
            v-for="excerpt in props.excerpts"
            :key="excerpt.id"
            class="excerpt-list-item"
            :class="{ active: excerpt.id === selectedExcerpt?.id }"
            type="button"
            @click="selectExcerpt(excerpt.id)"
          >
            <span class="item-title">{{ excerpt.quote }}</span>
            <span v-if="excerpt.bookTitle || excerpt.chapterTitle" class="item-meta">
              <span v-if="excerpt.bookTitle">《{{ excerpt.bookTitle }}》</span>
              <span v-if="excerpt.bookTitle && excerpt.chapterTitle"> / </span>
              <span v-if="excerpt.chapterTitle">{{ excerpt.chapterTitle }}</span>
            </span>
            <span class="item-meta">
              重要性 {{ excerpt.importance }} / {{ new Date(excerpt.createdAt).toLocaleDateString() }}
            </span>
          </button>

          <p v-if="props.excerpts.length === 0" class="empty-state">还没有摘抄。</p>
        </div>
      </aside>

      <article
        v-if="selectedExcerpt"
        class="detail-pane excerpt-detail-pane library-detail-pane"
        :class="{ 'is-editing': isEditingSelected }"
      >
        <form v-if="isEditingSelected" class="detail-document edit-document" @submit.prevent="submitEdit">
          <header class="detail-header document-header">
            <div>
              <p class="eyebrow">Editing</p>
              <h3>编辑摘抄</h3>
              <footer>
                <span>原地修改当前选中的摘抄</span>
                <span>{{ new Date(selectedExcerpt.createdAt).toLocaleString() }}</span>
              </footer>
            </div>
            <div class="action-row">
              <button class="secondary-action" type="button" @click="cancelEditing">取消</button>
              <button
                class="primary-action"
                :disabled="!canSaveEdit"
                type="submit"
              >
                保存
              </button>
            </div>
          </header>

          <div class="detail-scroll document-scroll">
            <div class="inline-editor-body">
              <label>
                摘抄原文
                <textarea v-model="editDraft.quote" rows="9" />
              </label>
              <label>
                阅读笔记
                <textarea v-model="editDraft.reflection" rows="7" />
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
            </div>
          </div>
        </form>

        <div v-else class="detail-document">
          <header class="detail-header document-header">
            <div>
              <p v-if="selectedExcerpt.bookTitle || selectedExcerpt.chapterTitle" class="source-line">
                <span v-if="selectedExcerpt.bookTitle">《{{ selectedExcerpt.bookTitle }}》</span>
                <span v-if="selectedExcerpt.bookTitle && selectedExcerpt.chapterTitle"> / </span>
                <span v-if="selectedExcerpt.chapterTitle">{{ selectedExcerpt.chapterTitle }}</span>
              </p>
              <p v-else class="source-line">未记录书籍与章节</p>
              <footer>
                <span>重要性 {{ selectedExcerpt.importance }}</span>
                <span>{{ selectedExcerpt.status }}</span>
                <span>{{ new Date(selectedExcerpt.createdAt).toLocaleString() }}</span>
              </footer>
            </div>
            <div class="action-row">
              <button class="secondary-action" type="button" @click="startEditing(selectedExcerpt)">
                编辑
              </button>
              <button
                class="secondary-action"
                type="button"
                :disabled="selectedExcerpt.status === 'archived'"
                @click="$emit('archiveExcerpt', selectedExcerpt.id)"
              >
                归档
              </button>
              <button
                class="danger-action"
                type="button"
                @click="requestDeleteExcerpt(selectedExcerpt.id)"
              >
                删除
              </button>
            </div>
          </header>

          <div class="detail-scroll document-scroll">
            <section class="reading-body document-body">
              <blockquote>{{ selectedExcerpt.quote }}</blockquote>
              <p v-if="selectedExcerpt.reflection" class="reflection">
                {{ selectedExcerpt.reflection }}
              </p>
              <p v-else class="empty-state">还没有记录阅读笔记。</p>
              <div v-if="selectedExcerpt.tags.length > 0" class="tag-row">
                <span v-for="tag in selectedExcerpt.tags" :key="tag.id" class="tag-pill">
                  #{{ tag.name }}
                </span>
              </div>
            </section>
          </div>
        </div>
      </article>
      <section v-else class="detail-pane empty-detail">
        <p class="empty-state">选择一条摘抄查看详情。</p>
      </section>
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
        <button class="primary-action" :disabled="props.isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="deleteModalOpen" title="删除摘抄" @close="cancelDeleteExcerpt">
    <div class="modal-form">
      <p class="reflection">删除后这条摘抄及其关联记录将不可见。确认删除吗？</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelDeleteExcerpt">取消</button>
        <button class="danger-action" type="button" @click="confirmDeleteExcerpt">删除</button>
      </div>
    </div>
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
          <option v-for="tag in props.tags" :key="tag.id" :value="tag.name">#{{ tag.name }}</option>
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
