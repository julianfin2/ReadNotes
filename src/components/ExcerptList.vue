<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import BaseModal from "./BaseModal.vue";
import CustomSelect from "./CustomSelect.vue";
import type { Excerpt, ExcerptFilters, UpdateExcerptInput } from "../types/excerpt";
import type { Tag } from "../types/tag";

const props = defineProps<{
  excerpts: Excerpt[];
  tags: Tag[];
  isSaving: boolean;
}>();

const emit = defineEmits<{
  applyFilters: [filters: ExcerptFilters];
  createExcerpt: [
    input: {
      quote: string;
      reflection: string;
      bookTitle: string;
      chapterTitle: string;
      tagNames: string[];
    },
  ];
  deleteExcerpt: [id: string];
  updateExcerpt: [input: UpdateExcerptInput];
}>();

const filterModalOpen = ref(false);
const deleteModalOpen = ref(false);
const viewMode = ref<"list" | "detail" | "create" | "edit">("list");
const activeExcerptId = ref("");
const deletingExcerptId = ref("");

const filters = reactive<ExcerptFilters>({
  search: "",
  tagName: "",
  sortBy: "createdAt",
  sortDirection: "desc",
});

const createDraft = reactive({
  quote: "",
  reflection: "",
  bookTitle: "",
  chapterTitle: "",
  tagInput: "",
});

const editDraft = reactive<UpdateExcerptInput & { tagInput: string }>({
  id: "",
  quote: "",
  reflection: "",
  sourceWorkId: null,
  bookTitle: "",
  chapterTitle: "",
  tagNames: [],
  tagInput: "",
});

const activeFilterCount = computed(() => {
  return [
    filters.search,
    filters.tagName,
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
  if (filters.sortBy !== "createdAt") {
    labels.push(`排序：${filters.sortBy}`);
  }
  if (filters.sortDirection !== "desc") {
    labels.push("升序");
  }

  return labels;
});

const tagFilterOptions = computed(() => [
  { value: "", label: "全部标签" },
  ...props.tags.map((tag) => ({ value: tag.name, label: `#${tag.name}` })),
]);

const sortByOptions = [
  { value: "createdAt", label: "创建时间" },
  { value: "updatedAt", label: "更新时间" },
];

const sortDirectionOptions = [
  { value: "desc", label: "降序" },
  { value: "asc", label: "升序" },
];

const editingExcerpt = computed(() => {
  return props.excerpts.find((excerpt) => excerpt.id === editDraft.id) || null;
});

const activeExcerpt = computed(() => {
  return props.excerpts.find((excerpt) => excerpt.id === activeExcerptId.value) || null;
});

const pageTitle = computed(() => {
  if (viewMode.value === "create") {
    return "新增摘抄";
  }

  if (viewMode.value === "edit") {
    return "编辑摘抄";
  }

  if (viewMode.value === "detail") {
    return "阅读摘抄";
  }

  return "摘抄库";
});

const canSaveCreate = computed(() => {
  return createDraft.quote.trim().length > 0 && !props.isSaving;
});

const isCreateDirty = computed(() => {
  return Boolean(
    createDraft.quote.trim() ||
      createDraft.reflection.trim() ||
      createDraft.bookTitle.trim() ||
      createDraft.chapterTitle.trim() ||
      createDraft.tagInput.trim(),
  );
});

const isEditDirty = computed(() => {
  const excerpt = editingExcerpt.value;

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
    excerptTags.join("\n") !== draftTags.join("\n")
  );
});

const canSaveEdit = computed(() => {
  return editDraft.quote.trim().length > 0 && isEditDirty.value && !props.isSaving;
});

watch(
  () => props.excerpts,
  (excerpts) => {
    if (viewMode.value === "edit" && !excerpts.some((excerpt) => excerpt.id === editDraft.id)) {
      goToList();
    }

    if (
      viewMode.value === "detail" &&
      activeExcerptId.value &&
      !excerpts.some((excerpt) => excerpt.id === activeExcerptId.value)
    ) {
      goToList();
    }
  },
  { immediate: true },
);

function submitCreate() {
  if (!canSaveCreate.value) {
    return;
  }

  emit("createExcerpt", {
    quote: createDraft.quote,
    reflection: createDraft.reflection,
    bookTitle: createDraft.bookTitle,
    chapterTitle: createDraft.chapterTitle,
    tagNames: parseTagInput(createDraft.tagInput),
  });
  resetCreateDraft();
  viewMode.value = "list";
}

function startEditing(excerpt: Excerpt) {
  if (!confirmLeaveEditor()) {
    return;
  }

  activeExcerptId.value = excerpt.id;
  editDraft.id = excerpt.id;
  editDraft.quote = excerpt.quote;
  editDraft.reflection = excerpt.reflection || "";
  editDraft.sourceWorkId = excerpt.sourceWorkId || null;
  editDraft.bookTitle = excerpt.bookTitle || "";
  editDraft.chapterTitle = excerpt.chapterTitle || "";
  editDraft.tagNames = excerpt.tags.map((tag) => tag.name);
  editDraft.tagInput = excerpt.tags.map((tag) => `#${tag.name}`).join(" ");
  viewMode.value = "edit";
}

