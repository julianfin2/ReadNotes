<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Pencil, Plus, Save, Trash2, X } from "@lucide/vue";
import BaseModal from "./BaseModal.vue";
import CustomSelect from "./CustomSelect.vue";
import TagColorField from "./TagColorField.vue";
import type { TagWithCount } from "../types/tag";
import { formatDateOnly, formatDateTime } from "../utils/date";
import { restoreScrollPosition, saveScrollPosition } from "../utils/scrollMemory";

defineProps<{
  embedded?: boolean;
}>();

const emit = defineEmits<{
  tagsChanged: [];
}>();

const tags = ref<TagWithCount[]>([]);
const search = ref("");
const createModalOpen = ref(false);
const editModalOpen = ref(false);
const deleteModalOpen = ref(false);
const editingTagId = ref("");
const deletingTagId = ref("");
const newTagName = ref("");
const newTagParentId = ref("");
const newTagColor = ref("");
const errorMessage = ref("");
const isSaving = ref(false);
const listScrollEl = ref<HTMLElement | null>(null);
const LIST_SCROLL_KEY = "tags";

const editingTags = reactive<
  Record<string, { name: string; parentId: string; color: string }>
>({});

const filteredTags = computed(() => {
  const query = search.value.trim().toLowerCase();
  if (!query) {
    return tags.value;
  }

  return tags.value.filter((tag) => {
    return (
      tag.name.toLowerCase().includes(query) ||
      parentLabel(tag).toLowerCase().includes(query)
    );
  });
});

const parentTagOptions = computed(() => [
  { value: "", label: "无" },
  ...tags.value.map((tag) => ({ value: tag.id, label: `#${tag.name}` })),
]);

onMounted(async () => {
  await loadTags();
  await nextTick();
  restoreListScroll();
});

onBeforeUnmount(() => {
  rememberListScroll();
});

function rememberListScroll() {
  if (listScrollEl.value) {
    saveScrollPosition(LIST_SCROLL_KEY, listScrollEl.value.scrollTop);
  }
}

function restoreListScroll() {
  restoreScrollPosition(LIST_SCROLL_KEY, listScrollEl.value);
}

async function loadTags() {
  tags.value = await invoke<TagWithCount[]>("list_tags_with_counts");
}

async function createTag() {
  await runSaving(async () => {
    await invoke("create_tag", {
      input: {
        name: newTagName.value,
        parentId: newTagParentId.value || null,
        color: normalizeColor(newTagColor.value),
      },
    });

    newTagName.value = "";
    newTagParentId.value = "";
    newTagColor.value = "";
    createModalOpen.value = false;
    await refreshAfterMutation();
  });
}

async function updateTag(tagId: string) {
  const draft = editingTags[tagId];
  if (!draft) {
    return;
  }

  await runSaving(async () => {
    await invoke("update_tag", {
      input: {
        id: tagId,
        name: draft.name,
        parentId: draft.parentId || null,
        color: normalizeColor(draft.color),
      },
    });

    delete editingTags[tagId];
    editingTagId.value = "";
    editModalOpen.value = false;
    await refreshAfterMutation();
  });
}

async function deleteTag(tagId: string) {
  await runSaving(async () => {
    await invoke("delete_tag", { id: tagId });
    await refreshAfterMutation();
  });
}

async function refreshAfterMutation() {
  await loadTags();
  emit("tagsChanged");
}

function requestDeleteTag(tagId: string) {
  deletingTagId.value = tagId;
  deleteModalOpen.value = true;
}

async function confirmDeleteTag() {
  if (!deletingTagId.value) {
    return;
  }

  const tagId = deletingTagId.value;
  deletingTagId.value = "";
  deleteModalOpen.value = false;
  await deleteTag(tagId);
}

function cancelDeleteTag() {
  deletingTagId.value = "";
  deleteModalOpen.value = false;
}

function startEditing(tag: TagWithCount) {
  editingTagId.value = tag.id;
  editingTags[tag.id] = {
    name: tag.name,
    parentId: tag.parentId || "",
    color: tag.color || "",
  };
  editModalOpen.value = true;
}

function cancelEditing(tagId: string) {
  delete editingTags[tagId];
  editingTagId.value = "";
  editModalOpen.value = false;
}

function parentLabel(tag: TagWithCount) {
  const parent = tags.value.find((candidate) => candidate.id === tag.parentId);
  return parent ? `#${parent.name}` : "无";
}

function selectableParents(tagId?: string) {
  return tags.value.filter((tag) => tag.id !== tagId);
}

