<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
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

const editingTopics = reactive<Record<string, TopicDraft>>({});
const editingNodes = reactive<Record<string, NodeDraft>>({});
const editingTopicExcerpts = reactive<Record<string, TopicExcerptDraft>>({});

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

onMounted(async () => {
  await loadTopics();
});

watch(selectedTopicId, async (topicId) => {
  selectedNodeId.value = "";
  topicNodes.value = [];
  topicExcerpts.value = [];
  clearEditingState();

  if (topicId) {
    await Promise.all([loadTopicNodes(topicId), loadTopicExcerpts(topicId)]);
  }
});

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
    await loadTopicExcerpts(selectedTopicId.value);
  });
}

async function updateTopicExcerpt(topicExcerptId: string) {
  const draft = editingTopicExcerpts[topicExcerptId];
  if (!draft || !selectedTopicId.value) {
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
  editingTopics[topic.id] = {
    title: topic.title,
    description: topic.description || "",
    researchQuestion: topic.researchQuestion || "",
    status: topic.status,
  };
}

function startEditingNode(node: TopicNode) {
  editingNodes[node.id] = {
    parentId: node.parentId || "",
    title: node.title,
    summary: node.summary || "",
    sortOrder: node.sortOrder,
  };
}

function startEditingTopicExcerpt(topicExcerpt: TopicExcerpt) {
  editingTopicExcerpts[topicExcerpt.id] = {
    nodeId: topicExcerpt.nodeId || "",
    reason: topicExcerpt.reason || "",
    topicReflection: topicExcerpt.topicReflection || "",
    sortOrder: topicExcerpt.sortOrder,
  };
}

function cancelEditingTopic(topicId: string) {
  delete editingTopics[topicId];
}

function cancelEditingNode(nodeId: string) {
  delete editingNodes[nodeId];
}

function cancelEditingTopicExcerpt(topicExcerptId: string) {
  delete editingTopicExcerpts[topicExcerptId];
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
  <section class="topic-panel">
    <div class="section-heading">
      <p class="eyebrow">Topics</p>
      <h2>主题</h2>
    </div>

    <form @submit.prevent="createTopic">
      <label>
        主题标题
        <input v-model="topicTitle" placeholder="例如：现代人的焦虑来源" />
      </label>

      <label>
        研究问题
        <textarea v-model="topicQuestion" rows="3" placeholder="这个主题想回答什么？" />
      </label>

      <button class="primary-action" :disabled="isSaving" type="submit">创建主题</button>
    </form>

    <div class="topic-list">
      <div
        v-for="topic in topics"
        :key="topic.id"
        class="topic-selector topic-selector-block"
        :class="{ active: topic.id === selectedTopicId }"
      >
        <template v-if="editingTopics[topic.id]">
          <form class="edit-form" @submit.prevent="updateTopic(topic.id)">
            <label>
              标题
              <input v-model="editingTopics[topic.id].title" />
            </label>
            <label>
              研究问题
              <textarea v-model="editingTopics[topic.id].researchQuestion" rows="3" />
            </label>
            <label>
              状态
              <select v-model="editingTopics[topic.id].status">
                <option value="collecting">collecting</option>
                <option value="organizing">organizing</option>
                <option value="drafting">drafting</option>
                <option value="finished">finished</option>
                <option value="paused">paused</option>
              </select>
            </label>
            <div class="action-row">
              <button class="primary-action" type="submit">保存</button>
              <button class="secondary-action" type="button" @click="cancelEditingTopic(topic.id)">
                取消
              </button>
            </div>
          </form>
        </template>

        <template v-else>
          <button class="plain-selector" type="button" @click="selectedTopicId = topic.id">
            <span>{{ topic.title }}</span>
            <small>{{ topic.status }}</small>
          </button>
          <div class="action-row">
            <button class="secondary-action" type="button" @click="startEditingTopic(topic)">
              编辑
            </button>
            <button class="danger-action" type="button" @click="deleteTopic(topic.id)">
              删除
            </button>
          </div>
        </template>
      </div>
    </div>

    <p v-if="topics.length === 0" class="empty-state">还没有主题。</p>
  </section>

  <section class="workspace-panel">
    <div class="section-heading">
      <p class="eyebrow">Workspace</p>
      <h2>{{ selectedTopic?.title || "主题工作台" }}</h2>
    </div>

    <div v-if="selectedTopic" class="topic-workspace-grid">
      <div class="stack">
        <div class="topic-card">
          <h3>子主题</h3>
          <form @submit.prevent="createTopicNode">
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
              <textarea v-model="nodeSummary" rows="3" placeholder="这个子主题目前怎么理解？" />
            </label>
            <button class="secondary-action" :disabled="isSaving" type="submit">
              添加子主题
            </button>
          </form>

          <div class="node-list">
            <button
              class="node-selector"
              :class="{ active: selectedNodeId === '' }"
              @click="selectedNodeId = ''"
            >
              全部摘抄
            </button>

            <div v-for="node in topicNodes" :key="node.id" class="node-editor">
              <template v-if="editingNodes[node.id]">
                <form class="edit-form" @submit.prevent="updateTopicNode(node.id)">
                  <label>
                    标题
                    <input v-model="editingNodes[node.id].title" />
                  </label>
                  <label>
                    父子主题
                    <select v-model="editingNodes[node.id].parentId">
                      <option value="">无</option>
                      <option
                        v-for="parent in selectableParents(node.id)"
                        :key="parent.id"
                        :value="parent.id"
                      >
                        {{ nodeLabel(parent) }}
                      </option>
                    </select>
                  </label>
                  <label>
                    摘要
                    <textarea v-model="editingNodes[node.id].summary" rows="3" />
                  </label>
                  <div class="action-row">
                    <button class="primary-action" type="submit">保存</button>
                    <button
                      class="secondary-action"
                      type="button"
                      @click="cancelEditingNode(node.id)"
                    >
                      取消
                    </button>
                  </div>
                </form>
              </template>

              <template v-else>
                <button
                  class="node-selector"
                  :class="{ active: node.id === selectedNodeId }"
                  @click="selectedNodeId = node.id"
                >
                  {{ nodeLabel(node) }}
                </button>
                <p v-if="node.summary" class="reflection">{{ node.summary }}</p>
                <div class="action-row">
                  <button class="secondary-action" type="button" @click="startEditingNode(node)">
                    编辑
                  </button>
                  <button class="danger-action" type="button" @click="deleteTopicNode(node.id)">
                    删除
                  </button>
                </div>
              </template>
            </div>
          </div>
        </div>

        <div class="topic-card">
          <h3>收录摘抄</h3>
          <form @submit.prevent="addExcerptToTopic">
            <label>
              选择摘抄
              <select v-model="excerptIdToAdd">
                <option value="">请选择</option>
                <option v-for="excerpt in props.excerpts" :key="excerpt.id" :value="excerpt.id">
                  {{ excerpt.quote.slice(0, 48) }}
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
                rows="4"
                placeholder="这条摘抄在当前主题下意味着什么？"
              />
            </label>

            <button class="primary-action" :disabled="isSaving" type="submit">
              加入主题
            </button>
          </form>
        </div>
      </div>

      <div class="stack">
        <div class="topic-card">
          <h3>{{ selectedNode?.title || "全部摘抄" }}</h3>
          <p v-if="selectedTopic.researchQuestion" class="reflection">
            {{ selectedTopic.researchQuestion }}
          </p>
        </div>

        <article
          v-for="topicExcerpt in visibleTopicExcerpts"
          :key="topicExcerpt.id"
          class="excerpt-card"
        >
          <template v-if="editingTopicExcerpts[topicExcerpt.id]">
            <form class="edit-form" @submit.prevent="updateTopicExcerpt(topicExcerpt.id)">
              <label>
                子主题
                <select v-model="editingTopicExcerpts[topicExcerpt.id].nodeId">
                  <option value="">未分类</option>
                  <option v-for="node in topicNodes" :key="node.id" :value="node.id">
                    {{ nodeLabel(node) }}
                  </option>
                </select>
              </label>
              <label>
                收录理由
                <textarea v-model="editingTopicExcerpts[topicExcerpt.id].reason" rows="3" />
              </label>
              <label>
                主题理解
                <textarea
                  v-model="editingTopicExcerpts[topicExcerpt.id].topicReflection"
                  rows="4"
                />
              </label>
              <div class="action-row">
                <button class="primary-action" type="submit">保存</button>
                <button
                  class="secondary-action"
                  type="button"
                  @click="cancelEditingTopicExcerpt(topicExcerpt.id)"
                >
                  取消
                </button>
              </div>
            </form>
          </template>

          <template v-else>
            <blockquote>{{ topicExcerpt.excerpt.quote }}</blockquote>
            <p
              v-if="topicExcerpt.excerpt.bookTitle || topicExcerpt.excerpt.chapterTitle"
              class="source-line"
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
            </p>
            <p v-if="topicExcerpt.reason" class="reflection">
              收录理由：{{ topicExcerpt.reason }}
            </p>
            <p v-if="topicExcerpt.topicReflection" class="reflection">
              主题理解：{{ topicExcerpt.topicReflection }}
            </p>
            <div v-if="topicExcerpt.excerpt.tags.length > 0" class="tag-row">
              <span v-for="tag in topicExcerpt.excerpt.tags" :key="tag.id" class="tag-pill">
                #{{ tag.name }}
              </span>
            </div>
            <footer>
              <span>{{ new Date(topicExcerpt.addedAt).toLocaleString() }}</span>
            </footer>
            <div class="action-row">
              <button
                class="secondary-action"
                type="button"
                @click="startEditingTopicExcerpt(topicExcerpt)"
              >
                编辑
              </button>
              <button
                class="danger-action"
                type="button"
                @click="removeTopicExcerpt(topicExcerpt.id)"
              >
                移除
              </button>
            </div>
          </template>
        </article>

        <p v-if="visibleTopicExcerpts.length === 0" class="empty-state">
          当前范围还没有收录摘抄。
        </p>
      </div>
    </div>

    <p v-else class="empty-state">先创建一个主题。</p>
    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
  </section>
</template>
