<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Excerpt } from "../types/excerpt";
import type { TagWithCount } from "../types/tag";

const tags = ref<TagWithCount[]>([]);
const excerpts = ref<Excerpt[]>([]);
const selectedTagName = ref("");
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

function startEditing(tag: TagWithCount) {
  editingTags[tag.id] = {
    name: tag.name,
    parentId: tag.parentId || "",
    color: tag.color || "",
  };
}

function cancelEditing(tagId: string) {
  delete editingTags[tagId];
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
  <section class="topic-panel">
    <div class="section-heading">
      <p class="eyebrow">Tags</p>
      <h2>标签</h2>
    </div>

    <form @submit.prevent="createTag">
      <label>
        标签名
        <input v-model="newTagName" placeholder="例如：写作素材" />
      </label>

      <label>
        父标签
        <select v-model="newTagParentId">
          <option value="">无</option>
          <option v-for="tag in tags" :key="tag.id" :value="tag.id">
            #{{ tag.name }}
          </option>
        </select>
      </label>

      <label>
        颜色
        <input v-model="newTagColor" placeholder="#2e6f62，可选" />
      </label>

      <button class="primary-action" :disabled="isSaving" type="submit">创建标签</button>
    </form>

    <div class="topic-list">
      <div
        v-for="tag in tags"
        :key="tag.id"
        class="topic-selector topic-selector-block"
        :class="{ active: tag.name === selectedTagName }"
      >
        <template v-if="editingTags[tag.id]">
          <form class="edit-form" @submit.prevent="updateTag(tag.id)">
            <label>
              标签名
              <input v-model="editingTags[tag.id].name" />
            </label>
            <label>
              父标签
              <select v-model="editingTags[tag.id].parentId">
                <option value="">无</option>
                <option
                  v-for="parent in selectableParents(tag.id)"
                  :key="parent.id"
                  :value="parent.id"
                >
                  #{{ parent.name }}
                </option>
              </select>
            </label>
            <label>
              颜色
              <input v-model="editingTags[tag.id].color" />
            </label>
            <div class="action-row">
              <button class="primary-action" type="submit">保存</button>
              <button class="secondary-action" type="button" @click="cancelEditing(tag.id)">
                取消
              </button>
            </div>
          </form>
        </template>

        <template v-else>
          <button class="plain-selector" type="button" @click="loadExcerptsForTag(tag.name)">
            <span>#{{ tag.name }}</span>
            <small>{{ tag.excerptCount }} 条</small>
          </button>
          <p v-if="tag.parentId" class="reflection">父标签：#{{ parentLabel(tag) }}</p>
          <div class="action-row">
            <button class="secondary-action" type="button" @click="startEditing(tag)">
              编辑
            </button>
            <button class="danger-action" type="button" @click="deleteTag(tag.id)">
              删除
            </button>
          </div>
        </template>
      </div>
    </div>

    <p v-if="tags.length === 0" class="empty-state">还没有标签。</p>
    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
  </section>

  <section class="workspace-panel">
    <div class="section-heading">
      <p class="eyebrow">Tagged excerpts</p>
      <h2>{{ selectedTagName ? `#${selectedTagName}` : "标签摘抄" }}</h2>
    </div>

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
      </article>

      <p v-if="selectedTagName && excerpts.length === 0" class="empty-state">
        这个标签下还没有摘抄。
      </p>
      <p v-if="!selectedTagName" class="empty-state">选择一个标签查看摘抄。</p>
    </div>
  </section>
</template>
