<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { ArrowLeft, Check, Pencil, Plus, Save, Search, Trash2, X } from "@lucide/vue";
import BaseModal from "./BaseModal.vue";
import ResponsiveTagList from "./ResponsiveTagList.vue";
import type { Note, NoteFilters, UpdateNoteInput } from "../types/note";
import type { Tag } from "../types/tag";
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
const deleteModalOpen = ref(false);
const searchDraft = ref("");

const createDraft = reactive({
  content: "",
  tagInput: "",
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

watch(
  () => props.filters,
  (filters) => {
    searchDraft.value = filters.search;
  },
  { immediate: true, deep: true },
);

watch(
  () => props.notes,
  (notes) => {
    if (viewMode.value === "detail" && activeNoteId.value && !notes.some((note) => note.id === activeNoteId.value)) {
      goToList();
    }
    if (viewMode.value === "edit" && editDraft.id && !notes.some((note) => note.id === editDraft.id)) {
      goToList();
    }
  },
);

function applySearch() {
  emit("applyFilters", {
    ...props.filters,
    search: searchDraft.value.trim(),
  });
}

function clearSearch() {
  searchDraft.value = "";
  emit("applyFilters", { search: "", tagName: props.filters.tagName });
}

function openDetail(note: Note) {
  activeNoteId.value = note.id;
  viewMode.value = "detail";
}

function startCreate() {
  createDraft.content = "";
  createDraft.tagInput = "";
  viewMode.value = "create";
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
      viewMode.value = "list";
    },
  );
}

function startEdit(note: Note) {
  editDraft.id = note.id;
  editDraft.content = note.content;
  editDraft.tagNames = note.tags.map((tag) => tag.name);
  editDraft.tagInput = note.tags.map((tag) => `#${tag.name}`).join(" ");
  viewMode.value = "edit";
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
    goToList();
  }
}

function goToList() {
  viewMode.value = "list";
  activeNoteId.value = "";
  editDraft.id = "";
}

function parseTagInput(value: string) {
  return value
    .split(/[\s,，#]+/)
    .map((tag) => tag.trim())
    .filter(Boolean);
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

      <div v-if="viewMode === 'list'" class="toolbar">
        <form class="toolbar" @submit.prevent="applySearch">
          <input
            v-model="searchDraft"
            class="toolbar-search"
            placeholder="搜索笔记或标签"
          />
          <button class="secondary-action" type="submit">
            <Search aria-hidden="true" />
            搜索
          </button>
          <button
            v-if="filters.search"
            class="secondary-action"
            type="button"
            @click="clearSearch"
          >
            <X aria-hidden="true" />
            清空
          </button>
        </form>
        <button class="primary-action" type="button" @click="startCreate">
          <Plus aria-hidden="true" />
          新增笔记
        </button>
      </div>

      <div v-else class="toolbar">
        <button class="secondary-action" type="button" @click="goToList">
          <ArrowLeft aria-hidden="true" />
          返回列表
        </button>
      </div>
    </header>

    <div v-if="viewMode === 'list'" class="excerpt-table">
      <div class="excerpt-table-head note-table-grid">
        <span>笔记</span>
        <span>标签</span>
        <span>创建时间</span>
        <span>操作</span>
      </div>
      <div class="excerpt-table-body">
        <button
          v-for="note in notes"
          :key="note.id"
          class="excerpt-table-row note-table-grid"
          type="button"
          @click="openDetail(note)"
        >
          <span class="table-primary">
            <strong>{{ note.content }}</strong>
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

    <article v-else-if="viewMode === 'detail' && activeNote" class="detail-pane document-detail-pane">
      <div class="detail-document">
        <header class="detail-header document-header">
          <div>
            <h3>笔记</h3>
            <footer>
              <span :title="formatDateTime(activeNote.createdAt)">
                创建于 {{ formatDateOnly(activeNote.createdAt) }}
              </span>
            </footer>
          </div>
          <div class="action-row">
            <button class="secondary-action" type="button" @click="startEdit(activeNote)">
              <Pencil aria-hidden="true" />
              编辑
            </button>
            <button class="danger-action" type="button" @click="requestDelete(activeNote)">
              <Trash2 aria-hidden="true" />
              删除
            </button>
          </div>
        </header>

        <div class="detail-scroll document-scroll">
          <div class="reading-body document-body">
            <p class="note-content">{{ activeNote.content }}</p>
            <div v-if="activeNote.tags.length > 0" class="tag-row">
              <span
                v-for="tag in activeNote.tags"
                :key="tag.id"
                class="tag-pill"
                :style="{ '--tag-accent': tag.color || undefined }"
              >
                #{{ tag.name }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </article>

    <form
      v-else-if="viewMode === 'create'"
      class="editor-page"
      @submit.prevent="saveCreate"
    >
      <section class="form-card">
        <label>
          笔记
          <textarea v-model="createDraft.content" rows="14" placeholder="记录一个想法、判断、问题或提纲" />
        </label>
        <label>
          标签
          <input v-model="createDraft.tagInput" placeholder="#主题 #概念 #写作素材" />
        </label>
      </section>
      <div class="editor-footer">
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
      <section class="form-card">
        <label>
          笔记
          <textarea v-model="editDraft.content" rows="14" />
        </label>
        <label>
          标签
          <input v-model="editDraft.tagInput" placeholder="#主题 #概念 #写作素材" />
        </label>
      </section>
      <div class="editor-footer">
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
  </section>
</template>
