<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, shallowRef, watch } from "vue";
import {
  ArrowLeft,
  Check,
  Pencil,
  Plus,
  RotateCcw,
  Save,
  SlidersHorizontal,
  Trash2,
  X,
} from "@lucide/vue";
import BaseModal from "./BaseModal.vue";
import CustomSelect from "./CustomSelect.vue";
import ResponsiveTagList from "./ResponsiveTagList.vue";
import type { Note, NoteFilters, UpdateNoteInput } from "../types/note";
import type { Tag } from "../types/tag";
import { deleteDraftPayload, getDraftPayload, saveDraftPayload } from "../utils/drafts";
import { formatDateOnly, formatDateTime } from "../utils/date";

const props = defineProps<{
  notes: Note[];
  tags: Tag[];
  filters: NoteFilters;
  isSaving: boolean;
}>();

const emit = defineEmits<{
  applyFilters: [filters: NoteFilters];
  createNote: [input: { content: string; tagNames: string[] }, onSaved?: () => void];
  updateNote: [input: UpdateNoteInput, onSaved?: () => void];
  deleteNote: [id: string];
}>();

const viewMode = ref<"list" | "detail" | "create" | "edit">("list");
const activeNoteId = ref("");
const deletingNoteId = ref("");
const filterModalOpen = ref(false);
const deleteModalOpen = ref(false);
const discardModalOpen = ref(false);
const restoreDraftModalOpen = ref(false);
const discardMessage = ref("");
const restoreDraftMessage = ref("");
const toolbarSearch = ref("");
const pendingEditorAction = shallowRef<(() => void | Promise<void>) | null>(null);
const restoreDraftKind = ref<"create" | "edit" | null>(null);
const pendingCreateDraft = ref<NoteCreateDraftPayload | null>(null);
const pendingEditDraft = ref<NoteEditDraftPayload | null>(null);
let createDraftSaveTimer: number | undefined;
let editDraftSaveTimer: number | undefined;

type NoteCreateDraftPayload = {
  content: string;
  tagInput: string;
};

type NoteEditDraftPayload = NoteCreateDraftPayload;

const NOTE_CREATE_DRAFT_TYPE = "noteCreate";
const NOTE_EDIT_DRAFT_TYPE = "note";
const ACTIVE_CREATE_DRAFT_ID = "active";

const createDraft = reactive({
  content: "",
  tagInput: "",
});

const appliedFilters = reactive<NoteFilters>({
  search: "",
  tagName: "",
  sortBy: "updatedAt",
  sortDirection: "desc",
});

const filterDraft = reactive<NoteFilters>({
  search: "",
  tagName: "",
  sortBy: "updatedAt",
  sortDirection: "desc",
});

const editDraft = reactive<UpdateNoteInput & { tagInput: string }>({
  id: "",
  content: "",
  tagNames: [],
  tagInput: "",
});

const activeNote = computed(() => props.notes.find((note) => note.id === activeNoteId.value) || null);
const editingNote = computed(() => props.notes.find((note) => note.id === editDraft.id) || null);

const pageTitle = computed(() => {
  if (viewMode.value === "create") return "新增笔记";
  if (viewMode.value === "edit") return "编辑笔记";
  if (viewMode.value === "detail") return "阅读笔记";
  return "笔记";
});

const isEditDirty = computed(() => {
  const note = editingNote.value;
  if (!note || note.id !== editDraft.id) return false;

  const noteTags = note.tags.map((tag) => tag.name).sort();
  const draftTags = parseTagInput(editDraft.tagInput).sort();

  return editDraft.content !== note.content || noteTags.join("\n") !== draftTags.join("\n");
});

const canSaveCreate = computed(() => {
  return createDraft.content.trim().length > 0 && !props.isSaving;
});

const canSaveEdit = computed(() => {
  return editDraft.content.trim().length > 0 && isEditDirty.value && !props.isSaving;
});

const tagFilterOptions = computed(() => [
  { value: "", label: "全部标签" },
  ...props.tags.map((tag) => ({ value: tag.name, label: `#${tag.name}` })),
]);

