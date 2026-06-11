<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  databasePath: string;
  errorMessage: string;
  isSaving: boolean;
}>();

const emit = defineEmits<{
  createExcerpt: [
    input: {
      quote: string;
      reflection: string;
      bookTitle: string;
      chapterTitle: string;
      tagNames: string[];
    },
  ];
}>();

const quote = ref("");
const reflection = ref("");
const bookTitle = ref("");
const chapterTitle = ref("");
const tagInput = ref("");

function submit() {
  emit("createExcerpt", {
    quote: quote.value,
    reflection: reflection.value,
    bookTitle: bookTitle.value,
    chapterTitle: chapterTitle.value,
    tagNames: parseTagInput(tagInput.value),
  });

  quote.value = "";
  reflection.value = "";
  bookTitle.value = "";
  chapterTitle.value = "";
  tagInput.value = "";
}

function parseTagInput(value: string) {
  return value
    .split(/[\s,，#]+/)
    .map((tag) => tag.trim())
    .filter(Boolean);
}
</script>

<template>
  <section class="capture-panel">
    <div class="section-heading">
      <p class="eyebrow">Quick capture</p>
      <h2>新增摘抄</h2>
    </div>

    <form @submit.prevent="submit">
      <label>
        原文
        <textarea v-model="quote" rows="7" placeholder="输入摘抄原文" />
      </label>

      <label>
        初始理解
        <textarea v-model="reflection" rows="5" placeholder="写下此刻的理解" />
      </label>

      <label>
        标签
        <input v-model="tagInput" placeholder="例如：人性 写作素材 #焦虑" />
      </label>

      <label>
        书籍名
        <input v-model="bookTitle" placeholder="例如：置身事内" />
      </label>

      <label>
        章节名
        <input v-model="chapterTitle" placeholder="例如：地方政府的权力与事务" />
      </label>
      <button class="primary-action" :disabled="isSaving" type="submit">
        {{ isSaving ? "保存中" : "保存摘抄" }}
      </button>

      <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
    </form>

    <p class="database-path">Database: {{ databasePath }}</p>
  </section>
</template>