function openDetail(excerpt: Excerpt) {
  if (!confirmLeaveEditor()) {
    return;
  }

  activeExcerptId.value = excerpt.id;
  resetEditDraft();
  viewMode.value = "detail";
}

function requestDeleteExcerpt(id: string) {
  deletingExcerptId.value = id;
  deleteModalOpen.value = true;
}

function confirmDeleteExcerpt() {
  if (!deletingExcerptId.value) {
    return;
  }

  const deletedCurrentExcerpt = deletingExcerptId.value === activeExcerptId.value;

  emit("deleteExcerpt", deletingExcerptId.value);
  deletingExcerptId.value = "";
  deleteModalOpen.value = false;

  if (deletedCurrentExcerpt) {
    goToList();
  }
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
    tagNames: parseTagInput(editDraft.tagInput),
  });
  activeExcerptId.value = editDraft.id;
  resetEditDraft();
  viewMode.value = "detail";
}

function startCreate() {
  if (!confirmLeaveEditor()) {
    return;
  }

  resetCreateDraft();
  viewMode.value = "create";
}

function goToList() {
  resetCreateDraft();
  resetEditDraft();
  activeExcerptId.value = "";
  viewMode.value = "list";
}

function cancelEditor() {
  if (confirmLeaveEditor()) {
    goToList();
  }
}

function applyFilters() {
  emit("applyFilters", { ...filters });
  filterModalOpen.value = false;
}

function resetFilters() {
  filters.search = "";
  filters.tagName = "";
  filters.sortBy = "createdAt";
  filters.sortDirection = "desc";
  applyFilters();
}

function resetCreateDraft() {
  createDraft.quote = "";
  createDraft.reflection = "";
  createDraft.bookTitle = "";
  createDraft.chapterTitle = "";
  createDraft.tagInput = "";
}

