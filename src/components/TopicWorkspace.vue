<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, shallowRef, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  ArrowLeft,
  Link,
  Pencil,
  Plus,
  RotateCcw,
  Save,
  Trash2,
  Unlink,
  X,
} from "@lucide/vue";
import BaseModal from "./BaseModal.vue";
import CustomSelect from "./CustomSelect.vue";
import type { Excerpt } from "../types/excerpt";
import type { Tag } from "../types/tag";
import type { Topic, TopicExcerpt, TopicNode, TopicStatus } from "../types/topic";
import { deleteDraftPayload, getDraftPayload, saveDraftPayload } from "../utils/drafts";
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

type TopicCreateDraft = {
  title: string;
  researchQuestion: string;
};

type NodeCreateDraft = {
  parentId: string;
  title: string;
  summary: string;
};

type TopicCollectDraft = {
  nodeId: string;
  excerptId: string;
  excerptSearch: string;
  reason: string;
  topicReflection: string;
};

type ConfirmAction = () => void | Promise<void>;
type RestoreDraftKind =
  | "topicCreate"
  | "topic"
  | "topicCollect"
  | "topicNodeCreate"
  | "topicNode"
  | "topicExcerpt";

const ACTIVE_CREATE_DRAFT_ID = "active";
const TOPIC_CREATE_DRAFT_TYPE = "topicCreate";
const TOPIC_DRAFT_TYPE = "topic";
const TOPIC_COLLECT_DRAFT_TYPE = "topicCollect";
const TOPIC_NODE_CREATE_DRAFT_TYPE = "topicNodeCreate";
const TOPIC_NODE_DRAFT_TYPE = "topicNode";
const TOPIC_EXCERPT_DRAFT_TYPE = "topicExcerpt";

const editingTopics = reactive<Record<string, TopicDraft>>({});
const editingNodes = reactive<Record<string, NodeDraft>>({});
const editingTopicExcerpts = reactive<Record<string, TopicExcerptDraft>>({});
const confirmAction = shallowRef<ConfirmAction | null>(null);
const restoreDraftModalOpen = ref(false);
const restoreDraftKind = ref<RestoreDraftKind | null>(null);
const pendingTopicCreateDraft = ref<TopicCreateDraft | null>(null);
const pendingTopicDraft = ref<TopicDraft | null>(null);
const pendingTopicCollectDraft = ref<TopicCollectDraft | null>(null);
const pendingNodeCreateDraft = ref<NodeCreateDraft | null>(null);
const pendingNodeDraft = ref<NodeDraft | null>(null);
const pendingTopicExcerptDraft = ref<TopicExcerptDraft | null>(null);
let topicCreateDraftSaveTimer: number | undefined;
let topicDraftSaveTimer: number | undefined;
let topicCollectDraftSaveTimer: number | undefined;
let nodeCreateDraftSaveTimer: number | undefined;
let nodeDraftSaveTimer: number | undefined;
let topicExcerptDraftSaveTimer: number | undefined;

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