function parentTagEditOptions(tagId: string) {
  return [
    { value: "", label: "无" },
    ...selectableParents(tagId).map((tag) => ({ value: tag.id, label: `#${tag.name}` })),
  ];
}

function normalizeColor(color: string) {
  const trimmed = color.trim();
  return trimmed || null;
}

async function runSaving(task: () => Promise<void>) {
  errorMessage.value = "";
  isSaving.value = true;

  try {
    await task();
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <section class="page-panel workspace-panel desktop-view tag-page" :class="{ embedded }">
    <header class="page-header list-toolbar-header">
      <div class="page-title-block">
        <h2 class="page-title-line">
          <span>标签管理</span>
          <span class="count-badge">{{ tags.length }}</span>
        </h2>
      </div>

      <div class="toolbar list-toolbar">
        <input
          v-model="search"
          class="toolbar-search"
          placeholder="搜索标签或父标签"
          type="search"
        />
        <button class="primary-action" type="button" @click="createModalOpen = true">
          <Plus aria-hidden="true" />
          新建标签
        </button>
      </div>
    </header>

    <div class="tag-table-page">
      <div class="tag-table">
        <div class="tag-table-head">
          <span>标签</span>
          <span>颜色</span>
          <span>父标签</span>
          <span>材料数</span>
          <span>创建时间</span>
          <span>操作</span>
        </div>

        <div
          ref="listScrollEl"
          class="tag-table-body"
          @scroll="rememberListScroll"
        >
          <div v-for="tag in filteredTags" :key="tag.id" class="tag-table-row">
            <span class="tag-name-cell">#{{ tag.name }}</span>
            <span>
              <span
                class="tag-color-swatch"
                :style="{ backgroundColor: tag.color || '#e8eee6' }"
              />
              <span class="subtle-inline">{{ tag.color || "未设置" }}</span>
            </span>
            <span>{{ parentLabel(tag) }}</span>
            <span>{{ tag.excerptCount }}</span>
            <span :title="formatDateTime(tag.createdAt)">
              {{ formatDateOnly(tag.createdAt) }}
            </span>
            <span class="table-actions">
              <button class="secondary-action" type="button" @click="startEditing(tag)">
                <Pencil aria-hidden="true" />
                编辑
              </button>
              <button class="danger-action" type="button" @click="requestDeleteTag(tag.id)">
                <Trash2 aria-hidden="true" />
                删除
              </button>
            </span>
          </div>

          <p v-if="filteredTags.length === 0" class="empty-state table-empty">
            {{ search ? "没有匹配的标签。" : "先创建一个标签。" }}
          </p>
        </div>
      </div>
    </div>

    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
  </section>

  <BaseModal :open="createModalOpen" title="新建标签" @close="createModalOpen = false">
    <form class="modal-form" @submit.prevent="createTag">
      <label>
        标签名
        <input v-model="newTagName" placeholder="例如：写作素材" />
      </label>
      <label>
        父标签
        <CustomSelect v-model="newTagParentId" :options="parentTagOptions" />
      </label>
      <div class="form-field">
        <span class="field-label">颜色</span>
        <TagColorField v-model="newTagColor" />
      </div>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="createModalOpen = false">
          <X aria-hidden="true" />
          取消
        </button>
        <button class="primary-action" :disabled="isSaving" type="submit">
          <Save aria-hidden="true" />
          保存
        </button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="editModalOpen" title="编辑标签" @close="cancelEditing(editingTagId)">
    <form
      v-if="editingTagId && editingTags[editingTagId]"
      class="modal-form"
      @submit.prevent="updateTag(editingTagId)"
    >
      <label>
        标签名
        <input v-model="editingTags[editingTagId].name" />
      </label>
      <label>
        父标签
        <CustomSelect
          v-model="editingTags[editingTagId].parentId"
          :options="parentTagEditOptions(editingTagId)"
        />
      </label>
      <div class="form-field">
        <span class="field-label">颜色</span>
        <TagColorField v-model="editingTags[editingTagId].color" />
      </div>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelEditing(editingTagId)">
          <X aria-hidden="true" />
          取消
        </button>
        <button class="primary-action" type="submit">
          <Save aria-hidden="true" />
          保存
        </button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="deleteModalOpen" title="删除标签" @close="cancelDeleteTag">
    <div class="modal-form">
      <p class="reflection">删除标签会移除它和摘抄之间的关联。确认删除吗？</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelDeleteTag">
          <X aria-hidden="true" />
          取消
        </button>
        <button class="danger-action" type="button" @click="confirmDeleteTag">
          <Trash2 aria-hidden="true" />
          删除
        </button>
      </div>
    </div>
  </BaseModal>
</template>