function resetEditDraft() {
  editDraft.id = "";
  editDraft.quote = "";
  editDraft.reflection = "";
  editDraft.sourceWorkId = null;
  editDraft.bookTitle = "";
  editDraft.chapterTitle = "";
  editDraft.tagNames = [];
  editDraft.tagInput = "";
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

function formatDate(value: string) {
  return new Date(value).toLocaleDateString();
}

function excerptSource(excerpt: Excerpt) {
  if (excerpt.bookTitle && excerpt.chapterTitle) {
    return `《${excerpt.bookTitle}》 / ${excerpt.chapterTitle}`;
  }

  if (excerpt.bookTitle) {
    return `《${excerpt.bookTitle}》`;
  }

  return excerpt.chapterTitle || "未记录";
}

function confirmLeaveEditor() {
  if (viewMode.value === "create" && isCreateDirty.value) {
    return window.confirm("当前新增摘抄还没有保存，确定离开吗？");
  }

  if (viewMode.value !== "edit") {
    return true;
  }

  if (isEditDirty.value && !window.confirm("当前摘抄有未保存修改，确定放弃吗？")) {
    return false;
  }

  return true;
}
</script>

<template>
  <section class="page-panel desktop-view library-workbench">
    <header class="page-header desktop-toolbar" :class="{ 'list-toolbar-header': viewMode === 'list' }">
      <div class="page-title-block">
        <h2>{{ pageTitle }}</h2>
        <p class="subtle-text">
          <template v-if="viewMode === 'list'">{{ props.excerpts.length }} 条摘抄</template>
          <template v-else-if="viewMode === 'detail'">
            {{ activeExcerpt ? excerptSource(activeExcerpt) : "查看摘抄内容" }}
          </template>
          <template v-else>保存或取消后返回阅读页或列表</template>
        </p>
      </div>

      <form v-if="viewMode === 'list'" class="toolbar list-toolbar" @submit.prevent="applyFilters">
        <input
          v-model="filters.search"
          class="toolbar-search"
          placeholder="搜索原文、笔记、书籍或章节"
        />
        <button class="secondary-action" type="button" @click="filterModalOpen = true">
          筛选{{ activeFilterCount ? ` (${activeFilterCount})` : "" }}
        </button>
        <button class="primary-action" type="button" @click="startCreate">
          新增摘抄
        </button>
      </form>
      <div v-else-if="viewMode === 'detail' && activeExcerpt" class="toolbar">
        <button class="secondary-action" type="button" @click="goToList">返回列表</button>
        <button class="secondary-action" type="button" @click="startEditing(activeExcerpt)">
          编辑
        </button>
        <button
          class="danger-action"
          type="button"
          @click="requestDeleteExcerpt(activeExcerpt.id)"
        >
          删除
        </button>
      </div>
      <div v-else class="toolbar">
        <button class="secondary-action" type="button" @click="cancelEditor">返回列表</button>
      </div>
    </header>

    <div v-if="viewMode === 'list' && activeFilterLabels.length > 0" class="filter-chip-row">
      <span v-for="label in activeFilterLabels" :key="label" class="filter-chip">
        {{ label }}
      </span>
      <button class="text-action" type="button" @click="resetFilters">清空筛选</button>
    </div>

    <div v-if="viewMode === 'list'" class="table-page">
      <div class="excerpt-table">
        <div class="excerpt-table-head">
          <span>摘抄</span>
          <span>来源</span>
          <span>标签</span>
          <span>创建时间</span>
          <span>操作</span>
        </div>

        <div
          v-for="excerpt in props.excerpts"
          :key="excerpt.id"
          class="excerpt-table-row"
          role="button"
          tabindex="0"
          @click="openDetail(excerpt)"
          @keydown.enter="openDetail(excerpt)"
        >
          <span class="table-quote">
            <strong>{{ excerpt.quote }}</strong>
            <small v-if="excerpt.reflection">{{ excerpt.reflection }}</small>
          </span>
          <span class="table-source">{{ excerptSource(excerpt) }}</span>
          <span
            class="table-tags"
            :title="excerpt.tags.map((tag) => `#${tag.name}`).join(' ')"
          >
            <span v-for="tag in excerpt.tags.slice(0, 2)" :key="tag.id" class="table-tag">
              #{{ tag.name }}
            </span>
            <span v-if="excerpt.tags.length > 2" class="table-tag-more">
              +{{ excerpt.tags.length - 2 }}
            </span>
          </span>
          <span class="item-meta">{{ formatDate(excerpt.createdAt) }}</span>
          <span class="row-actions" @click.stop>
            <button class="secondary-action" type="button" @click="startEditing(excerpt)">
              编辑
            </button>
            <button class="danger-action" type="button" @click="requestDeleteExcerpt(excerpt.id)">
              删除
            </button>
          </span>
        </div>

        <p v-if="props.excerpts.length === 0" class="empty-state table-empty">还没有摘抄。</p>
      </div>
    </div>

    <article v-else-if="viewMode === 'detail' && activeExcerpt" class="reader-page">
      <section class="reader-surface">
        <header class="reader-meta">
          <p class="source-line">{{ excerptSource(activeExcerpt) }}</p>
          <footer>
            <span>创建于 {{ new Date(activeExcerpt.createdAt).toLocaleString() }}</span>
            <span v-if="activeExcerpt.updatedAt !== activeExcerpt.createdAt">
              更新于 {{ new Date(activeExcerpt.updatedAt).toLocaleString() }}
            </span>
          </footer>
        </header>

        <blockquote>{{ activeExcerpt.quote }}</blockquote>

        <section class="reader-section">
          <h3>阅读笔记</h3>
          <p v-if="activeExcerpt.reflection" class="reflection">
            {{ activeExcerpt.reflection }}
          </p>
          <p v-else class="empty-state">还没有记录阅读笔记。</p>
        </section>

        <div v-if="activeExcerpt.tags.length > 0" class="tag-row">
          <span v-for="tag in activeExcerpt.tags" :key="tag.id" class="tag-pill">
            #{{ tag.name }}
          </span>
        </div>
      </section>
    </article>

    <form v-else-if="viewMode === 'create'" class="editor-page" @submit.prevent="submitCreate">
      <section class="editor-surface">
        <label>
          摘抄原文
          <textarea v-model="createDraft.quote" rows="10" placeholder="输入摘抄原文" />
        </label>
        <label>
          阅读笔记
          <textarea v-model="createDraft.reflection" rows="8" placeholder="写下此刻的理解" />
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
      </section>
      <div class="editor-actions">
        <button class="secondary-action" type="button" @click="cancelEditor">取消</button>
        <button class="primary-action" :disabled="!canSaveCreate" type="submit">保存</button>
      </div>
    </form>

    <form v-else-if="viewMode === 'edit'" class="editor-page" @submit.prevent="submitEdit">
      <section class="editor-surface">
        <label>
          摘抄原文
          <textarea v-model="editDraft.quote" rows="10" />
        </label>
        <label>
          阅读笔记
          <textarea v-model="editDraft.reflection" rows="8" />
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
      </section>
      <div class="editor-actions">
        <button class="secondary-action" type="button" @click="cancelEditor">取消</button>
        <button class="primary-action" :disabled="!canSaveEdit" type="submit">保存</button>
      </div>
    </form>
  </section>

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
        <CustomSelect v-model="filters.tagName" :options="tagFilterOptions" />
      </label>
      <div class="source-grid">
        <label>
          排序
          <CustomSelect v-model="filters.sortBy" :options="sortByOptions" />
        </label>
        <label>
          方向
          <CustomSelect v-model="filters.sortDirection" :options="sortDirectionOptions" />
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="resetFilters">清空</button>
        <button class="primary-action" type="submit">应用</button>
      </div>
    </form>
  </BaseModal>
</template>
