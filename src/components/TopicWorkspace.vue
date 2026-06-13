<script setup lang="ts">
import { computed, onMounted, reactive, ref, shallowRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "./BaseModal.vue";
import CustomSelect from "./CustomSelect.vue";
import type { Excerpt } from "../types/excerpt";
import type { Tag } from "../types/tag";
import type { Topic, TopicExcerpt, TopicNode, TopicStatus } from "../types/topic";
import { formatDateOnly, formatDateTime } from "../utils/date";

const props = defineProps<{
  excerpts: Excerpt[];
}>();

const topics = ref<Topic[]>([]);
const topicNodes = ref<TopicNode[]>([]);
const topicExcerpts = ref<TopicExcerpt[]>([]);
const viewMode = ref<"list" | "create" | "edit" | "workspace">("list");
const selectedTopicId = ref("");
const selectedNodeId = ref("");
const selectedTopicExcerptId = ref("");
const nodeModalOpen = ref(false);
const addExcerptModalOpen = ref(false);
const editNodeModalOpen = ref(false);
const confirmModalOpen = ref(false);
const editingTopicId = ref("");
const editingNodeId = ref("");
const editingTopicExcerptId = ref("");
const confirmTitle = ref("");
const confirmMessage = ref("");
const confirmActionLabel = ref("确认");
const topicTitle = ref("");
const topicQuestion = ref("");
const nodeTitle = ref("");
const nodeSummary = ref("");
const nodeParentId = ref("");
const excerptIdToAdd = ref("");
const excerptSearch = ref("");
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

const pageTitle = computed(() => {
  if (viewMode.value === "create") {
    return "新建主题";
  }

  if (viewMode.value === "edit") {
    return "编辑主题";
  }

  if (viewMode.value === "workspace") {
    return selectedTopic.value?.title || "主题工作台";
  }

  return "主题";
});

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

const isCreateTopicDirty = computed(() => {
  return Boolean(topicTitle.value.trim() || topicQuestion.value.trim());
});

const isTopicEditDirty = computed(() => {
  if (viewMode.value !== "edit" || !editingTopicId.value) {
    return false;
  }

  return isTopicDraftDirty(editingTopicId.value);
});

const canSaveTopicCreate = computed(() => {
  return topicTitle.value.trim().length > 0 && !isSaving.value;
});

const canSaveTopicEdit = computed(() => {
  return (
    Boolean(editingTopicId.value && editingTopics[editingTopicId.value]?.title.trim()) &&
    isTopicEditDirty.value &&
    !isSaving.value
  );
});

const topicStatusOptions = [
  { value: "collecting", label: "收集中" },
  { value: "organizing", label: "整理中" },
  { value: "drafting", label: "写作中" },
  { value: "finished", label: "已完成" },
  { value: "paused", label: "已暂停" },
];

const topicNodeOptions = computed(() => [
  { value: "", label: "未分类" },
  ...topicNodes.value.map((node) => ({ value: node.id, label: nodeLabel(node) })),
]);

const nodeParentOptions = computed(() => [
  { value: "", label: "无" },
  ...topicNodes.value.map((node) => ({ value: node.id, label: nodeLabel(node) })),
]);

const availableExcerptsToCollect = computed(() => {
  const collectedExcerptIds = new Set(
    topicExcerpts.value.map((topicExcerpt) => topicExcerpt.excerptId),
  );

  return props.excerpts.filter((excerpt) => !collectedExcerptIds.has(excerpt.id));
});

const filteredExcerptsToCollect = computed(() => {
  const query = excerptSearch.value.trim().toLowerCase();

  if (!query) {
    return availableExcerptsToCollect.value;
  }

  return availableExcerptsToCollect.value.filter((excerpt) => {
    const searchableText = [
      excerpt.quote,
      excerpt.reflection || "",
      excerpt.bookTitle || "",
      excerpt.chapterTitle || "",
      excerpt.tags.map((tag) => tag.name).join(" "),
    ]
      .join(" ")
      .toLowerCase();

    return searchableText.includes(query);
  });
});

const excerptToAdd = computed(() => {
  return props.excerpts.find((excerpt) => excerpt.id === excerptIdToAdd.value) || null;
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
  if (!canSaveTopicCreate.value) {
    return;
  }

  await runSaving(async () => {
    await invoke<Topic>("create_topic", {
      input: {
        title: topicTitle.value,
        researchQuestion: topicQuestion.value,
      },
    });

    topicTitle.value = "";
    topicQuestion.value = "";
    await loadTopics();
    viewMode.value = "list";
  });
}

async function updateTopic(topicId: string) {
  const draft = editingTopics[topicId];
  if (!draft || !canSaveTopicEdit.value) {
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
    await reloadSelectedTopic();
    viewMode.value = "list";
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
    if (viewMode.value !== "list") {
      viewMode.value = "list";
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
    excerptSearch.value = "";
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
  runAfterTopicExcerptDiscard(() => {
    editingTopicId.value = topic.id;
    editingTopics[topic.id] = {
      title: topic.title,
      description: topic.description || "",
      researchQuestion: topic.researchQuestion || "",
      status: topic.status,
    };
    viewMode.value = "edit";
  });
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

function cancelEditingTopic() {
  returnToTopicList();
}

function startCreatingTopic() {
  topicTitle.value = "";
  topicQuestion.value = "";
  viewMode.value = "create";
}

function openTopicWorkspace(topic: Topic) {
  runAfterTopicExcerptDiscard(() => {
    selectedTopicId.value = topic.id;
    viewMode.value = "workspace";
  });
}

function returnToTopicList() {
  runAfterTopicEditorDiscard(() => {
    runAfterTopicExcerptDiscard(() => {
      clearEditingState();
      topicTitle.value = "";
      topicQuestion.value = "";
      viewMode.value = "list";
    });
  });
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

  runAfterTopicExcerptDiscard(() => {});
}

function requestConfirmation(
  title: string,
  message: string,
  action: ConfirmAction,
  actionLabel = "确认",
) {
  confirmTitle.value = title;
  confirmMessage.value = message;
  confirmActionLabel.value = actionLabel;
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

function openAddExcerptModal() {
  excerptIdToAdd.value = "";
  excerptSearch.value = "";
  reason.value = "";
  topicReflection.value = "";
  addExcerptModalOpen.value = true;
}

function closeAddExcerptModal() {
  addExcerptModalOpen.value = false;
  excerptIdToAdd.value = "";
  excerptSearch.value = "";
  reason.value = "";
  topicReflection.value = "";
}

function selectTopicNode(nodeId: string) {
  if (nodeId === selectedNodeId.value) {
    return;
  }

  runAfterTopicExcerptDiscard(() => {
    selectedNodeId.value = nodeId;
  });
}

function selectTopicExcerpt(topicExcerptId: string) {
  if (topicExcerptId === selectedTopicExcerptId.value) {
    return;
  }

  runAfterTopicExcerptDiscard(() => {
    selectedTopicExcerptId.value = topicExcerptId;
  });
}

async function confirmDestructiveAction() {
  const action = confirmAction.value;
  if (!action) {
    return;
  }

  confirmModalOpen.value = false;
  confirmAction.value = null;
  confirmActionLabel.value = "确认";
  await action();
}

function cancelConfirmation() {
  confirmModalOpen.value = false;
  confirmAction.value = null;
  confirmActionLabel.value = "确认";
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
  editingTopicId.value = "";
  editingNodeId.value = "";
  editingTopicExcerptId.value = "";
}

function clearTopicExcerptEditing() {
  if (!editingTopicExcerptId.value) {
    return;
  }

  delete editingTopicExcerpts[editingTopicExcerptId.value];
  editingTopicExcerptId.value = "";
}

function isTopicDraftDirty(topicId: string) {
  const topic = topics.value.find((candidate) => candidate.id === topicId);
  const draft = editingTopics[topicId];

  if (!topic || !draft) {
    return false;
  }

  return (
    draft.title !== topic.title ||
    draft.description !== (topic.description || "") ||
    draft.researchQuestion !== (topic.researchQuestion || "") ||
    draft.status !== topic.status
  );
}

function topicEditorDiscardMessage() {
  if (viewMode.value === "create" && isCreateTopicDirty.value) {
    return "当前新建主题还没有保存，确定放弃这些修改并离开吗？";
  }

  if (viewMode.value === "edit" && isTopicEditDirty.value) {
    return "当前主题有未保存修改，确定放弃吗？";
  }

  return "";
}

function runAfterTopicEditorDiscard(action: ConfirmAction) {
  const message = topicEditorDiscardMessage();

  if (!message) {
    void action();
    return;
  }

  requestConfirmation("放弃更改", message, action, "放弃更改");
}

function runAfterTopicExcerptDiscard(action: ConfirmAction) {
  if (!editingTopicExcerptId.value) {
    void action();
    return;
  }

  if (!isTopicExcerptEditDirty.value) {
    clearTopicExcerptEditing();
    void action();
    return;
  }

  requestConfirmation(
    "放弃更改",
    "当前收录信息有未保存修改，确定放弃吗？",
    async () => {
      clearTopicExcerptEditing();
      await action();
    },
    "放弃更改",
  );
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

function nodeParentEditOptions(nodeId: string) {
  return [
    { value: "", label: "无" },
    ...selectableParents(nodeId).map((node) => ({ value: node.id, label: nodeLabel(node) })),
  ];
}

function excerptSourceLabel(excerpt: Excerpt) {
  if (excerpt.bookTitle && excerpt.chapterTitle) {
    return `《${excerpt.bookTitle}》 / ${excerpt.chapterTitle}`;
  }

  if (excerpt.bookTitle) {
    return `《${excerpt.bookTitle}》`;
  }

  return excerpt.chapterTitle || "未记录书籍与章节";
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
    return normalized;
  }

  const value = hex[1].length === 3
    ? hex[1].split("").map((part) => part + part).join("")
    : hex[1];
  const red = Number.parseInt(value.slice(0, 2), 16);
  const green = Number.parseInt(value.slice(2, 4), 16);
  const blue = Number.parseInt(value.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, 0.14)`;
}

function topicStatusLabel(status: TopicStatus) {
  const labels: Record<TopicStatus, string> = {
    collecting: "收集中",
    organizing: "整理中",
    drafting: "写作中",
    finished: "已完成",
    paused: "已暂停",
  };

  return labels[status];
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
    <header class="page-header" :class="{ 'list-toolbar-header': viewMode === 'list' }">
      <div class="page-title-block">
        <h2>{{ pageTitle }}</h2>
        <p v-if="viewMode === 'workspace' && selectedTopic?.researchQuestion" class="subtle-text">
          {{ selectedTopic.researchQuestion }}
        </p>
        <p v-else-if="viewMode === 'workspace'" class="subtle-text">管理当前主题的子主题和材料</p>
        <p v-else-if="viewMode === 'list'" class="subtle-text">{{ topics.length }} 个主题</p>
      </div>

      <div v-if="viewMode === 'list'" class="toolbar topic-toolbar">
        <button class="primary-action" type="button" @click="startCreatingTopic">
          新建主题
        </button>
      </div>

      <div v-else-if="viewMode === 'workspace'" class="toolbar topic-toolbar">
        <button class="secondary-action" type="button" @click="returnToTopicList">
          返回列表
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

      <div v-else class="toolbar topic-toolbar">
        <button class="secondary-action" type="button" @click="returnToTopicList">
          返回列表
        </button>
      </div>
    </header>

    <div v-if="viewMode === 'list'" class="table-page">
      <div class="topic-table">
        <div class="topic-table-head">
          <span>主题</span>
          <span>研究问题</span>
          <span>状态</span>
          <span>更新时间</span>
          <span>操作</span>
        </div>

        <div class="topic-table-body">
          <div
            v-for="topic in topics"
            :key="topic.id"
            class="topic-table-row"
            role="button"
            tabindex="0"
            @click="openTopicWorkspace(topic)"
            @keydown.enter="openTopicWorkspace(topic)"
          >
            <span class="table-quote">
              <strong>{{ topic.title }}</strong>
              <small v-if="topic.description">{{ topic.description }}</small>
            </span>
            <span class="table-source">{{ topic.researchQuestion || "未记录" }}</span>
            <span class="topic-status">{{ topicStatusLabel(topic.status) }}</span>
            <span class="item-meta" :title="formatDateTime(topic.updatedAt)">
              {{ formatDateOnly(topic.updatedAt) }}
            </span>
            <span class="row-actions" @click.stop>
              <button class="secondary-action" type="button" @click="openTopicWorkspace(topic)">
                打开
              </button>
              <button class="secondary-action" type="button" @click="startEditingTopic(topic)">
                编辑
              </button>
              <button class="danger-action" type="button" @click="requestDeleteTopic(topic)">
                删除
              </button>
            </span>
          </div>

          <p v-if="topics.length === 0" class="empty-state table-empty">还没有主题。</p>
        </div>
      </div>
    </div>

    <form v-else-if="viewMode === 'create'" class="editor-page" @submit.prevent="createTopic">
      <section class="editor-surface">
        <label>
          主题标题
          <input v-model="topicTitle" placeholder="例如：现代人的焦虑来源" />
        </label>
        <label>
          研究问题
          <textarea v-model="topicQuestion" rows="5" placeholder="这个主题想回答什么？" />
        </label>
      </section>
      <div class="editor-actions">
        <button class="secondary-action" type="button" @click="returnToTopicList">取消</button>
        <button class="primary-action" :disabled="!canSaveTopicCreate" type="submit">
          保存
        </button>
      </div>
    </form>

    <form
      v-else-if="viewMode === 'edit' && editingTopicId && editingTopics[editingTopicId]"
      class="editor-page"
      @submit.prevent="updateTopic(editingTopicId)"
    >
      <section class="editor-surface">
        <label>
          标题
          <input v-model="editingTopics[editingTopicId].title" />
        </label>
        <label>
          研究问题
          <textarea v-model="editingTopics[editingTopicId].researchQuestion" rows="5" />
        </label>
        <label>
          状态
          <CustomSelect
            v-model="editingTopics[editingTopicId].status"
            :options="topicStatusOptions"
          />
        </label>
      </section>
      <div class="editor-actions">
        <button class="secondary-action" type="button" @click="cancelEditingTopic">
          取消
        </button>
        <button class="primary-action" :disabled="!canSaveTopicEdit" type="submit">保存</button>
      </div>
    </form>

    <div v-else-if="viewMode === 'workspace' && selectedTopic" class="topic-workspace">
      <section class="topic-tabs-bar">
        <div class="topic-tabs">
          <button
            class="topic-tab"
            :class="{ active: selectedNodeId === '' }"
            type="button"
            @click="selectTopicNode('')"
          >
            全部摘抄
          </button>
          <button
            v-for="node in topicNodes"
            :key="node.id"
            class="topic-tab"
            :class="{ active: node.id === selectedNodeId }"
            type="button"
            @click="selectTopicNode(node.id)"
          >
            {{ nodeLabel(node) }}
          </button>
        </div>

        <div class="topic-tab-actions">
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
            添加子主题
          </button>
        </div>
      </section>

      <p v-if="selectedNode?.summary" class="topic-node-summary">{{ selectedNode.summary }}</p>

      <div class="topic-workspace-grid">
        <aside class="topic-material-pane">
          <section class="context-section material-context-section">
            <div class="card-header">
              <div>
                <h3>材料</h3>
                <p class="subtle-text">{{ visibleTopicExcerpts.length }} 条材料</p>
              </div>
              <button class="primary-action" type="button" @click="openAddExcerptModal">
                收录
              </button>
            </div>

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
                <span class="item-meta">
                  {{ formatDateOnly(topicExcerpt.addedAt) }}
                </span>
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
              <h3>编辑收录</h3>
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
                <CustomSelect
                  v-model="editingTopicExcerpts[editingTopicExcerptId].nodeId"
                  :options="topicNodeOptions"
                />
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
                <span>{{ formatDateTime(selectedTopicExcerpt.addedAt) }}</span>
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
                  :style="tagStyle(tag)"
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
    </div>

    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
  </section>

  <BaseModal :open="nodeModalOpen" title="添加子主题" @close="nodeModalOpen = false">
    <form class="modal-form" @submit.prevent="createTopicNode">
      <label>
        子主题标题
        <input v-model="nodeTitle" placeholder="例如：比较心理" />
      </label>
      <label>
        父子主题
        <CustomSelect v-model="nodeParentId" :options="nodeParentOptions" />
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

  <BaseModal :open="addExcerptModalOpen" title="收录摘抄" @close="closeAddExcerptModal">
    <form class="collect-excerpt-form" @submit.prevent="addExcerptToTopic">
      <label>
        搜索材料
        <input
          v-model="excerptSearch"
          placeholder="搜索原文、笔记、书籍、章节或标签"
        />
      </label>

      <section class="excerpt-picker">
        <div class="excerpt-picker-list">
          <button
            v-for="excerpt in filteredExcerptsToCollect"
            :key="excerpt.id"
            class="excerpt-picker-item"
            :class="{ active: excerpt.id === excerptIdToAdd }"
            type="button"
            @click="excerptIdToAdd = excerpt.id"
          >
            <span class="item-title">{{ excerpt.quote }}</span>
            <span class="item-meta">{{ excerptSourceLabel(excerpt) }}</span>
            <span v-if="excerpt.tags.length > 0" class="tag-row compact-tags">
              <span
                v-for="tag in excerpt.tags"
                :key="tag.id"
                class="tag-pill"
                :style="tagStyle(tag)"
              >
                #{{ tag.name }}
              </span>
            </span>
          </button>

          <p
            v-if="availableExcerptsToCollect.length === 0"
            class="empty-state excerpt-picker-empty"
          >
            当前主题已经收录了全部摘抄。
          </p>
          <p
            v-else-if="filteredExcerptsToCollect.length === 0"
            class="empty-state excerpt-picker-empty"
          >
            没有匹配的摘抄。
          </p>
        </div>

        <aside class="excerpt-picker-preview">
          <template v-if="excerptToAdd">
            <section class="excerpt-picker-selected">
              <p class="source-line">{{ excerptSourceLabel(excerptToAdd) }}</p>
              <blockquote>{{ excerptToAdd.quote }}</blockquote>
            </section>
          </template>
          <p v-else class="empty-state">选择一条摘抄后填写收录信息。</p>

          <label>
            收录理由
            <textarea v-model="reason" rows="4" placeholder="为什么把它放进这个主题？" />
          </label>
          <label>
            主题内理解
            <textarea
              v-model="topicReflection"
              rows="6"
              placeholder="这条摘抄在当前主题下意味着什么？"
            />
          </label>
        </aside>
      </section>

      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="closeAddExcerptModal">
          取消
        </button>
        <button class="primary-action" :disabled="isSaving || !excerptIdToAdd" type="submit">
          保存
        </button>
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
        <CustomSelect
          v-model="editingNodes[editingNodeId].parentId"
          :options="nodeParentEditOptions(editingNodeId)"
        />
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
        <button class="danger-action" type="button" @click="confirmDestructiveAction">
          {{ confirmActionLabel }}
        </button>
      </div>
    </div>
  </BaseModal>
</template>
