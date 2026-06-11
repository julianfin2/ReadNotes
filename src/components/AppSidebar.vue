<script setup lang="ts">
type ViewKey = "excerpts" | "topics" | "tags" | "timeline";

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
      <p class="eyebrow">ReadNotes</p>
      <h1>{{ activeView === "topics" ? "主题" : "摘抄库" }}</h1>
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
      <button class="nav-item" disabled>标签</button>
      <button class="nav-item" disabled>时间线</button>
    </nav>
  </aside>
</template>