const activeFilterCount = computed(() => {
  return [
    appliedFilters.search,
    appliedFilters.tagName,
    appliedFilters.sortBy !== "updatedAt" ? appliedFilters.sortBy : "",
    appliedFilters.sortDirection !== "desc" ? appliedFilters.sortDirection : "",
  ].filter(Boolean).length;
});

const activeFilterLabels = computed(() => {
  const labels: string[] = [];

  if (appliedFilters.search) {
    labels.push(`搜索：${appliedFilters.search}`);
  }

  if (appliedFilters.tagName) {
    labels.push(`标签：#${appliedFilters.tagName}`);
  }
  if (appliedFilters.sortBy !== "updatedAt") {
    labels.push(`排序：${appliedFilters.sortBy === "createdAt" ? "创建时间" : "更新时间"}`);
  }
  if (appliedFilters.sortDirection !== "desc") {
    labels.push("升序");
  }

  return labels;
});

const sortByOptions = [
  { value: "updatedAt", label: "更新时间" },
  { value: "createdAt", label: "创建时间" },
];

const sortDirectionOptions = [
  { value: "desc", label: "降序" },
  { value: "asc", label: "升序" },
];

watch(
  () => props.filters,
  (filters) => {
    syncAppliedFilters(filters);
  },
  { immediate: true, deep: true },
);

watch(
  () => ({
    mode: viewMode.value,
    content: createDraft.content,
    tagInput: createDraft.tagInput,
  }),
  () => scheduleCreateDraftSave(),
);

watch(
  () => ({
    mode: viewMode.value,
    id: editDraft.id,
    content: editDraft.content,
    tagInput: editDraft.tagInput,
  }),
  () => scheduleEditDraftSave(),
);

onBeforeUnmount(() => {
  saveCreateDraftNow();
  saveEditDraftNow();
  if (createDraftSaveTimer) {
    window.clearTimeout(createDraftSaveTimer);
  }
  if (editDraftSaveTimer) {
    window.clearTimeout(editDraftSaveTimer);
  }
});

watch(
  () => props.notes,
  (notes) => {
    if (viewMode.value === "detail" && activeNoteId.value && !notes.some((note) => note.id === activeNoteId.value)) {
      forceGoToList();
    }
    if (viewMode.value === "edit" && editDraft.id && !notes.some((note) => note.id === editDraft.id)) {
      forceGoToList();
    }
  },
);

function applyToolbarSearch() {
  const nextFilters = {
    ...appliedFilters,
    search: toolbarSearch.value.trim(),
  };
  copyFilters(nextFilters, appliedFilters);
  emit("applyFilters", { ...nextFilters });
}

function openFilterModal() {
  copyFilters(appliedFilters, filterDraft);
  filterModalOpen.value = true;
}

function applyFilters() {
  const nextFilters = {
    search: filterDraft.search.trim(),
    tagName: filterDraft.tagName,
    sortBy: filterDraft.sortBy,
    sortDirection: filterDraft.sortDirection,
  };
  toolbarSearch.value = nextFilters.search;
  copyFilters(nextFilters, appliedFilters);
  emit("applyFilters", { ...nextFilters });
  filterModalOpen.value = false;
}

function resetFilterDraft() {
  copyFilters(createDefaultFilters(), filterDraft);
}

function resetAppliedFilters() {
  const nextFilters = createDefaultFilters();
  toolbarSearch.value = "";
  copyFilters(nextFilters, appliedFilters);
  copyFilters(nextFilters, filterDraft);
  emit("applyFilters", { ...nextFilters });
}

function syncAppliedFilters(filters: NoteFilters) {
  copyFilters(filters, appliedFilters);
  toolbarSearch.value = filters.search;
  if (!filterModalOpen.value) {
    copyFilters(filters, filterDraft);
  }
}

function copyFilters(source: NoteFilters, target: NoteFilters) {
  target.search = source.search;
  target.tagName = source.tagName;
  target.sortBy = source.sortBy;
  target.sortDirection = source.sortDirection;
}

function createDefaultFilters(): NoteFilters {
  return {
    search: "",
    tagName: "",
    sortBy: "updatedAt",
    sortDirection: "desc",
  };
}

function openDetail(note: Note) {
  activeNoteId.value = note.id;
  viewMode.value = "detail";
}

