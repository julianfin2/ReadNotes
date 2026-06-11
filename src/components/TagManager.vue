<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "./BaseModal.vue";
import type { Excerpt } from "../types/excerpt";
import type { TagWithCount } from "../types/tag";

const tags = ref<TagWithCount[]>([]);
const excerpts = ref<Excerpt[]>([]);
const selectedTagName = ref("");
const selectedExcerptId = ref("");
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

const editingTags = reactive<
  Record<string, { name: string; parentId: string; color: string }>
>({});

const selectedTag = computed(() =>
  tags.value.find((tag) => tag.name === selectedTagName.value),
);

const selectedExcerpt = computed(() =>
  excerpts.value.find((excerpt) => excerpt.id === selectedExcerptId.value) || null,
);

watch(
  excerpts,
  (items) => {
    if (items.length === 0) {
      selectedExcerptId.value = "";
      return;
    }

    if (!items.some((excerpt) => excerpt.id === selectedExcerptId.value)) {
      selectedExcerptId.value = items[0].id;
    }
  },
  { immediate: true },
);

onMounted(async () => {
  await loadTags();
});

async function loadTags() {
  tags.value = await invoke<TagWithCount[]>("list_tags_with_counts");
  if (!selectedTagName.value && tags.value.length > 0) {
    selectedTagName.value = tags.value[0].name;
    await loadExcerptsForTag(selectedTagName.value);
  }
}

async function loadExcerptsForTag(tagName: string) {
  selectedTagName.value = tagName;
  excerpts.value = await invoke<Excerpt[]>("list_excerpts", {
    input: {
      tagName,
      sortBy: "createdAt",
      sortDirection: "desc",
    },
  });
}

function selectExcerpt(excerptId: string) {
  selectedExcerptId.value = excerptId;
}

async function createTag() {
  await runSaving(async () => {
    await invoke("create_tag", {
      input: {
        name: newTagName.value,
        parentId: newTagParentId.value || null,
        color: newTagColor.value || null,
      },
    });

    newTagName.value = "";
    newTagParentId.value = "";
    newTagColor.value = "";
    createModalOpen.value = false;
    await loadTags();
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
        color: draft.color || null,
      },
    });

    delete editingTags[tagId];
    editingTagId.value = "";
    editModalOpen.value = false;
    const previousSelection = selectedTagName.value;
    await loadTags();
    if (previousSelection) {
      const updated = tags.value.find((tag) => tag.id === tagId);
      if (updated) {
        await loadExcerptsForTag(updated.name);
      }
    }
  });
}

async function deleteTag(tagId: string) {
  await runSaving(async () => {
    await invoke("delete_tag", { id: tagId });
    const deletedSelected = selectedTag.value?.id === tagId;
    await loadTags();
    if (deletedSelected) {
      selectedTagName.value = tags.value[0]?.name || "";
      excerpts.value = [];
      if (selectedTagName.value) {
        await loadExcerptsForTag(selectedTagName.value);
      }
    }
  });
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
  return parent ? parent.name : "";
}

