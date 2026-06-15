<script setup lang="ts">
import { BookOpen, FileText, GitBranch, NotebookText, Settings, Tags } from "@lucide/vue";
import appIcon from "../assets/icon.png";
import packageJson from "../../package.json";

type ViewKey = "excerpts" | "notes" | "topics" | "tags" | "books" | "settings";

defineProps<{
  activeView: ViewKey;
}>();

defineEmits<{
  selectView: [view: ViewKey];
}>();

const appVersion = packageJson.version;
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-brand">
      <img class="sidebar-brand-icon" :src="appIcon" alt="" aria-hidden="true" />
      <h1>摘抄库</h1>
    </div>

    <nav>
      <button
        class="nav-item"
        :class="{ active: activeView === 'excerpts' }"
        @click="$emit('selectView', 'excerpts')"
      >
        <FileText class="nav-item-icon" aria-hidden="true" />
        <span>摘抄</span>
      </button>
      <button
        class="nav-item"
        :class="{ active: activeView === 'notes' }"
        @click="$emit('selectView', 'notes')"
      >
        <NotebookText class="nav-item-icon" aria-hidden="true" />
        <span>笔记</span>
      </button>
      <button
        class="nav-item"
        :class="{ active: activeView === 'topics' }"
        @click="$emit('selectView', 'topics')"
      >
        <GitBranch class="nav-item-icon" aria-hidden="true" />
        <span>主题</span>
      </button>
      <div class="nav-separator" aria-hidden="true" />
      <button
        class="nav-item"
        :class="{ active: activeView === 'tags' }"
        @click="$emit('selectView', 'tags')"
      >
        <Tags class="nav-item-icon" aria-hidden="true" />
        <span>标签</span>
      </button>
      <button
        class="nav-item"
        :class="{ active: activeView === 'books' }"
        @click="$emit('selectView', 'books')"
      >
        <BookOpen class="nav-item-icon" aria-hidden="true" />
        <span>书籍</span>
      </button>
      <div class="nav-separator" aria-hidden="true" />
      <button
        class="nav-item"
        :class="{ active: activeView === 'settings' }"
        @click="$emit('selectView', 'settings')"
      >
        <Settings class="nav-item-icon" aria-hidden="true" />
        <span>设置</span>
      </button>
    </nav>

    <p class="sidebar-version">v{{ appVersion }}</p>
  </aside>
</template>