const restoreDraftMessage = computed(() => {
  if (restoreDraftKind.value === "topicCreate") {
    return "存在上次未保存的新建主题内容，是否恢复草稿？";
  }

  if (restoreDraftKind.value === "topic") {
    return "当前主题存在上次未保存的编辑内容，是否恢复草稿？";
  }

  if (restoreDraftKind.value === "topicCollect") {
    return "当前主题存在上次未保存的收录内容，是否恢复草稿？";
  }

  if (restoreDraftKind.value === "topicNodeCreate") {
    return "当前主题存在上次未保存的新建子主题内容，是否恢复草稿？";
  }

  if (restoreDraftKind.value === "topicNode") {
    return "当前子主题存在上次未保存的编辑内容，是否恢复草稿？";
  }

  return "当前收录信息存在上次未保存的编辑内容，是否恢复草稿？";
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

onBeforeUnmount(() => {
  saveTopicCreateDraftNow();
  saveTopicDraftNow();
  saveTopicCollectDraftNow();
  saveNodeCreateDraftNow();
  saveNodeDraftNow();
  saveTopicExcerptDraftNow();
  if (topicCreateDraftSaveTimer) {
    window.clearTimeout(topicCreateDraftSaveTimer);
  }
  if (topicDraftSaveTimer) {
    window.clearTimeout(topicDraftSaveTimer);
  }
  if (topicCollectDraftSaveTimer) {
    window.clearTimeout(topicCollectDraftSaveTimer);
  }
  if (nodeCreateDraftSaveTimer) {
    window.clearTimeout(nodeCreateDraftSaveTimer);
  }
  if (nodeDraftSaveTimer) {
    window.clearTimeout(nodeDraftSaveTimer);
  }
  if (topicExcerptDraftSaveTimer) {
    window.clearTimeout(topicExcerptDraftSaveTimer);
  }
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
  () => ({
    viewMode: viewMode.value,
    title: topicTitle.value,
    researchQuestion: topicQuestion.value,
  }),
  () => scheduleTopicCreateDraftSave(),
);

watch(
  () => {
    const topicId = editingTopicId.value;
    const draft = topicId ? editingTopics[topicId] : null;
    return draft
      ? {
          topicId,
          title: draft.title,
          description: draft.description,
          researchQuestion: draft.researchQuestion,
          status: draft.status,
        }
      : null;
  },
  () => scheduleTopicDraftSave(),
);

watch(
  () => ({
    open: addExcerptModalOpen.value,
    topicId: selectedTopicId.value,
    nodeId: selectedNodeId.value,
    excerptId: excerptIdToAdd.value,
    excerptSearch: excerptSearch.value,
    reason: reason.value,
    topicReflection: topicReflection.value,
  }),
  () => scheduleTopicCollectDraftSave(),
);

watch(
  () => ({
    open: nodeModalOpen.value,
    topicId: selectedTopicId.value,
    parentId: nodeParentId.value,
    title: nodeTitle.value,
    summary: nodeSummary.value,
  }),
  () => scheduleNodeCreateDraftSave(),
);

watch(
  () => {
    const nodeId = editingNodeId.value;
    const draft = nodeId ? editingNodes[nodeId] : null;
    return draft
      ? {
          nodeId,
          parentId: draft.parentId,
          title: draft.title,
          summary: draft.summary,
          sortOrder: draft.sortOrder,
        }
      : null;
  },
  () => scheduleNodeDraftSave(),
);

watch(
  () => {
    const topicExcerptId = editingTopicExcerptId.value;
    const draft = topicExcerptId ? editingTopicExcerpts[topicExcerptId] : null;
    return draft
      ? {
          topicExcerptId,
          nodeId: draft.nodeId,
          reason: draft.reason,
          topicReflection: draft.topicReflection,
          sortOrder: draft.sortOrder,
        }
      : null;
  },
  () => scheduleTopicExcerptDraftSave(),
);

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

    await deleteDraftPayload(TOPIC_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
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

    await deleteDraftPayload(TOPIC_DRAFT_TYPE, topicId);
    delete editingTopics[topicId];
    editingTopicId.value = "";
    await reloadSelectedTopic();
    viewMode.value = "list";
  });
}

async function deleteTopic(topicId: string) {
  await runSaving(async () => {
    await invoke("delete_topic", { id: topicId });
    await deleteDraftPayload(TOPIC_DRAFT_TYPE, topicId);
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

    await deleteDraftPayload(TOPIC_NODE_CREATE_DRAFT_TYPE, selectedTopicId.value);
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

    await deleteDraftPayload(TOPIC_NODE_DRAFT_TYPE, nodeId);
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
    await deleteDraftPayload(TOPIC_NODE_DRAFT_TYPE, nodeId);
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

    await deleteDraftPayload(TOPIC_COLLECT_DRAFT_TYPE, selectedTopicId.value);
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

    await deleteDraftPayload(TOPIC_EXCERPT_DRAFT_TYPE, topicExcerptId);
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
    await deleteDraftPayload(TOPIC_EXCERPT_DRAFT_TYPE, topicExcerptId);
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
    void checkTopicDraft(topic.id);
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
  void checkNodeDraft(node.id);
}

function startEditingTopicExcerpt(topicExcerpt: TopicExcerpt) {
  editingTopicExcerptId.value = topicExcerpt.id;
  editingTopicExcerpts[topicExcerpt.id] = {
    nodeId: topicExcerpt.nodeId || "",
    reason: topicExcerpt.reason || "",
    topicReflection: topicExcerpt.topicReflection || "",
    sortOrder: topicExcerpt.sortOrder,
  };
  void checkTopicExcerptDraft(topicExcerpt.id);
}

function cancelEditingTopic() {
  returnToTopicList();
}

function startCreatingTopic() {
  topicTitle.value = "";
  topicQuestion.value = "";
  viewMode.value = "create";
  void checkTopicCreateDraft();
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
      if (viewMode.value === "create") {
        void deleteDraftPayload(TOPIC_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
      }
      clearEditingState();
      topicTitle.value = "";
      topicQuestion.value = "";
      viewMode.value = "list";
    });
  });
}

