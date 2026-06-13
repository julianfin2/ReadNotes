<script setup lang="ts">
import { ref } from "vue";
import BookManager from "./BookManager.vue";
import TagManager from "./TagManager.vue";

const emit = defineEmits<{
  booksChanged: [];
  tagsChanged: [];
}>();

const activeSection = ref<"tags" | "books">("tags");
</script>

<template>
  <section class="page-panel workspace-panel desktop-view management-page">
    <header class="page-header list-toolbar-header">
      <div class="page-title-block">
        <h2>管理</h2>
      </div>

      <div class="management-tabs" role="tablist" aria-label="管理内容">
        <button
          class="management-tab"
          :class="{ active: activeSection === 'tags' }"
          type="button"
          @click="activeSection = 'tags'"
        >
          标签管理
        </button>
        <button
          class="management-tab"
          :class="{ active: activeSection === 'books' }"
          type="button"
          @click="activeSection = 'books'"
        >
          书籍管理
        </button>
      </div>
    </header>

    <TagManager v-if="activeSection === 'tags'" embedded @tags-changed="emit('tagsChanged')" />
    <BookManager v-else @books-changed="emit('booksChanged')" />
  </section>
</template>
