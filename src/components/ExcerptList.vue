<script setup lang="ts">
import type { Excerpt } from "../types/excerpt";

defineProps<{
  excerpts: Excerpt[];
}>();
</script>

<template>
  <section class="library-panel">
    <div class="section-heading">
      <p class="eyebrow">Library</p>
      <h2>{{ excerpts.length }} 条摘抄</h2>
    </div>

    <div class="excerpt-list">
      <article v-for="excerpt in excerpts" :key="excerpt.id" class="excerpt-card">
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
      </article>

      <p v-if="excerpts.length === 0" class="empty-state">还没有摘抄。</p>
    </div>
  </section>
</template>
