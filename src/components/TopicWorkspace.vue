<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Excerpt } from "../types/excerpt";
import type { Topic, TopicExcerpt, TopicNode } from "../types/topic";

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
const excerptIdToAdd = ref("");
const reason = ref("");
const topicReflection = ref("");
const errorMessage = ref("");
const isSaving = ref(false);

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

async function createTopicNode() {
  if (!selectedTopicId.value) {
    errorMessage.value = "请先选择主题";
    return;
  }

  await runSaving(async () => {
    await invoke<TopicNode>("create_topic_node", {
      input: {
        topicId: selectedTopicId.value,
        title: nodeTitle.value,
      },
    });

    nodeTitle.value = "";
    await loadTopicNodes(selectedTopicId.value);
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
      <button
        v-for="topic in topics"
        :key="topic.id"
        class="topic-selector"
        :class="{ active: topic.id === selectedTopicId }"
        @click="selectedTopicId = topic.id"
      >
        <span>{{ topic.title }}</span>
        <small>{{ topic.status }}</small>
      </button>
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
            <button
              v-for="node in topicNodes"
              :key="node.id"
              class="node-selector"
              :class="{ active: node.id === selectedNodeId }"
              @click="selectedNodeId = node.id"
            >
              {{ node.title }}
            </button>
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
          <blockquote>{{ topicExcerpt.excerpt.quote }}</blockquote>
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