function cancelEditingNode(nodeId: string) {
  if (nodeId) {
    void deleteDraftPayload(TOPIC_NODE_DRAFT_TYPE, nodeId);
  }
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
  void checkTopicCollectDraft();
}

function closeAddExcerptModal() {
  if (selectedTopicId.value) {
    void deleteDraftPayload(TOPIC_COLLECT_DRAFT_TYPE, selectedTopicId.value);
  }
  addExcerptModalOpen.value = false;
  excerptIdToAdd.value = "";
  excerptSearch.value = "";
  reason.value = "";
  topicReflection.value = "";
}

function openNodeModal() {
  nodeTitle.value = "";
  nodeSummary.value = "";
  nodeParentId.value = "";
  nodeModalOpen.value = true;
  void checkNodeCreateDraft();
}

function closeNodeModal() {
  if (selectedTopicId.value) {
    void deleteDraftPayload(TOPIC_NODE_CREATE_DRAFT_TYPE, selectedTopicId.value);
  }
  nodeModalOpen.value = false;
  nodeTitle.value = "";
  nodeSummary.value = "";
  nodeParentId.value = "";
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

function isCollectDraftDirty() {
  return Boolean(
    addExcerptModalOpen.value &&
      selectedTopicId.value &&
      (excerptIdToAdd.value ||
        excerptSearch.value.trim() ||
        reason.value.trim() ||
        topicReflection.value.trim()),
  );
}

function isNodeCreateDraftDirty() {
  return Boolean(
    nodeModalOpen.value &&
      selectedTopicId.value &&
      (nodeParentId.value || nodeTitle.value.trim() || nodeSummary.value.trim()),
  );
}

function isNodeDraftDirty(nodeId: string) {
  const node = topicNodes.value.find((candidate) => candidate.id === nodeId);
  const draft = editingNodes[nodeId];

  if (!node || !draft) {
    return false;
  }

  return (
    draft.parentId !== (node.parentId || "") ||
    draft.title !== node.title ||
    draft.summary !== (node.summary || "") ||
    draft.sortOrder !== node.sortOrder
  );
}

function topicCreateDraftPayload(): TopicCreateDraft {
  return {
    title: topicTitle.value,
    researchQuestion: topicQuestion.value,
  };
}

function topicCollectDraftPayload(): TopicCollectDraft {
  return {
    nodeId: selectedNodeId.value,
    excerptId: excerptIdToAdd.value,
    excerptSearch: excerptSearch.value,
    reason: reason.value,
    topicReflection: topicReflection.value,
  };
}

function nodeCreateDraftPayload(): NodeCreateDraft {
  return {
    parentId: nodeParentId.value,
    title: nodeTitle.value,
    summary: nodeSummary.value,
  };
}

function scheduleTopicCreateDraftSave() {
  if (topicCreateDraftSaveTimer) {
    window.clearTimeout(topicCreateDraftSaveTimer);
    topicCreateDraftSaveTimer = undefined;
  }

  if (viewMode.value !== "create" || !isCreateTopicDirty.value) {
    return;
  }

  topicCreateDraftSaveTimer = window.setTimeout(() => {
    topicCreateDraftSaveTimer = undefined;
    if (viewMode.value !== "create" || !isCreateTopicDirty.value) {
      return;
    }

    void saveDraftPayload(
      TOPIC_CREATE_DRAFT_TYPE,
      ACTIVE_CREATE_DRAFT_ID,
      topicCreateDraftPayload(),
    );
  }, 800);
}

function saveTopicCreateDraftNow() {
  if (viewMode.value !== "create" || !isCreateTopicDirty.value) {
    return;
  }

  void saveDraftPayload(
    TOPIC_CREATE_DRAFT_TYPE,
    ACTIVE_CREATE_DRAFT_ID,
    topicCreateDraftPayload(),
  );
}

function isTopicDraftPayloadDifferent(topicId: string, payload: TopicDraft) {
  const draft = editingTopics[topicId];
  return Boolean(draft && JSON.stringify(draft) !== JSON.stringify(payload));
}

function isNodeDraftPayloadDifferent(nodeId: string, payload: NodeDraft) {
  const draft = editingNodes[nodeId];
  return Boolean(draft && JSON.stringify(draft) !== JSON.stringify(payload));
}

function isTopicExcerptDraftPayloadDifferent(
  topicExcerptId: string,
  payload: TopicExcerptDraft,
) {
  const draft = editingTopicExcerpts[topicExcerptId];
  return Boolean(draft && JSON.stringify(draft) !== JSON.stringify(payload));
}

function scheduleTopicDraftSave() {
  if (topicDraftSaveTimer) {
    window.clearTimeout(topicDraftSaveTimer);
    topicDraftSaveTimer = undefined;
  }

  if (viewMode.value !== "edit" || !editingTopicId.value || !isTopicEditDirty.value) {
    return;
  }

  topicDraftSaveTimer = window.setTimeout(() => {
    topicDraftSaveTimer = undefined;
    const topicId = editingTopicId.value;
    const draft = topicId ? editingTopics[topicId] : null;
    if (viewMode.value !== "edit" || !topicId || !draft || !isTopicEditDirty.value) {
      return;
    }

    void saveDraftPayload(TOPIC_DRAFT_TYPE, topicId, { ...draft });
  }, 800);
}

function saveTopicDraftNow() {
  const topicId = editingTopicId.value;
  const draft = topicId ? editingTopics[topicId] : null;
  if (viewMode.value !== "edit" || !topicId || !draft || !isTopicEditDirty.value) {
    return;
  }

  void saveDraftPayload(TOPIC_DRAFT_TYPE, topicId, { ...draft });
}

function scheduleTopicCollectDraftSave() {
  if (topicCollectDraftSaveTimer) {
    window.clearTimeout(topicCollectDraftSaveTimer);
    topicCollectDraftSaveTimer = undefined;
  }

  if (!isCollectDraftDirty()) {
    return;
  }

  topicCollectDraftSaveTimer = window.setTimeout(() => {
    topicCollectDraftSaveTimer = undefined;
    if (!isCollectDraftDirty()) {
      return;
    }

    void saveDraftPayload(
      TOPIC_COLLECT_DRAFT_TYPE,
      selectedTopicId.value,
      topicCollectDraftPayload(),
    );
  }, 800);
}

function saveTopicCollectDraftNow() {
  if (!isCollectDraftDirty()) {
    return;
  }

  void saveDraftPayload(
    TOPIC_COLLECT_DRAFT_TYPE,
    selectedTopicId.value,
    topicCollectDraftPayload(),
  );
}

function scheduleNodeCreateDraftSave() {
  if (nodeCreateDraftSaveTimer) {
    window.clearTimeout(nodeCreateDraftSaveTimer);
    nodeCreateDraftSaveTimer = undefined;
  }

  if (!isNodeCreateDraftDirty()) {
    return;
  }

  nodeCreateDraftSaveTimer = window.setTimeout(() => {
    nodeCreateDraftSaveTimer = undefined;
    if (!isNodeCreateDraftDirty()) {
      return;
    }

    void saveDraftPayload(
      TOPIC_NODE_CREATE_DRAFT_TYPE,
      selectedTopicId.value,
      nodeCreateDraftPayload(),
    );
  }, 800);
}

function saveNodeCreateDraftNow() {
  if (!isNodeCreateDraftDirty()) {
    return;
  }

  void saveDraftPayload(
    TOPIC_NODE_CREATE_DRAFT_TYPE,
    selectedTopicId.value,
    nodeCreateDraftPayload(),
  );
}

function scheduleNodeDraftSave() {
  if (nodeDraftSaveTimer) {
    window.clearTimeout(nodeDraftSaveTimer);
    nodeDraftSaveTimer = undefined;
  }

  if (!editingNodeId.value || !isNodeDraftDirty(editingNodeId.value)) {
    return;
  }

  nodeDraftSaveTimer = window.setTimeout(() => {
    nodeDraftSaveTimer = undefined;
    const nodeId = editingNodeId.value;
    const draft = nodeId ? editingNodes[nodeId] : null;
    if (!nodeId || !draft || !isNodeDraftDirty(nodeId)) {
      return;
    }

    void saveDraftPayload(TOPIC_NODE_DRAFT_TYPE, nodeId, { ...draft });
  }, 800);
}

function saveNodeDraftNow() {
  const nodeId = editingNodeId.value;
  const draft = nodeId ? editingNodes[nodeId] : null;
  if (!nodeId || !draft || !isNodeDraftDirty(nodeId)) {
    return;
  }

  void saveDraftPayload(TOPIC_NODE_DRAFT_TYPE, nodeId, { ...draft });
}

function scheduleTopicExcerptDraftSave() {
  if (topicExcerptDraftSaveTimer) {
    window.clearTimeout(topicExcerptDraftSaveTimer);
    topicExcerptDraftSaveTimer = undefined;
  }

  if (!editingTopicExcerptId.value || !isTopicExcerptEditDirty.value) {
    return;
  }

  topicExcerptDraftSaveTimer = window.setTimeout(() => {
    topicExcerptDraftSaveTimer = undefined;
    const topicExcerptId = editingTopicExcerptId.value;
    const draft = topicExcerptId ? editingTopicExcerpts[topicExcerptId] : null;
    if (!topicExcerptId || !draft || !isTopicExcerptEditDirty.value) {
      return;
    }

    void saveDraftPayload(TOPIC_EXCERPT_DRAFT_TYPE, topicExcerptId, { ...draft });
  }, 800);
}

function saveTopicExcerptDraftNow() {
  const topicExcerptId = editingTopicExcerptId.value;
  const draft = topicExcerptId ? editingTopicExcerpts[topicExcerptId] : null;
  if (!topicExcerptId || !draft || !isTopicExcerptEditDirty.value) {
    return;
  }

  void saveDraftPayload(TOPIC_EXCERPT_DRAFT_TYPE, topicExcerptId, { ...draft });
}

async function checkTopicCreateDraft() {
  try {
    const draft = await getDraftPayload<TopicCreateDraft>(
      TOPIC_CREATE_DRAFT_TYPE,
      ACTIVE_CREATE_DRAFT_ID,
    );
    if (!draft || viewMode.value !== "create") {
      return;
    }

    if (JSON.stringify(topicCreateDraftPayload()) === JSON.stringify(draft.payload)) {
      await deleteDraftPayload(TOPIC_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
      return;
    }

    pendingTopicCreateDraft.value = draft.payload;
    restoreDraftKind.value = "topicCreate";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingRestoreDraft();
  }
}

async function checkTopicDraft(topicId: string) {
  try {
    const draft = await getDraftPayload<TopicDraft>(TOPIC_DRAFT_TYPE, topicId);
    if (!draft || viewMode.value !== "edit" || editingTopicId.value !== topicId) {
      return;
    }

    if (!isTopicDraftPayloadDifferent(topicId, draft.payload)) {
      await deleteDraftPayload(TOPIC_DRAFT_TYPE, topicId);
      return;
    }

    pendingTopicDraft.value = draft.payload;
    pendingTopicExcerptDraft.value = null;
    restoreDraftKind.value = "topic";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingRestoreDraft();
  }
}

async function checkTopicCollectDraft() {
  if (!selectedTopicId.value) {
    return;
  }

  try {
    const draft = await getDraftPayload<TopicCollectDraft>(
      TOPIC_COLLECT_DRAFT_TYPE,
      selectedTopicId.value,
    );
    if (!draft || !addExcerptModalOpen.value || !selectedTopicId.value) {
      return;
    }

    if (JSON.stringify(topicCollectDraftPayload()) === JSON.stringify(draft.payload)) {
      await deleteDraftPayload(TOPIC_COLLECT_DRAFT_TYPE, selectedTopicId.value);
      return;
    }

    pendingTopicCollectDraft.value = draft.payload;
    restoreDraftKind.value = "topicCollect";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingRestoreDraft();
  }
}

async function checkNodeCreateDraft() {
  if (!selectedTopicId.value) {
    return;
  }

  try {
    const draft = await getDraftPayload<NodeCreateDraft>(
      TOPIC_NODE_CREATE_DRAFT_TYPE,
      selectedTopicId.value,
    );
    if (!draft || !nodeModalOpen.value || !selectedTopicId.value) {
      return;
    }

    if (JSON.stringify(nodeCreateDraftPayload()) === JSON.stringify(draft.payload)) {
      await deleteDraftPayload(TOPIC_NODE_CREATE_DRAFT_TYPE, selectedTopicId.value);
      return;
    }

    pendingNodeCreateDraft.value = draft.payload;
    restoreDraftKind.value = "topicNodeCreate";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingRestoreDraft();
  }
}

async function checkNodeDraft(nodeId: string) {
  try {
    const draft = await getDraftPayload<NodeDraft>(TOPIC_NODE_DRAFT_TYPE, nodeId);
    if (!draft || editingNodeId.value !== nodeId || !editNodeModalOpen.value) {
      return;
    }

    if (!isNodeDraftPayloadDifferent(nodeId, draft.payload)) {
      await deleteDraftPayload(TOPIC_NODE_DRAFT_TYPE, nodeId);
      return;
    }

    pendingNodeDraft.value = draft.payload;
    restoreDraftKind.value = "topicNode";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingRestoreDraft();
  }
}

async function checkTopicExcerptDraft(topicExcerptId: string) {
  try {
    const draft = await getDraftPayload<TopicExcerptDraft>(
      TOPIC_EXCERPT_DRAFT_TYPE,
      topicExcerptId,
    );
    if (!draft || editingTopicExcerptId.value !== topicExcerptId) {
      return;
    }

    if (!isTopicExcerptDraftPayloadDifferent(topicExcerptId, draft.payload)) {
      await deleteDraftPayload(TOPIC_EXCERPT_DRAFT_TYPE, topicExcerptId);
      return;
    }

    pendingTopicExcerptDraft.value = draft.payload;
    pendingTopicDraft.value = null;
    restoreDraftKind.value = "topicExcerpt";
    restoreDraftModalOpen.value = true;
  } catch {
    clearPendingRestoreDraft();
  }
}

function restorePendingDraft() {
  if (restoreDraftKind.value === "topicCreate" && pendingTopicCreateDraft.value) {
    topicTitle.value = pendingTopicCreateDraft.value.title;
    topicQuestion.value = pendingTopicCreateDraft.value.researchQuestion;
  }

  if (
    restoreDraftKind.value === "topic" &&
    editingTopicId.value &&
    pendingTopicDraft.value
  ) {
    editingTopics[editingTopicId.value] = { ...pendingTopicDraft.value };
  }

  if (restoreDraftKind.value === "topicCollect" && pendingTopicCollectDraft.value) {
    const draft = pendingTopicCollectDraft.value;
    if (!draft.nodeId || topicNodes.value.some((node) => node.id === draft.nodeId)) {
      selectedNodeId.value = draft.nodeId;
    }
    excerptIdToAdd.value = draft.excerptId;
    excerptSearch.value = draft.excerptSearch;
    reason.value = draft.reason;
    topicReflection.value = draft.topicReflection;
  }

  if (restoreDraftKind.value === "topicNodeCreate" && pendingNodeCreateDraft.value) {
    nodeParentId.value = pendingNodeCreateDraft.value.parentId;
    nodeTitle.value = pendingNodeCreateDraft.value.title;
    nodeSummary.value = pendingNodeCreateDraft.value.summary;
  }

  if (restoreDraftKind.value === "topicNode" && editingNodeId.value && pendingNodeDraft.value) {
    editingNodes[editingNodeId.value] = { ...pendingNodeDraft.value };
  }

  if (
    restoreDraftKind.value === "topicExcerpt" &&
    editingTopicExcerptId.value &&
    pendingTopicExcerptDraft.value
  ) {
    editingTopicExcerpts[editingTopicExcerptId.value] = { ...pendingTopicExcerptDraft.value };
  }

  clearPendingRestoreDraft();
}

function discardPendingDraft() {
  if (restoreDraftKind.value === "topicCreate") {
    void deleteDraftPayload(TOPIC_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
  }

  if (restoreDraftKind.value === "topic" && editingTopicId.value) {
    void deleteDraftPayload(TOPIC_DRAFT_TYPE, editingTopicId.value);
  }

  if (restoreDraftKind.value === "topicCollect" && selectedTopicId.value) {
    void deleteDraftPayload(TOPIC_COLLECT_DRAFT_TYPE, selectedTopicId.value);
  }

  if (restoreDraftKind.value === "topicNodeCreate" && selectedTopicId.value) {
    void deleteDraftPayload(TOPIC_NODE_CREATE_DRAFT_TYPE, selectedTopicId.value);
  }

  if (restoreDraftKind.value === "topicNode" && editingNodeId.value) {
    void deleteDraftPayload(TOPIC_NODE_DRAFT_TYPE, editingNodeId.value);
  }

  if (restoreDraftKind.value === "topicExcerpt" && editingTopicExcerptId.value) {
    void deleteDraftPayload(TOPIC_EXCERPT_DRAFT_TYPE, editingTopicExcerptId.value);
  }

  clearPendingRestoreDraft();
}

function clearPendingRestoreDraft() {
  restoreDraftModalOpen.value = false;
  restoreDraftKind.value = null;
  pendingTopicCreateDraft.value = null;
  pendingTopicDraft.value = null;
  pendingTopicCollectDraft.value = null;
  pendingNodeCreateDraft.value = null;
  pendingNodeDraft.value = null;
  pendingTopicExcerptDraft.value = null;
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
    if (viewMode.value === "create") {
      void deleteDraftPayload(TOPIC_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
    }
    void action();
    return;
  }

  requestConfirmation(
    "放弃更改",
    message,
    async () => {
      if (viewMode.value === "create") {
        await deleteDraftPayload(TOPIC_CREATE_DRAFT_TYPE, ACTIVE_CREATE_DRAFT_ID);
      }
      if (viewMode.value === "edit" && editingTopicId.value) {
        await deleteDraftPayload(TOPIC_DRAFT_TYPE, editingTopicId.value);
      }
      await action();
    },
    "放弃更改",
  );
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
      await deleteDraftPayload(TOPIC_EXCERPT_DRAFT_TYPE, editingTopicExcerptId.value);
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

function topicStatusClass(status: TopicStatus) {
  return `topic-status-${status}`;
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
    <header
      class="page-header"
      :class="{
        'list-toolbar-header': viewMode === 'list',
        'compact-page-header': viewMode === 'edit',
      }"
    >
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
          <Plus aria-hidden="true" />
          新建主题
        </button>
      </div>

      <div v-else-if="viewMode === 'workspace'" class="toolbar topic-toolbar">
        <button class="secondary-action" type="button" @click="returnToTopicList">
          <ArrowLeft aria-hidden="true" />
          返回列表
        </button>
        <button
          v-if="selectedTopic"
          class="secondary-action"
          type="button"
          @click="startEditingTopic(selectedTopic)"
        >
          <Pencil aria-hidden="true" />
          编辑主题
        </button>
        <button
          v-if="selectedTopic"
          class="danger-action"
          type="button"
          @click="requestDeleteTopic(selectedTopic)"
        >
          <Trash2 aria-hidden="true" />
          删除主题
        </button>
      </div>

      <div v-else class="toolbar topic-toolbar">
        <button class="secondary-action" type="button" @click="returnToTopicList">
          <ArrowLeft aria-hidden="true" />
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
            <span class="topic-status" :class="topicStatusClass(topic.status)">
              {{ topicStatusLabel(topic.status) }}
            </span>
            <span class="item-meta" :title="formatDateTime(topic.updatedAt)">
              {{ formatDateOnly(topic.updatedAt) }}
            </span>
            <span class="row-actions" @click.stop>
              <button class="secondary-action" type="button" @click="startEditingTopic(topic)">
                <Pencil aria-hidden="true" />
                编辑
              </button>
              <button class="danger-action" type="button" @click="requestDeleteTopic(topic)">
                <Trash2 aria-hidden="true" />
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
        <button class="secondary-action" type="button" @click="returnToTopicList">
          <X aria-hidden="true" />
          取消
        </button>
        <button class="primary-action" :disabled="!canSaveTopicCreate" type="submit">
          <Save aria-hidden="true" />
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
          <X aria-hidden="true" />
          取消
        </button>
        <button class="primary-action" :disabled="!canSaveTopicEdit" type="submit">
          <Save aria-hidden="true" />
          保存
        </button>
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
            <Pencil aria-hidden="true" />
            编辑
          </button>
          <button
            v-if="selectedNode"
            class="danger-action"
            type="button"
            @click="requestDeleteTopicNode(selectedNode)"
          >
            <Trash2 aria-hidden="true" />
            删除
          </button>
          <button class="secondary-action" type="button" @click="openNodeModal">
            <Plus aria-hidden="true" />
            添加子主题
          </button>
        </div>
      </section>

      <div v-if="selectedNode?.summary" class="topic-scope-note">
        <span class="scope-note-label">摘要</span>
        <span>{{ selectedNode.summary }}</span>
      </div>

      <div class="topic-workspace-grid">
        <aside class="topic-material-pane">
          <section class="context-section material-context-section">
            <div class="card-header">
              <div>
                <h3>材料</h3>
                <p class="subtle-text">{{ visibleTopicExcerpts.length }} 条材料</p>
              </div>
              <button class="primary-action" type="button" @click="openAddExcerptModal">
                <Link aria-hidden="true" />
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
                <X aria-hidden="true" />
                取消
              </button>
              <button
                class="primary-action"
                :disabled="!canSaveTopicExcerptEdit"
                type="submit"
              >
                <Save aria-hidden="true" />
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
                <Pencil aria-hidden="true" />
                编辑收录
              </button>
              <button
                class="danger-action"
                type="button"
                title="只从当前主题移除，不删除摘抄原文"
                @click="requestRemoveTopicExcerpt(selectedTopicExcerpt)"
              >
                <Unlink aria-hidden="true" />
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

  <BaseModal :open="nodeModalOpen" title="添加子主题" @close="closeNodeModal">
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
        <button class="secondary-action" type="button" @click="closeNodeModal">
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
          <X aria-hidden="true" />
          取消
        </button>
        <button class="primary-action" :disabled="isSaving || !excerptIdToAdd" type="submit">
          <Save aria-hidden="true" />
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

  <BaseModal :open="confirmModalOpen" :title="confirmTitle" @close="cancelConfirmation">
    <div class="modal-form">
      <p class="reflection">{{ confirmMessage }}</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="cancelConfirmation">
          <X aria-hidden="true" />
          取消
        </button>
        <button class="danger-action" type="button" @click="confirmDestructiveAction">
          <Trash2 aria-hidden="true" />
          {{ confirmActionLabel }}
        </button>
      </div>
    </div>
  </BaseModal>

  <BaseModal :open="restoreDraftModalOpen" title="发现未保存草稿" @close="discardPendingDraft">
    <div class="modal-form">
      <p class="reflection">{{ restoreDraftMessage }}</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="discardPendingDraft">
          <Trash2 aria-hidden="true" />
          忽略草稿
        </button>
        <button class="primary-action" type="button" @click="restorePendingDraft">
          <RotateCcw aria-hidden="true" />
          恢复草稿
        </button>
      </div>
    </div>
  </BaseModal>
</template>
