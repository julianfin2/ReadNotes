<script setup lang="ts">
import { computed, onMounted, reactive, ref, shallowRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "./BaseModal.vue";
import type { Excerpt } from "../types/excerpt";
import type { Topic, TopicExcerpt, TopicNode, TopicStatus } from "../types/topic";

const props = defineProps<{
  excerpts: Excerpt[];
}>();

const topics = ref<Topic[]>([]);
const topicNodes = ref<TopicNode[]>([]);
const topicExcerpts = ref<TopicExcerpt[]>([]);
const selectedTopicId = ref("");
const selectedNodeId = ref("");
const selectedTopicExcerptId = ref("");
const topicModalOpen = ref(false);
const nodeModalOpen = ref(false);
const addExcerptModalOpen = ref(false);
const editTopicModalOpen = ref(false);
const editNodeModalOpen = ref(false);
const confirmModalOpen = ref(false);
const editingTopicId = ref("");
const editingNodeId = ref("");
const editingTopicExcerptId = ref("");
const confirmTitle = ref("");
const confirmMessage = ref("");
const topicTitle = ref("");
const topicQuestion = ref("");
const nodeTitle = ref("");
const nodeSummary = ref("");
const nodeParentId = ref("");
const excerptIdToAdd = ref("");
const reason = ref("");
const topicReflection = ref("");
const errorMessage = ref("");
const isSaving = ref(false);

type TopicDraft = {
  title: string;
  description: string;
  researchQuestion: string;
  status: TopicStatus;
};

type NodeDraft = {
  parentId: string;
  title: string;
  summary: string;
  sortOrder: number;
};

type TopicExcerptDraft = {
  nodeId: string;
  reason: string;
  topicReflection: string;
  sortOrder: number;
};

type ConfirmAction = () => void | Promise<void>;

const editingTopics = reactive<Record<string, TopicDraft>>({});
const editingNodes = reactive<Record<string, NodeDraft>>({});
const editingTopicExcerpts = reactive<Record<string, TopicExcerptDraft>>({});
const confirmAction = shallowRef<ConfirmAction | null>(null);

const selectedTopic = computed(() =>
  topics.value.find((topic) => topic.id === selectedTopicId.value),
);

const selectedNode = computed(() =>
  topicNodes.value.find((node) => node.id === selectedNodeId.value),
);

const visibleTopicExcerpts = computed(() => {
  if (!selectedNodeId.value) {
    return topicExcerpts.value;
  }

  return topicExcerpts.value.filter(
    (topicExcerpt) => topicExcerpt.nodeId === selectedNodeId.value,
  );
});

const selectedTopicExcerpt = computed(() =>
  visibleTopicExcerpts.value.find(
    (topicExcerpt) => topicExcerpt.id === selectedTopicExcerptId.value,
  ) || null,
);

const isEditingSelectedTopicExcerpt = computed(() => {
  return Boolean(
    selectedTopicExcerpt.value &&
      selectedTopicExcerpt.value.id === editingTopicExcerptId.value &&
      editingTopicExcerpts[editingTopicExcerptId.value],
  );
});

const isTopicExcerptEditDirty = computed(() => {
  const topicExcerpt = selectedTopicExcerpt.value;

  if (!topicExcerpt || topicExcerpt.id !== editingTopicExcerptId.value) {
    return false;
  }

  const draft = editingTopicExcerpts[topicExcerpt.id];
  if (!draft) {
    return false;
  }

  return (
    draft.nodeId !== (topicExcerpt.nodeId || "") ||
    draft.reason !== (topicExcerpt.reason || "") ||
    draft.topicReflection !== (topicExcerpt.topicReflection || "") ||
    draft.sortOrder !== topicExcerpt.sortOrder
  );
});

const canSaveTopicExcerptEdit = computed(() => {
  return isTopicExcerptEditDirty.value && !isSaving.value;
});

onMounted(async () => {
  await loadTopics();
});

watch(selectedTopicId, async (topicId) => {
  selectedNodeId.value = "";
  selectedTopicExcerptId.value = "";
  topicNodes.value = [];
  topicExcerpts.value = [];
  clearEditingState();

  if (topicId) {
    await Promise.all([loadTopicNodes(topicId), loadTopicExcerpts(topicId)]);
  }
});

watch(
  visibleTopicExcerpts,
  (items) => {
    if (items.length === 0) {
      selectedTopicExcerptId.value = "";
      return;
    }

    if (!items.some((topicExcerpt) => topicExcerpt.id === selectedTopicExcerptId.value)) {
      selectedTopicExcerptId.value = items[0].id;
    }
  },
  { immediate: true },
);

async function loadTopics() {
  topics.value = await invoke<Topic[]>("list_topics");
  if (!selectedTopicId.value && topics.value.length > 0) {
    selectedTopicId.value = topics.value[0].id;
  }
}

async function loadTopicNodes(topicId: string) {
  topicNodes.value = await invoke<TopicNode[]>("list_topic_nodes", { topicId });
}

async function loadTopicExcerpts(topicId: string) {
  topicExcerpts.value = await invoke<TopicExcerpt[]>("list_topic_excerpts", {
    topicId,
  });
}

async function reloadSelectedTopic() {
  await loadTopics();
  if (selectedTopicId.value) {
    await Promise.all([
      loadTopicNodes(selectedTopicId.value),
      loadTopicExcerpts(selectedTopicId.value),
    ]);
  }
}

async function createTopic() {
  await runSaving(async () => {
    const topic = await invoke<Topic>("create_topic", {
      input: {
        title: topicTitle.value,
        researchQuestion: topicQuestion.value,
      },
    });

    topicTitle.value = "";
    topicQuestion.value = "";
    topicModalOpen.value = false;
    await loadTopics();
    selectedTopicId.value = topic.id;
  });
}

async function updateTopic(topicId: string) {
  const draft = editingTopics[topicId];
  if (!draft) {
    return;
  }

  await runSaving(async () => {
    await invoke<Topic>("update_topic", {
      input: {
        id: topicId,
        title: draft.title,
        description: draft.description,
        researchQuestion: draft.researchQuestion,
        status: draft.status,
      },
    });

    delete editingTopics[topicId];
    editingTopicId.value = "";
    editTopicModalOpen.value = false;
    await reloadSelectedTopic();
  });
}

async function deleteTopic(topicId: string) {
  await runSaving(async () => {
    await invoke("delete_topic", { id: topicId });
    if (selectedTopicId.value === topicId) {
      selectedTopicId.value = "";
    }
    await loadTopics();
    if (!selectedTopicId.value && topics.value.length > 0) {
      selectedTopicId.value = topics.value[0].id;
    }
  });
}

async function createTopicNode() {
  if (!selectedTopicId.value) {
    errorMessage.value = "请先选择主题";
    return;
  }

  await runSaving(async () => {
    await invoke<TopicNode>("create_topic_node", {
      input: {
        topicId: selectedTopicId.value,
        parentId: nodeParentId.value || null,
        title: nodeTitle.value,
        summary: nodeSummary.value,
      },
    });

    nodeTitle.value = "";
    nodeSummary.value = "";
    nodeParentId.value = "";
    nodeModalOpen.value = false;
    await loadTopicNodes(selectedTopicId.value);
  });
}

async function updateTopicNode(nodeId: string) {
  const draft = editingNodes[nodeId];
  if (!draft || !selectedTopicId.value) {
    return;
  }

  await runSaving(async () => {
    await invoke<TopicNode>("update_topic_node", {
      input: {
        id: nodeId,
        parentId: draft.parentId || null,
        title: draft.title,
        summary: draft.summary,
        sortOrder: draft.sortOrder,
      },
    });

    delete editingNodes[nodeId];
    editingNodeId.value = "";
    editNodeModalOpen.value = false;
    await loadTopicNodes(selectedTopicId.value);
  });
}

async function deleteTopicNode(nodeId: string) {
  if (!selectedTopicId.value) {
    return;
  }

  await runSaving(async () => {
    await invoke("delete_topic_node", { id: nodeId });
    if (selectedNodeId.value === nodeId) {
      selectedNodeId.value = "";
    }
    await Promise.all([
      loadTopicNodes(selectedTopicId.value),
      loadTopicExcerpts(selectedTopicId.value),
    ]);
  });
}

async function addExcerptToTopic() {
  if (!selectedTopicId.value) {
    errorMessage.value = "请先选择主题";
    return;
  }

  if (!excerptIdToAdd.value) {
    errorMessage.value = "请选择要收录的摘抄";
    return;
  }

  await runSaving(async () => {
    await invoke<TopicExcerpt>("add_excerpt_to_topic", {
      input: {
        topicId: selectedTopicId.value,
        excerptId: excerptIdToAdd.value,
        nodeId: selectedNodeId.value || null,
        reason: reason.value,
        topicReflection: topicReflection.value,
      },
    });

    excerptIdToAdd.value = "";
    reason.value = "";
    topicReflection.value = "";
    addExcerptModalOpen.value = false;
    await loadTopicExcerpts(selectedTopicId.value);
  });
}

async function updateTopicExcerpt(topicExcerptId: string) {
  const draft = editingTopicExcerpts[topicExcerptId];
  if (!draft || !selectedTopicId.value || !canSaveTopicExcerptEdit.value) {
    return;
  }

  await runSaving(async () => {
    await invoke<TopicExcerpt>("update_topic_excerpt", {
      input: {
        id: topicExcerptId,
        nodeId: draft.nodeId || null,
        reason: draft.reason,
        topicReflection: draft.topicReflection,
        sortOrder: draft.sortOrder,
      },
    });

    delete editingTopicExcerpts[topicExcerptId];
    editingTopicExcerptId.value = "";
    await loadTopicExcerpts(selectedTopicId.value);
  });
}

async function removeTopicExcerpt(topicExcerptId: string) {
  if (!selectedTopicId.value) {
    return;
  }

  await runSaving(async () => {
    await invoke("remove_excerpt_from_topic", { id: topicExcerptId });
    await loadTopicExcerpts(selectedTopicId.value);
  });
}

function startEditingTopic(topic: Topic) {
  editingTopicId.value = topic.id;
  editingTopics[topic.id] = {
    title: topic.title,
    description: topic.description || "",
    researchQuestion: topic.researchQuestion || "",
    status: topic.status,
  };
  editTopicModalOpen.value = true;
}

function startEditingNode(node: TopicNode) {
  editingNodeId.value = node.id;
  editingNodes[node.id] = {
    parentId: node.parentId || "",
    title: node.title,
    summary: node.summary || "",
    sortOrder: node.sortOrder,
  };
  editNodeModalOpen.value = true;
}

function startEditingTopicExcerpt(topicExcerpt: TopicExcerpt) {
  editingTopicExcerptId.value = topicExcerpt.id;
  editingTopicExcerpts[topicExcerpt.id] = {
    nodeId: topicExcerpt.nodeId || "",
    reason: topicExcerpt.reason || "",
    topicReflection: topicExcerpt.topicReflection || "",
    sortOrder: topicExcerpt.sortOrder,
  };
}

function cancelEditingTopic(topicId: string) {
  delete editingTopics[topicId];
  editingTopicId.value = "";
  editTopicModalOpen.value = false;
}

function cancelEditingNode(nodeId: string) {
  delete editingNodes[nodeId];
  editingNodeId.value = "";
  editNodeModalOpen.value = false;
}

function cancelEditingTopicExcerpt(topicExcerptId: string) {
  if (!topicExcerptId || topicExcerptId !== editingTopicExcerptId.value) {
    return;
  }

  discardTopicExcerptEditing();
}

function requestConfirmation(title: string, message: string, action: ConfirmAction) {
  confirmTitle.value = title;
  confirmMessage.value = message;
  confirmAction.value = action;
  confirmModalOpen.value = true;
}

function requestDeleteTopic(topic: Topic) {
  requestConfirmation(
    "删除主题",
    "删除主题会同时移除它的子主题和收录材料。确认删除吗？",
    () => deleteTopic(topic.id),
  );
}

function requestDeleteTopicNode(node: TopicNode) {
  requestConfirmation(
    "删除子主题",
    "删除子主题会影响当前主题下的材料归类。确认删除吗？",
    () => deleteTopicNode(node.id),
  );
}

function requestRemoveTopicExcerpt(topicExcerpt: TopicExcerpt) {
  requestConfirmation(
    "从主题移除",
    "这只会把摘抄从当前主题中移除，不会删除摘抄原文。确认移除吗？",
    () => removeTopicExcerpt(topicExcerpt.id),
  );
}

function handleTopicChange(event: Event) {
  const topicId = (event.target as HTMLSelectElement).value;

  if (topicId === selectedTopicId.value) {
    return;
  }

  if (!discardTopicExcerptEditing()) {
    (event.target as HTMLSelectElement).value = selectedTopicId.value;
    return;
  }

  selectedTopicId.value = topicId;
}

function selectTopicNode(nodeId: string) {
  if (nodeId === selectedNodeId.value) {
    return;
  }

  if (!discardTopicExcerptEditing()) {
    return;
  }

  selectedNodeId.value = nodeId;
}

function selectTopicExcerpt(topicExcerptId: string) {
  if (topicExcerptId === selectedTopicExcerptId.value) {
    return;
  }

  if (!discardTopicExcerptEditing()) {
    return;
  }

  selectedTopicExcerptId.value = topicExcerptId;
}

async function confirmDestructiveAction() {
  const action = confirmAction.value;
  if (!action) {
    return;
  }

  confirmModalOpen.value = false;
  confirmAction.value = null;
  await action();
}

function cancelConfirmation() {
  confirmModalOpen.value = false;
  confirmAction.value = null;
}

function clearEditingState() {
  for (const key of Object.keys(editingTopics)) {
    delete editingTopics[key];
  }
  for (const key of Object.keys(editingNodes)) {
    delete editingNodes[key];
  }
  for (const key of Object.keys(editingTopicExcerpts)) {
    delete editingTopicExcerpts[key];
  }
  editingTopicExcerptId.value = "";
}

function discardTopicExcerptEditing() {
  if (!editingTopicExcerptId.value) {
    return true;
  }

  if (isTopicExcerptEditDirty.value && !window.confirm("当前收录信息有未保存修改，确定放弃吗？")) {
    return false;
  }

  delete editingTopicExcerpts[editingTopicExcerptId.value];
  editingTopicExcerptId.value = "";
  return true;
}

function nodeLabel(node: TopicNode) {
  if (!node.parentId) {
    return node.title;
  }

  const parent = topicNodes.value.find((candidate) => candidate.id === node.parentId);
  return parent ? `${parent.title} / ${node.title}` : node.title;
}

function selectableParents(nodeId?: string) {
  return topicNodes.value.filter((node) => node.id !== nodeId);
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
  <section class="page-panel workspace-panel desktop-view topic-page">
    <header class="page-header">
      <div>
        <p class="eyebrow">Workspace</p>
        <h2>{{ selectedTopic?.title || "主题工作台" }}</h2>
        <p v-if="selectedTopic?.researchQuestion" class="subtle-text">
          {{ selectedTopic.researchQuestion }}
        </p>
        <p v-else class="subtle-text">{{ topics.length }} 个主题</p>
      </div>

      <div class="toolbar topic-toolbar">
        <select
          v-if="topics.length > 0"
          :value="selectedTopicId"
          class="topic-switcher"
          aria-label="切换主题"
          @change="handleTopicChange"
        >
          <option v-for="topic in topics" :key="topic.id" :value="topic.id">
            {{ topic.title }}
          </option>
        </select>
        <button class="primary-action" type="button" @click="topicModalOpen = true">
          新建主题
        </button>
        <button
          v-if="selectedTopic"
          class="secondary-action"
          type="button"
          @click="startEditingTopic(selectedTopic)"
        >
          编辑主题
        </button>
        <button
          v-if="selectedTopic"
          class="danger-action"
          type="button"
          @click="requestDeleteTopic(selectedTopic)"
        >
          删除主题
        </button>
      </div>
    </header>

    <div v-if="selectedTopic" class="topic-workspace-grid">
      <aside class="topic-context-pane">
        <section class="context-section">
          <div class="card-header">
            <h3>子主题</h3>
            <div class="inline-actions">
              <button
                v-if="selectedNode"
                class="secondary-action"
                type="button"
                @click="startEditingNode(selectedNode)"
              >
                编辑
              </button>
              <button
                v-if="selectedNode"
                class="danger-action"
                type="button"
                @click="requestDeleteTopicNode(selectedNode)"
              >
                删除
              </button>
              <button class="secondary-action" type="button" @click="nodeModalOpen = true">
                添加
              </button>
            </div>
          </div>

          <div class="node-list">
            <button
              class="node-selector"
              :class="{ active: selectedNodeId === '' }"
              @click="selectTopicNode('')"
            >
              全部摘抄
            </button>

            <div v-for="node in topicNodes" :key="node.id" class="node-editor">
              <button
                class="node-selector"
                :class="{ active: node.id === selectedNodeId }"
                @click="selectTopicNode(node.id)"
              >
                {{ nodeLabel(node) }}
              </button>
            </div>
          </div>

          <div v-if="selectedNode" class="context-actions">
            <p v-if="selectedNode.summary" class="subtle-text">{{ selectedNode.summary }}</p>
          </div>
        </section>

        <section class="context-section material-context-section">
          <div class="card-header">
            <div>
              <h3>材料</h3>
              <p class="subtle-text">{{ visibleTopicExcerpts.length }} 条材料</p>
            </div>
            <button class="primary-action" type="button" @click="addExcerptModalOpen = true">
              收录
            </button>
          </div>

          <p class="context-caption">{{ selectedNode?.title || "全部摘抄" }}</p>

          <div class="material-list-scroll">
            <button
              v-for="topicExcerpt in visibleTopicExcerpts"
              :key="topicExcerpt.id"
              class="excerpt-list-item"
              :class="{ active: topicExcerpt.id === selectedTopicExcerptId }"
              type="button"
              @click="selectTopicExcerpt(topicExcerpt.id)"
            >
              <span class="item-title">{{ topicExcerpt.excerpt.quote }}</span>
              <span
                v-if="topicExcerpt.excerpt.bookTitle || topicExcerpt.excerpt.chapterTitle"
                class="item-meta"
              >
                <span v-if="topicExcerpt.excerpt.bookTitle">
                  《{{ topicExcerpt.excerpt.bookTitle }}》
                </span>
                <span v-if="topicExcerpt.excerpt.bookTitle && topicExcerpt.excerpt.chapterTitle">
                  /
                </span>
                <span v-if="topicExcerpt.excerpt.chapterTitle">
                  {{ topicExcerpt.excerpt.chapterTitle }}
                </span>
              </span>
              <span class="item-meta">{{ new Date(topicExcerpt.addedAt).toLocaleDateString() }}</span>
            </button>
          </div>

          <p v-if="visibleTopicExcerpts.length === 0" class="empty-state">
            当前范围还没有收录摘抄。
          </p>
        </section>
      </aside>

      <article
        v-if="selectedTopicExcerpt"
        class="detail-pane excerpt-detail-pane topic-detail-pane document-detail-pane"
        :class="{ 'is-editing': isEditingSelectedTopicExcerpt }"
      >
        <form
          v-if="isEditingSelectedTopicExcerpt && editingTopicExcerpts[editingTopicExcerptId]"
          class="detail-document edit-document"
          @submit.prevent="updateTopicExcerpt(editingTopicExcerptId)"
        >
          <header class="detail-header document-header">
            <div>
              <p class="eyebrow">Editing</p>
              <h3>编辑收录</h3>
              <footer>
                <span>调整这条材料在当前主题中的归类、收录理由和主题理解</span>
              </footer>
            </div>
            <div class="action-row">
              <button
                class="secondary-action"
                type="button"
                @click="cancelEditingTopicExcerpt(editingTopicExcerptId)"
              >
                取消
              </button>
              <button
                class="primary-action"
                :disabled="!canSaveTopicExcerptEdit"
                type="submit"
              >
                保存
              </button>
            </div>
          </header>

          <div class="detail-scroll document-scroll">
            <div class="inline-editor-body topic-excerpt-editor">
              <section class="readonly-excerpt-preview">
                <p
                  v-if="
                    selectedTopicExcerpt.excerpt.bookTitle ||
                    selectedTopicExcerpt.excerpt.chapterTitle
                  "
                  class="source-line"
                >
                  <span v-if="selectedTopicExcerpt.excerpt.bookTitle">
                    《{{ selectedTopicExcerpt.excerpt.bookTitle }}》
                  </span>
                  <span
                    v-if="
                      selectedTopicExcerpt.excerpt.bookTitle &&
                      selectedTopicExcerpt.excerpt.chapterTitle
                    "
                  >
                    /
                  </span>
                  <span v-if="selectedTopicExcerpt.excerpt.chapterTitle">
                    {{ selectedTopicExcerpt.excerpt.chapterTitle }}
                  </span>
                </p>
                <blockquote>{{ selectedTopicExcerpt.excerpt.quote }}</blockquote>
              </section>

              <label>
                子主题
                <select v-model="editingTopicExcerpts[editingTopicExcerptId].nodeId">
                  <option value="">未分类</option>
                  <option v-for="node in topicNodes" :key="node.id" :value="node.id">
                    {{ nodeLabel(node) }}
                  </option>
                </select>
              </label>
              <label>
                收录理由
                <textarea
                  v-model="editingTopicExcerpts[editingTopicExcerptId].reason"
                  rows="5"
                />
              </label>
              <label>
                主题理解
                <textarea
                  v-model="editingTopicExcerpts[editingTopicExcerptId].topicReflection"
                  rows="7"
                />
              </label>
            </div>
          </div>
        </form>

        <div v-else class="detail-document">
          <header class="detail-header document-header">
            <div>
              <p
                v-if="
                  selectedTopicExcerpt.excerpt.bookTitle ||
                  selectedTopicExcerpt.excerpt.chapterTitle
                "
                class="source-line"
              >
                <span v-if="selectedTopicExcerpt.excerpt.bookTitle">
                  《{{ selectedTopicExcerpt.excerpt.bookTitle }}》
                </span>
                <span
                  v-if="
                    selectedTopicExcerpt.excerpt.bookTitle &&
                    selectedTopicExcerpt.excerpt.chapterTitle
                  "
                >
                  /
                </span>
                <span v-if="selectedTopicExcerpt.excerpt.chapterTitle">
                  {{ selectedTopicExcerpt.excerpt.chapterTitle }}
                </span>
              </p>
              <footer>
                <span>{{ new Date(selectedTopicExcerpt.addedAt).toLocaleString() }}</span>
              </footer>
            </div>
            <div class="action-row">
              <button
                class="secondary-action"
                type="button"
                title="编辑这条材料在当前主题中的子主题、收录理由和主题理解"
                @click="startEditingTopicExcerpt(selectedTopicExcerpt)"
              >
                编辑收录
              </button>
              <button
                class="danger-action"
                type="button"
                title="只从当前主题移除，不删除摘抄原文"
                @click="requestRemoveTopicExcerpt(selectedTopicExcerpt)"
              >
                移出主题
              </button>
            </div>
          </header>

          <div class="detail-scroll document-scroll">
            <div class="reading-body topic-reading-body document-body">
              <blockquote>{{ selectedTopicExcerpt.excerpt.quote }}</blockquote>
              <p v-if="selectedTopicExcerpt.reason" class="reflection">
                收录理由：{{ selectedTopicExcerpt.reason }}
              </p>
              <p v-if="selectedTopicExcerpt.topicReflection" class="reflection">
                主题理解：{{ selectedTopicExcerpt.topicReflection }}
              </p>
              <div v-if="selectedTopicExcerpt.excerpt.tags.length > 0" class="tag-row">
                <span
                  v-for="tag in selectedTopicExcerpt.excerpt.tags"
                  :key="tag.id"
                  class="tag-pill"
                >
                  #{{ tag.name }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </article>

      <section v-else class="detail-pane empty-detail">
        <p class="empty-state">选择一条材料查看详情。</p>
      </section>
    </div>

    <div v-else class="empty-detail topic-empty-state">
      <p class="empty-state">先创建一个主题。</p>
      <button class="primary-action" type="button" @click="topicModalOpen = true">
        新建主题
      </button>
    </div>

    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
  </section>

  <BaseModal :open="topicModalOpen" title="新建主题" @close="topicModalOpen = false">
    <form class="modal-form" @submit.prevent="createTopic">
      <label>
        主题标题
        <input v-model="topicTitle" placeholder="例如：现代人的焦虑来源" />
      </label>
      <label>
        研究问题
        <textarea v-model="topicQuestion" rows="4" placeholder="这个主题想回答什么？" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="topicModalOpen = false">取消</button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="nodeModalOpen" title="添加子主题" @close="nodeModalOpen = false">
    <form class="modal-form" @submit.prevent="createTopicNode">
      <label>
        子主题标题
        <input v-model="nodeTitle" placeholder="例如：比较心理" />
      </label>
      <label>
        父子主题
        <select v-model="nodeParentId">
          <option value="">无</option>
          <option v-for="node in topicNodes" :key="node.id" :value="node.id">
            {{ nodeLabel(node) }}
          </option>
        </select>
      </label>
      <label>
        摘要
        <textarea v-model="nodeSummary" rows="4" placeholder="这个子主题目前怎么理解？" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="nodeModalOpen = false">取消</button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="addExcerptModalOpen" title="收录摘抄" @close="addExcerptModalOpen = false">
    <form class="modal-form" @submit.prevent="addExcerptToTopic">
      <label>
        选择摘抄
        <select v-model="excerptIdToAdd">
          <option value="">请选择</option>
          <option v-for="excerpt in props.excerpts" :key="excerpt.id" :value="excerpt.id">
            {{ excerpt.quote.slice(0, 64) }}
          </option>
        </select>
      </label>
      <label>
        收录理由
        <textarea v-model="reason" rows="3" placeholder="为什么把它放进这个主题？" />
      </label>
      <label>
        主题内理解
        <textarea
          v-model="topicReflection"
          rows="5"
          placeholder="这条摘抄在当前主题下意味着什么？"
        />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="addExcerptModalOpen = false">
          取消
        </button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="editTopicModalOpen" title="编辑主题" @close="cancelEditingTopic(editingTopicId)">
    <form
      v-if="editingTopicId && editingTopics[editingTopicId]"
      class="modal-form"
      @submit.prevent="updateTopic(editingTopicId)"
    >
      <label>
        标题
        <input v-model="editingTopics[editingTopicId].title" />
      </label>
      <label>
        研究问题
        <textarea v-model="editingTopics[editingTopicId].researchQuestion" rows="4" />
      </label>
      <label>
        状态
        <select v-model="editingTopics[editingTopicId].status">
          <option value="collecting">collecting</option>
          <option value="organizing">organizing</option>
          <option value="drafting">drafting</option>
          <option value="finished">finished</option>
          <option value="paused">paused</option>
        </select>
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelEditingTopic(editingTopicId)">
          取消
        </button>
        <button class="primary-action" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="editNodeModalOpen" title="编辑子主题" @close="cancelEditingNode(editingNodeId)">
    <form
      v-if="editingNodeId && editingNodes[editingNodeId]"
      class="modal-form"
      @submit.prevent="updateTopicNode(editingNodeId)"
    >
      <label>
        标题
        <input v-model="editingNodes[editingNodeId].title" />
      </label>
      <label>
        父子主题
        <select v-model="editingNodes[editingNodeId].parentId">
          <option value="">无</option>
          <option
            v-for="parent in selectableParents(editingNodeId)"
            :key="parent.id"
            :value="parent.id"
          >
            {{ nodeLabel(parent) }}
          </option>
        </select>
      </label>
      <label>
        摘要
        <textarea v-model="editingNodes[editingNodeId].summary" rows="4" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelEditingNode(editingNodeId)">
          取消
        </button>
        <button class="primary-action" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="confirmModalOpen" :title="confirmTitle" @close="cancelConfirmation">
    <div class="modal-form">
      <p class="reflection">{{ confirmMessage }}</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelConfirmation">取消</button>
        <button class="danger-action" type="button" @click="confirmDestructiveAction">确认</button>
      </div>
    </div>
  </BaseModal>
</template>
