<script setup lang="ts">
import { reactive } from "vue";
import type { Excerpt, ExcerptFilters, UpdateExcerptInput } from "../types/excerpt";
import type { Tag } from "../types/tag";

defineProps<{
  excerpts: Excerpt[];
  tags: Tag[];
}>();

const emit = defineEmits<{
  applyFilters: [filters: ExcerptFilters];
  archiveExcerpt: [id: string];
  deleteExcerpt: [id: string];
  updateExcerpt: [input: UpdateExcerptInput];
}>();

const filters = reactive<ExcerptFilters>({
  search: "",
  tagName: "",
  status: "",
  minImportance: null,
  sortBy: "createdAt",
  sortDirection: "desc",
});

function applyFilters() {
  emit("applyFilters", { ...filters });
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

const editingById = reactive<Record<string, UpdateExcerptInput & { tagInput: string }>>({});

function startEditing(excerpt: Excerpt) {
  editingById[excerpt.id] = {
    id: excerpt.id,
    quote: excerpt.quote,
    reflection: excerpt.reflection || "",
    sourceWorkId: excerpt.sourceWorkId || null,
    location: excerpt.location || "",
    importance: excerpt.importance,
    status: excerpt.status,
    tagNames: excerpt.tags.map((tag) => tag.name),
    tagInput: excerpt.tags.map((tag) => `#${tag.name}`).join(" "),
  };
}

function cancelEditing(id: string) {
  delete editingById[id];
}

function saveEditing(id: string) {
  const draft = editingById[id];
  if (!draft) {
    return;
  }

  emit("updateExcerpt", {
    id: draft.id,
    quote: draft.quote,
    reflection: draft.reflection,
    sourceWorkId: draft.sourceWorkId,
    location: draft.location,
    importance: draft.importance,
    status: draft.status,
    tagNames: parseTagInput(draft.tagInput),
  });
  delete editingById[id];
}

function parseTagInput(value: string) {
  return value
    .split(/[\s,，#]+/)
    .map((tag) => tag.trim())
    .filter(Boolean);
}
</script>

<template>
  <section class="library-panel">
    <div class="section-heading">
      <p class="eyebrow">Library</p>
      <h2>{{ excerpts.length }} 条摘抄</h2>
    </div>

    <form class="filter-bar" @submit.prevent="applyFilters">
      <label>
        搜索
        <input v-model="filters.search" placeholder="搜索原文或初始理解" />
      </label>

      <label>
        标签
        <select v-model="filters.tagName">
          <option value="">全部标签</option>
          <option v-for="tag in tags" :key="tag.id" :value="tag.name">
            #{{ tag.name }}
          </option>
        </select>
      </label>

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

      <label>
        方向
        <select v-model="filters.sortDirection">
          <option value="desc">降序</option>
          <option value="asc">升序</option>
        </select>
      </label>

      <button class="primary-action" type="submit">筛选</button>
      <button class="secondary-action" type="button" @click="resetFilters">清空</button>
    </form>

    <div class="excerpt-list">
      <article v-for="excerpt in excerpts" :key="excerpt.id" class="excerpt-card">
        <template v-if="editingById[excerpt.id]">
          <form class="edit-form" @submit.prevent="saveEditing(excerpt.id)">
            <label>
              原文
              <textarea v-model="editingById[excerpt.id].quote" rows="5" />
            </label>

            <label>
              初始理解
              <textarea v-model="editingById[excerpt.id].reflection" rows="4" />
            </label>

            <label>
              标签
              <input v-model="editingById[excerpt.id].tagInput" />
            </label>

            <div class="edit-grid">
              <label>
                位置
                <input v-model="editingById[excerpt.id].location" />
              </label>

              <label>
                重要性
                <input
                  v-model.number="editingById[excerpt.id].importance"
                  max="5"
                  min="1"
                  type="number"
                />
              </label>

              <label>
                状态
                <select v-model="editingById[excerpt.id].status">
                  <option value="inbox">inbox</option>
                  <option value="processed">processed</option>
                  <option value="archived">archived</option>
                </select>
              </label>
            </div>

            <div class="action-row">
              <button class="primary-action" type="submit">保存</button>
              <button class="secondary-action" type="button" @click="cancelEditing(excerpt.id)">
                取消
              </button>
            </div>
          </form>
        </template>

        <template v-else>
          <blockquote>{{ excerpt.quote }}</blockquote>
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
        </template>
      </article>

      <p v-if="excerpts.length === 0" class="empty-state">还没有摘抄。</p>
    </div>
  </section>
</template>
