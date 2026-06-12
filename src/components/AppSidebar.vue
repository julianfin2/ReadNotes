<script setup lang="ts">
import appIcon from "../assets/icon.png";
import packageJson from "../../package.json";

type ViewKey = "excerpts" | "topics" | "management" | "settings";

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
        :class="{ active: activeView === 'management' }"
        @click="$emit('selectView', 'management')"
      >
        管理
      </button>
      <button
        class="nav-item"
        :class="{ active: activeView === 'settings' }"
        @click="$emit('selectView', 'settings')"
      >
        设置
      </button>
    </nav>

    <p class="sidebar-version">v{{ appVersion }}</p>
  </aside>
</template>
