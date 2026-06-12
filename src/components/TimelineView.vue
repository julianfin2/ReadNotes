<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "./BaseModal.vue";
import CustomSelect from "./CustomSelect.vue";
import type { Excerpt } from "../types/excerpt";
import type { TimelineEntry } from "../types/note";
import type { Topic } from "../types/topic";

const props = defineProps<{
  excerpts: Excerpt[];
}>();

const topics = ref<Topic[]>([]);
const entries = ref<TimelineEntry[]>([]);
const noteModalOpen = ref(false);
const filterModalOpen = ref(false);
const targetKind = ref("excerpt");
const targetId = ref("");
const topicFilterId = ref("");
const excerptFilterId = ref("");
const content = ref("");
const errorMessage = ref("");
const isSaving = ref(false);

const targetOptions = computed(() => {
  if (targetKind.value === "topic") {
    return topics.value.map((topic) => ({
      id: topic.id,
      label: topic.title,
    }));
  }

  return props.excerpts.map((excerpt) => ({
    id: excerpt.id,
    label: excerpt.quote.slice(0, 64),
  }));
});

const targetKindOptions = [
  { value: "excerpt", label: "摘抄" },
  { value: "topic", label: "主题" },
];

const topicFilterOptions = computed(() => [
  { value: "", label: "全部主题" },
  ...topics.value.map((topic) => ({ value: topic.id, label: topic.title })),
]);

const excerptFilterOptions = computed(() => [
  { value: "", label: "全部摘抄" },
  ...props.excerpts.map((excerpt) => ({
    value: excerpt.id,
    label: excerpt.quote.slice(0, 48),
  })),
]);

const activeFilterCount = computed(() =>
  [topicFilterId.value, excerptFilterId.value].filter(Boolean).length,
);

const activeFilterLabels = computed(() => {
  const labels: string[] = [];
  const topic = topics.value.find((candidate) => candidate.id === topicFilterId.value);
  const excerpt = props.excerpts.find((candidate) => candidate.id === excerptFilterId.value);

  if (topic) {
    labels.push(`主题：${topic.title}`);
  }
  if (excerpt) {
    labels.push(`摘抄：${excerpt.quote.slice(0, 24)}`);
  }

  return labels;
});

onMounted(async () => {
  await Promise.all([loadTopics(), loadTimeline()]);
});

async function loadTopics() {
  topics.value = await invoke<Topic[]>("list_topics");
}

async function loadTimeline() {
  entries.value = await invoke<TimelineEntry[]>("list_timeline", {
    input: {
      topicId: topicFilterId.value || null,
      excerptId: excerptFilterId.value || null,
    },
  });
}

async function createNote() {
  if (!targetId.value) {
    errorMessage.value = "请选择笔记对象";
    return;
  }

  errorMessage.value = "";
  isSaving.value = true;

  try {
    await invoke("create_note", {
      input: {
        targetType: targetKind.value,
        targetId: targetId.value,
        content: content.value,
      },
    });
    content.value = "";
    noteModalOpen.value = false;
    await loadTimeline();
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    isSaving.value = false;
  }
}

function applyFilters() {
  filterModalOpen.value = false;
  loadTimeline();
}

function resetFilters() {
  topicFilterId.value = "";
  excerptFilterId.value = "";
  filterModalOpen.value = false;
  loadTimeline();
}

function kindLabel(kind: TimelineEntry["kind"]) {
  switch (kind) {
    case "excerptCreated":
      return "摘抄";
    case "topicExcerptAdded":
      return "收录";
    case "noteCreated":
      return "笔记";
  }
}
</script>

<template>
  <section class="page-panel workspace-panel desktop-view">
    <header class="page-header desktop-toolbar">
      <div>
        <p class="eyebrow">Timeline</p>
        <h2>时间线</h2>
        <p class="subtle-text">{{ entries.length }} 条记录</p>
      </div>

      <div class="toolbar">
        <button class="secondary-action" type="button" @click="filterModalOpen = true">
          筛选{{ activeFilterCount ? ` (${activeFilterCount})` : "" }}
        </button>
        <button class="primary-action" type="button" @click="noteModalOpen = true">
          追加笔记
        </button>
      </div>
    </header>

    <div v-if="activeFilterLabels.length > 0" class="filter-chip-row">
      <span v-for="label in activeFilterLabels" :key="label" class="filter-chip">
        {{ label }}
      </span>
      <button class="text-action" type="button" @click="resetFilters">清空筛选</button>
    </div>

    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>

    <div class="timeline-list">
      <article v-for="entry in entries" :key="`${entry.kind}-${entry.id}`" class="timeline-card">
        <div class="timeline-marker">{{ kindLabel(entry.kind) }}</div>
        <div class="timeline-content">
          <h3>{{ entry.title }}</h3>
          <p v-if="entry.content" class="reflection">{{ entry.content }}</p>
          <footer>
            <span>{{ new Date(entry.occurredAt).toLocaleString() }}</span>
          </footer>
        </div>
      </article>

      <p v-if="entries.length === 0" class="empty-state">还没有时间线记录。</p>
    </div>
  </section>

  <BaseModal :open="noteModalOpen" title="追加笔记" @close="noteModalOpen = false">
    <form class="modal-form" @submit.prevent="createNote">
      <label>
        对象类型
        <CustomSelect
          v-model="targetKind"
          :options="targetKindOptions"
          @change="targetId = ''"
        />
      </label>
      <label>
        对象
        <CustomSelect
          v-model="targetId"
          :options="[{ value: '', label: '请选择' }, ...targetOptions.map((option) => ({ value: option.id, label: option.label }))]"
        />
      </label>
      <label>
        笔记内容
        <textarea v-model="content" rows="7" placeholder="记录新的理解、问题或阶段性想法" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="noteModalOpen = false">取消</button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="filterModalOpen" title="筛选时间线" @close="filterModalOpen = false">
    <form class="modal-form" @submit.prevent="applyFilters">
      <label>
        主题筛选
        <CustomSelect v-model="topicFilterId" :options="topicFilterOptions" />
      </label>
      <label>
        摘抄筛选
        <CustomSelect v-model="excerptFilterId" :options="excerptFilterOptions" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="resetFilters">清空</button>
        <button class="primary-action" type="submit">应用</button>
      </div>
    </form>
  </BaseModal>
</template>
