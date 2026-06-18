<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { X } from "@lucide/vue";
import type { Tag } from "../types/tag";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    tags: Tag[];
    placeholder?: string;
  }>(),
  {
    placeholder: "输入标签后按空格或回车",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const draft = ref("");
const tokens = ref<string[]>(parseTokens(props.modelValue));
const isComposing = ref(false);

const knownTags = computed(() => {
  return new Map(props.tags.map((tag) => [tag.name.toLocaleLowerCase(), tag]));
});

watch(
  () => props.modelValue,
  (value) => {
    const nextTokens = parseTokens(value);
    if (serializeTokens(nextTokens) !== serializeTokens(tokens.value)) {
      tokens.value = nextTokens;
    }
  },
);

function parseTokens(value: string) {
  const seen = new Set<string>();
  return value
    .split(/[\s,，#]+/)
    .map((token) => token.trim())
    .filter((token) => {
      if (!token) {
        return false;
      }

      const key = token.toLocaleLowerCase();
      if (seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    });
}

function serializeTokens(values: string[]) {
  return values.map((token) => `#${token}`).join(" ");
}

function emitTokens() {
  emit("update:modelValue", serializeTokens(tokens.value));
}

function commitDraft() {
  const nextTokens = parseTokens(draft.value);
  if (nextTokens.length === 0) {
    draft.value = "";
    return;
  }

  const existingKeys = new Set(tokens.value.map((token) => token.toLocaleLowerCase()));
  for (const token of nextTokens) {
    const key = token.toLocaleLowerCase();
    if (existingKeys.has(key)) {
      continue;
    }

    const knownTag = knownTags.value.get(key);
    tokens.value.push(knownTag?.name || token);
    existingKeys.add(key);
  }

  draft.value = "";
  emitTokens();
}

function handleKeydown(event: KeyboardEvent) {
  if (isComposing.value) {
    return;
  }

  if (event.key === "Enter" || event.key === " " || event.key === "," || event.key === "，") {
    event.preventDefault();
    commitDraft();
    return;
  }

  if (event.key === "Backspace" && !draft.value && tokens.value.length > 0) {
    tokens.value.pop();
    emitTokens();
  }
}

function handleInput() {
  if (isComposing.value || !/[\s,，]/.test(draft.value)) {
    return;
  }

  commitDraft();
}

function removeToken(index: number) {
  tokens.value.splice(index, 1);
  emitTokens();
  void nextTick(() => inputRef.value?.focus());
}

function focusInput() {
  inputRef.value?.focus();
}

function tagForToken(token: string) {
  return knownTags.value.get(token.toLocaleLowerCase()) || null;
}

function tokenStyle(token: string) {
  const tag = tagForToken(token);
  if (!tag?.color) {
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
    return color;
  }

  const value =
    hex[1].length === 3
      ? hex[1]
          .split("")
          .map((part) => part + part)
          .join("")
      : hex[1];
  const red = Number.parseInt(value.slice(0, 2), 16);
  const green = Number.parseInt(value.slice(2, 4), 16);
  const blue = Number.parseInt(value.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, 0.14)`;
}
</script>

<template>
  <div class="tag-token-input" @click="focusInput">
    <span
      v-for="(token, index) in tokens"
      :key="token.toLocaleLowerCase()"
      class="tag-token"
      :class="{ 'is-known': tagForToken(token) }"
      :style="tokenStyle(token)"
    >
      #{{ tagForToken(token)?.name || token }}
      <button type="button" :aria-label="`删除标签 ${token}`" @click.stop="removeToken(index)">
        <X aria-hidden="true" />
      </button>
    </span>
    <input
      ref="inputRef"
      v-model="draft"
      :placeholder="tokens.length === 0 ? placeholder : ''"
      @blur="commitDraft"
      @compositionstart="isComposing = true"
      @compositionend="isComposing = false; handleInput()"
      @input="handleInput"
      @keydown="handleKeydown"
    />
  </div>
</template>

<style scoped>
.tag-token-input {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  min-height: 40px;
  padding: 5px 8px;
  border: 1px solid #d6cfc2;
  border-radius: 6px;
  background: #fffdf9;
  cursor: text;
}

.tag-token-input:focus-within {
  border-color: #8a6552;
}

.tag-token-input input {
  flex: 1 1 140px;
  width: auto;
  min-width: 100px;
  min-height: 28px;
  padding: 0 4px;
  border: 0;
  background: transparent;
  outline: none;
}

.tag-token {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  max-width: 180px;
  padding: 2px 4px 2px 7px;
  border: 1px solid #ded7ca;
  border-radius: 999px;
  background: #f3f0e9;
  color: #596366;
  font-size: 0.78rem;
  font-weight: 700;
  line-height: 1.35;
}

.tag-token.is-known {
  border-color: var(--tag-background, transparent);
  background: var(--tag-background, #e8eee6);
  color: var(--tag-accent, #2e6f62);
}

.tag-token button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  width: 18px;
  height: 18px;
  padding: 0;
  border-radius: 999px;
  background: transparent;
  color: currentColor;
  cursor: pointer;
  opacity: 0.72;
}

.tag-token button:hover {
  background: rgba(37, 50, 56, 0.08);
  opacity: 1;
}

.tag-token svg {
  width: 12px;
  height: 12px;
}
</style>