function selectableParents(tagId?: string) {
  return tags.value.filter((tag) => tag.id !== tagId);
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
  <section class="topic-panel desktop-side-pane">
    <header class="page-header">
      <div>
        <p class="eyebrow">Tags</p>
        <h2>标签</h2>
        <p class="subtle-text">{{ tags.length }} 个标签</p>
      </div>
      <button class="primary-action" type="button" @click="createModalOpen = true">
        新建标签
      </button>
    </header>

    <div class="topic-list">
      <div
        v-for="tag in tags"
        :key="tag.id"
        class="topic-selector topic-selector-block"
        :class="{ active: tag.name === selectedTagName }"
        role="button"
        tabindex="0"
        @click="loadExcerptsForTag(tag.name)"
        @keydown.enter="loadExcerptsForTag(tag.name)"
        @keydown.space.prevent="loadExcerptsForTag(tag.name)"
      >
        <button class="plain-selector" type="button" @click.stop="loadExcerptsForTag(tag.name)">
          <span>#{{ tag.name }}</span>
          <small>{{ tag.excerptCount }} 条</small>
        </button>
        <p v-if="tag.parentId" class="reflection">父标签：#{{ parentLabel(tag) }}</p>
      </div>
    </div>

    <p v-if="tags.length === 0" class="empty-state">还没有标签。</p>
    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
  </section>

  <section class="workspace-panel desktop-view">
    <header class="page-header">
      <div>
        <p class="eyebrow">Tagged excerpts</p>
        <h2>{{ selectedTagName ? `#${selectedTagName}` : "标签摘抄" }}</h2>
        <p v-if="selectedTag" class="subtle-text">
          {{ selectedTag.excerptCount }} 条摘抄
          <span v-if="selectedTag.parentId"> / 父标签 #{{ parentLabel(selectedTag) }}</span>
        </p>
      </div>
      <div v-if="selectedTag" class="toolbar">
        <button class="secondary-action" type="button" @click="startEditing(selectedTag)">
          编辑标签
        </button>
        <button class="danger-action" type="button" @click="requestDeleteTag(selectedTag.id)">
          删除标签
        </button>
      </div>
    </header>

    <div class="split-workspace tag-workspace-grid">
      <aside class="list-pane">
        <div class="list-scroll">
          <button
            v-for="excerpt in excerpts"
            :key="excerpt.id"
            class="excerpt-list-item"
            :class="{ active: excerpt.id === selectedExcerptId }"
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

          <p v-if="selectedTagName && excerpts.length === 0" class="empty-state">
            这个标签下还没有摘抄。
          </p>
          <p v-if="!selectedTagName" class="empty-state">选择一个标签查看摘抄。</p>
        </div>
      </aside>

      <article v-if="selectedExcerpt" class="detail-pane excerpt-detail-pane">
        <div class="detail-scroll">
          <header class="detail-header">
            <div>
              <p v-if="selectedExcerpt.bookTitle || selectedExcerpt.chapterTitle" class="source-line">
                <span v-if="selectedExcerpt.bookTitle">《{{ selectedExcerpt.bookTitle }}》</span>
                <span v-if="selectedExcerpt.bookTitle && selectedExcerpt.chapterTitle"> / </span>
                <span v-if="selectedExcerpt.chapterTitle">{{ selectedExcerpt.chapterTitle }}</span>
              </p>
              <footer>
                <span>重要性 {{ selectedExcerpt.importance }}</span>
                <span>{{ selectedExcerpt.status }}</span>
                <span>{{ new Date(selectedExcerpt.createdAt).toLocaleString() }}</span>
              </footer>
            </div>
          </header>

          <div class="reading-body">
            <blockquote>{{ selectedExcerpt.quote }}</blockquote>
            <p v-if="selectedExcerpt.reflection" class="reflection">
              {{ selectedExcerpt.reflection }}
            </p>
            <div v-if="selectedExcerpt.tags.length > 0" class="tag-row">
              <span v-for="tag in selectedExcerpt.tags" :key="tag.id" class="tag-pill">
                #{{ tag.name }}
              </span>
            </div>
          </div>
        </div>
      </article>

      <section v-else class="detail-pane empty-detail">
        <p class="empty-state">选择一条摘抄查看详情。</p>
      </section>
    </div>
  </section>

  <BaseModal :open="createModalOpen" title="新建标签" @close="createModalOpen = false">
    <form class="modal-form" @submit.prevent="createTag">
      <label>
        标签名
        <input v-model="newTagName" placeholder="例如：写作素材" />
      </label>
      <label>
        父标签
        <select v-model="newTagParentId">
          <option value="">无</option>
          <option v-for="tag in tags" :key="tag.id" :value="tag.id">#{{ tag.name }}</option>
        </select>
      </label>
      <label>
        颜色
        <input v-model="newTagColor" placeholder="#2e6f62，可选" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="createModalOpen = false">
          取消
        </button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
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
        <select v-model="editingTags[editingTagId].parentId">
          <option value="">无</option>
          <option
            v-for="parent in selectableParents(editingTagId)"
            :key="parent.id"
            :value="parent.id"
          >
            #{{ parent.name }}
          </option>
        </select>
      </label>
      <label>
        颜色
        <input v-model="editingTags[editingTagId].color" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelEditing(editingTagId)">
          取消
        </button>
        <button class="primary-action" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="deleteModalOpen" title="删除标签" @close="cancelDeleteTag">
    <div class="modal-form">
      <p class="reflection">删除标签会移除它和摘抄之间的关联。确认删除吗？</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelDeleteTag">取消</button>
        <button class="danger-action" type="button" @click="confirmDeleteTag">删除</button>
      </div>
    </div>
  </BaseModal>
</template>