function startCreate() {
  runAfterDiscard(() => {
    createDraft.content = "";
    createDraft.tagInput = "";
    viewMode.value = "create";
    void checkCreateDraft();
  });
}

function saveCreate() {
  if (!canSaveCreate.value) return;

  emit(
    "createNote",
    {
      content: createDraft.content,
      tagNames: parseTagInput(createDraft.tagInput),
    },
    () => {
      createDraft.content = "";
      createDraft.tagInput = "";
      void deleteDraftPayload(NOTE_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
      forceGoToList();
    },
  );
}

function startEdit(note: Note) {
  runAfterDiscard(() => {
    editDraft.id = note.id;
    editDraft.content = note.content;
    editDraft.tagNames = note.tags.map((tag) => tag.name);
    editDraft.tagInput = note.tags.map((tag) => `#${tag.name}`).join(" ");
    viewMode.value = "edit";
    void checkEditDraft(note.id);
  });
}

function saveEdit() {
  if (!canSaveEdit.value) return;

  emit(
    "updateNote",
    {
      id: editDraft.id,
      content: editDraft.content,
      tagNames: parseTagInput(editDraft.tagInput),
    },
    () => {
      activeNoteId.value = editDraft.id;
      void deleteDraftPayload(NOTE_EDIT_DRAFT_TYPE, editDraft.id);
      viewMode.value = "detail";
    },
  );
}

function requestDelete(note: Note) {
  deletingNoteId.value = note.id;
  deleteModalOpen.value = true;
}

function confirmDelete() {
  if (!deletingNoteId.value) return;

  emit("deleteNote", deletingNoteId.value);
  deleteModalOpen.value = false;
  deletingNoteId.value = "";
  if (viewMode.value !== "list") {
    forceGoToList();
  }
}

function goToList() {
  runAfterDiscard(() => forceGoToList());
}

function forceGoToList() {
  viewMode.value = "list";
  activeNoteId.value = "";
  editDraft.id = "";
}

function runAfterDiscard(action: () => void) {
  if (viewMode.value === "create" && isCreateDraftDirty()) {
    discardMessage.value = "当前新增笔记有未保存内容，确定放弃吗？";
    pendingEditorAction.value = async () => {
      await deleteDraftPayload(NOTE_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
      action();
    };
    discardModalOpen.value = true;
    return;
  }

  if (viewMode.value === "edit" && isEditDirty.value) {
    discardMessage.value = "当前笔记有未保存修改，确定放弃吗？";
    pendingEditorAction.value = async () => {
      await deleteDraftPayload(NOTE_EDIT_DRAFT_TYPE, editDraft.id);
      action();
    };
    discardModalOpen.value = true;
    return;
  }

  action();
}

async function confirmDiscard() {
  const action = pendingEditorAction.value;
  discardModalOpen.value = false;
  pendingEditorAction.value = null;
  await action?.();
}

function isCreateDraftDirty() {
  return Boolean(createDraft.content.trim() || createDraft.tagInput.trim());
}

function createDraftPayload(): NoteCreateDraftPayload {
  return {
    content: createDraft.content,
    tagInput: createDraft.tagInput,
  };
}

function editDraftPayload(): NoteEditDraftPayload {
  return {
    content: editDraft.content,
    tagInput: editDraft.tagInput,
  };
}

function scheduleCreateDraftSave() {
  if (createDraftSaveTimer) {
    window.clearTimeout(createDraftSaveTimer);
    createDraftSaveTimer = undefined;
  }

  if (viewMode.value !== "create" || !isCreateDraftDirty()) {
    return;
  }

  createDraftSaveTimer = window.setTimeout(() => {
    createDraftSaveTimer = undefined;
    saveCreateDraftNow();
  }, 800);
}

function saveCreateDraftNow() {
  if (viewMode.value !== "create" || !isCreateDraftDirty()) {
    return;
  }

  void saveDraftPayload(NOTE_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID, createDraftPayload());
}

function scheduleEditDraftSave() {
  if (editDraftSaveTimer) {
    window.clearTimeout(editDraftSaveTimer);
    editDraftSaveTimer = undefined;
  }

  if (viewMode.value !== "edit" || !editDraft.id || !isEditDirty.value) {
    return;
  }

  editDraftSaveTimer = window.setTimeout(() => {
    editDraftSaveTimer = undefined;
    saveEditDraftNow();
  }, 800);
}

function saveEditDraftNow() {
  if (viewMode.value !== "edit" || !editDraft.id || !isEditDirty.value) {
    return;
  }

  void saveDraftPayload(NOTE_EDIT_DRAFT_TYPE, editDraft.id, editDraftPayload());
}

async function checkCreateDraft() {
  try {
    const draft = await getDraftPayload<NoteCreateDraftPayload>(
      NOTE_CREATE_DRAFT_TYPE,
      ACTIVE_CREATE_DRAFT_ID,
    );
    if (!draft || viewMode.value !== "create") {
      return;
    }

    if (JSON.stringify(createDraftPayload()) === JSON.stringify(draft.payload)) {
      await deleteDraftPayload(NOTE_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
      return;
    }

    pendingCreateDraft.value = draft.payload;
    restoreDraftKind.value = "create";
    restoreDraftMessage.value = "存在上次未保存的新建笔记内容，是否恢复草稿？";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingDraftRestore();
  }
}

async function checkEditDraft(noteId: string) {
  try {
    const draft = await getDraftPayload<NoteEditDraftPayload>(NOTE_EDIT_DRAFT_TYPE, noteId);
    if (!draft || viewMode.value !== "edit" || editDraft.id !== noteId) {
      return;
    }

    if (JSON.stringify(editDraftPayload()) === JSON.stringify(draft.payload)) {
      await deleteDraftPayload(NOTE_EDIT_DRAFT_TYPE, noteId);
      return;
    }

    pendingEditDraft.value = draft.payload;
    restoreDraftKind.value = "edit";
    restoreDraftMessage.value = "当前笔记存在上次未保存的编辑内容，是否恢复草稿？";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingDraftRestore();
  }
}

function restorePendingDraft() {
  if (restoreDraftKind.value === "create" && pendingCreateDraft.value) {
    createDraft.content = pendingCreateDraft.value.content;
    createDraft.tagInput = pendingCreateDraft.value.tagInput;
  }

  if (restoreDraftKind.value === "edit" && pendingEditDraft.value) {
    editDraft.content = pendingEditDraft.value.content;
    editDraft.tagInput = pendingEditDraft.value.tagInput;
  }

  clearPendingDraftRestore();
}

function discardPendingDraft() {
  if (restoreDraftKind.value === "create") {
    void deleteDraftPayload(NOTE_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
  }

  if (restoreDraftKind.value === "edit" && editDraft.id) {
    void deleteDraftPayload(NOTE_EDIT_DRAFT_TYPE, editDraft.id);
  }

  clearPendingDraftRestore();
}

function clearPendingDraftRestore() {
  restoreDraftModalOpen.value = false;
  restoreDraftKind.value = null;
  pendingCreateDraft.value = null;
  pendingEditDraft.value = null;
}

function parseTagInput(value: string) {
  return value
    .split(/[\s,，#]+/)
    .map((tag) => tag.trim())
    .filter(Boolean);
}

function noteSummary(note: Note) {
  const normalized = note.content.replace(/\s+/g, " ").trim();
  return normalized.slice(0, 36) || "未命名笔记";
}

function notePreview(note: Note) {
  const normalized = note.content.replace(/\s+/g, " ").trim();
  if (normalized.length <= 36) {
    return "";
  }

  return normalized.slice(36, 110);
}

function tagStyle(tag: Tag) {
  if (!tag.color) {
    return {};
  }

  return {
    "--tag-accent": tag.color,
    "--tag-background": toTagBackground(tag.color),
  };
}

function toTagBackground(color: string) {
  const normalized = color.trim();
  const hex = normalized.match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
  if (!hex) {
    return color;
  }

  const value = hex[1].length === 3
    ? hex[1].split("").map((part) => part + part).join("")
    : hex[1];
  const red = parseInt(value.slice(0, 2), 16);
  const green = parseInt(value.slice(2, 4), 16);
  const blue = parseInt(value.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, 0.14)`;
}
</script>

<template>
  <section class="page-panel workspace-panel desktop-view">
    <header class="page-header list-toolbar-header">
      <div class="page-title-block">
        <h2 class="page-title-line">
          {{ pageTitle }}
          <span v-if="viewMode === 'list'" class="count-badge">{{ notes.length }}</span>
        </h2>
      </div>

      <form
        v-if="viewMode === 'list'"
        class="toolbar list-toolbar"
        @submit.prevent="applyToolbarSearch"
      >
        <div v-if="activeFilterLabels.length > 0" class="filter-chip-row toolbar-filter-chips">
          <span v-for="label in activeFilterLabels" :key="label" class="filter-chip">
            {{ label }}
          </span>
          <button class="text-action" type="button" @click="resetAppliedFilters">
            <RotateCcw aria-hidden="true" />
            清空筛选
          </button>
        </div>
          <input
            v-model="toolbarSearch"
            class="toolbar-search"
            placeholder="搜索笔记或标签"
          />
          <button class="secondary-action" type="button" @click="openFilterModal">
            <SlidersHorizontal aria-hidden="true" />
            筛选{{ activeFilterCount ? ` (${activeFilterCount})` : "" }}
          </button>
        <button class="primary-action" type="button" @click="startCreate">
          <Plus aria-hidden="true" />
          新增笔记
        </button>
      </form>

      <div v-else-if="viewMode === 'detail' && activeNote" class="toolbar">
        <button class="secondary-action" type="button" @click="goToList">
          <ArrowLeft aria-hidden="true" />
          返回列表
        </button>
        <button class="secondary-action" type="button" @click="startEdit(activeNote)">
          <Pencil aria-hidden="true" />
          编辑
        </button>
        <button class="danger-action" type="button" @click="requestDelete(activeNote)">
          <Trash2 aria-hidden="true" />
          删除
        </button>
      </div>

      <div v-else class="toolbar">
        <button class="secondary-action" type="button" @click="goToList">
          <ArrowLeft aria-hidden="true" />
          返回列表
        </button>
      </div>
    </header>

    <div v-if="viewMode === 'list'" class="note-table">
      <div class="note-table-head note-table-grid">
        <span>笔记</span>
        <span>标签</span>
        <span>创建时间</span>
        <span>操作</span>
      </div>
      <div class="note-table-body">
        <button
          v-for="note in notes"
          :key="note.id"
          class="note-table-row note-table-grid"
          type="button"
          @click="openDetail(note)"
        >
          <span class="note-table-primary">
            <strong>{{ noteSummary(note) }}</strong>
            <small v-if="notePreview(note)">{{ notePreview(note) }}</small>
          </span>
          <span class="table-tags">
            <ResponsiveTagList :tags="note.tags" />
          </span>
          <span class="item-meta" :title="formatDateTime(note.createdAt)">
            {{ formatDateOnly(note.createdAt) }}
          </span>
          <span class="table-actions">
            <button class="secondary-action" type="button" @click.stop="startEdit(note)">
              <Pencil aria-hidden="true" />
              编辑
            </button>
            <button class="danger-action" type="button" @click.stop="requestDelete(note)">
              <Trash2 aria-hidden="true" />
              删除
            </button>
          </span>
        </button>
        <p v-if="notes.length === 0" class="empty-state table-empty">还没有笔记。</p>
      </div>
    </div>

    <article v-else-if="viewMode === 'detail' && activeNote" class="reader-page">
      <section class="reader-surface">
        <header class="reader-meta">
          <footer>
            <span :title="formatDateTime(activeNote.createdAt)">
              创建于 {{ formatDateTime(activeNote.createdAt) }}
            </span>
            <span
              v-if="activeNote.updatedAt !== activeNote.createdAt"
              :title="formatDateTime(activeNote.updatedAt)"
            >
              更新于 {{ formatDateTime(activeNote.updatedAt) }}
            </span>
          </footer>
        </header>

        <p class="note-content">{{ activeNote.content }}</p>

        <div v-if="activeNote.tags.length > 0" class="tag-row">
          <span
            v-for="tag in activeNote.tags"
            :key="tag.id"
            class="tag-pill"
            :style="tagStyle(tag)"
          >
            #{{ tag.name }}
          </span>
        </div>
      </section>
    </article>

    <form
      v-else-if="viewMode === 'create'"
      class="editor-page"
      @submit.prevent="saveCreate"
    >
      <section class="editor-surface">
        <label>
          笔记
          <textarea v-model="createDraft.content" rows="14" placeholder="记录一个想法、判断、问题或提纲" />
        </label>
        <label>
          标签
          <input v-model="createDraft.tagInput" placeholder="#主题 #概念 #写作素材" />
        </label>
      </section>
      <div class="editor-actions">
        <button class="secondary-action" type="button" @click="goToList">
          <X aria-hidden="true" />
          取消
        </button>
        <button class="primary-action" :disabled="!canSaveCreate" type="submit">
          <Save aria-hidden="true" />
          保存
        </button>
      </div>
    </form>

    <form v-else-if="viewMode === 'edit'" class="editor-page" @submit.prevent="saveEdit">
      <section class="editor-surface">
        <label>
          笔记
          <textarea v-model="editDraft.content" rows="14" />
        </label>
        <label>
          标签
          <input v-model="editDraft.tagInput" placeholder="#主题 #概念 #写作素材" />
        </label>
      </section>
      <div class="editor-actions">
        <button class="secondary-action" type="button" @click="goToList">
          <X aria-hidden="true" />
          取消
        </button>
        <button class="primary-action" :disabled="!canSaveEdit" type="submit">
          <Check aria-hidden="true" />
          保存
        </button>
      </div>
    </form>

    <BaseModal :open="deleteModalOpen" title="删除笔记" @close="deleteModalOpen = false">
      <div class="modal-form">
        <p>确定要删除这条笔记吗？它也会从已收录的主题中移除。</p>
        <div class="modal-actions">
          <button class="secondary-action" type="button" @click="deleteModalOpen = false">
            <X aria-hidden="true" />
            取消
          </button>
          <button class="danger-action" type="button" @click="confirmDelete">
            <Trash2 aria-hidden="true" />
            删除
          </button>
        </div>
      </div>
    </BaseModal>

    <BaseModal :open="filterModalOpen" title="筛选笔记" @close="filterModalOpen = false">
      <form class="modal-form" @submit.prevent="applyFilters">
        <label>
          搜索
          <input v-model="filterDraft.search" placeholder="搜索笔记或标签" />
        </label>
        <label>
          标签
          <CustomSelect v-model="filterDraft.tagName" :options="tagFilterOptions" />
        </label>
        <div class="source-grid">
          <label>
            排序
            <CustomSelect v-model="filterDraft.sortBy" :options="sortByOptions" />
          </label>
          <label>
            方向
            <CustomSelect v-model="filterDraft.sortDirection" :options="sortDirectionOptions" />
          </label>
        </div>
        <div class="modal-actions">
          <button class="secondary-action" type="button" @click="resetFilterDraft">
            <RotateCcw aria-hidden="true" />
            清空
          </button>
          <button class="primary-action" type="submit">
            <Check aria-hidden="true" />
            应用
          </button>
        </div>
      </form>
    </BaseModal>

    <BaseModal :open="discardModalOpen" title="放弃更改" @close="discardModalOpen = false">
      <div class="modal-form">
        <p class="reflection">{{ discardMessage }}</p>
        <div class="modal-actions">
          <button class="secondary-action" type="button" @click="discardModalOpen = false">
            <X aria-hidden="true" />
            继续编辑
          </button>
          <button class="danger-action" type="button" @click="confirmDiscard">
            <Trash2 aria-hidden="true" />
            放弃更改
          </button>
        </div>
      </div>
    </BaseModal>

    <BaseModal :open="restoreDraftModalOpen" title="发现草稿" @close="discardPendingDraft">
      <div class="modal-form">
        <p class="reflection">{{ restoreDraftMessage }}</p>
        <div class="modal-actions">
          <button class="secondary-action" type="button" @click="discardPendingDraft">
            <X aria-hidden="true" />
            不恢复
          </button>
          <button class="primary-action" type="button" @click="restorePendingDraft">
            <Check aria-hidden="true" />
            恢复草稿
          </button>
        </div>
      </div>
    </BaseModal>
  </section>
</template>
