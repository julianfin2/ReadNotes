<script setup lang="ts">
type ViewKey = "excerpts" | "topics" | "tags";

defineProps<{
  activeView: ViewKey;
}>();

defineEmits<{
  selectView: [view: ViewKey];
}>();
</script>

<template>
  <aside class="sidebar">
    <div>
      <h1>摘抄库</h1>
    </div>

    <nav>
      <button
        class="nav-item"
        :class="{ active: activeView === 'excerpts' }"
        @click="$emit('selectView', 'excerpts')"
      >
        摘抄
      </button>
      <button
        class="nav-item"
        :class="{ active: activeView === 'topics' }"
        @click="$emit('selectView', 'topics')"
      >
        主题
      </button>
      <button
        class="nav-item"
        :class="{ active: activeView === 'tags' }"
        @click="$emit('selectView', 'tags')"
      >
        标签
      </button>
    </nav>
  </aside>
</template>
