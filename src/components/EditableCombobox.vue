<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from "vue";

const props = defineProps<{
  modelValue: string;
  options: string[];
  placeholder?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  select: [value: string];
}>();

const isOpen = ref(false);
const inputEl = ref<HTMLInputElement | null>(null);
const menuStyle = ref<Record<string, string>>({});

const visibleOptions = computed(() => {
  const query = props.modelValue.trim().toLowerCase();
  const unique = Array.from(new Set(props.options.filter(Boolean)));
  if (!query) {
    return unique.slice(0, 8);
  }

  return unique
    .filter((option) => option.toLowerCase().includes(query))
    .slice(0, 8);
});

function updateValue(value: string) {
  emit("update:modelValue", value);
  openMenu();
}

function selectOption(value: string) {
  emit("update:modelValue", value);
  emit("select", value);
  isOpen.value = false;
}

function closeLater() {
  window.setTimeout(() => {
    isOpen.value = false;
  }, 120);
}

function openMenu() {
  isOpen.value = true;
  void nextTick(updateMenuPosition);
}

function updateMenuPosition() {
  if (!inputEl.value) {
    return;
  }

  const rect = inputEl.value.getBoundingClientRect();
  const gap = 6;
  const maxHeight = Math.min(220, window.innerHeight - rect.bottom - gap - 12);
  menuStyle.value = {
    left: `${rect.left}px`,
    top: `${rect.bottom + gap}px`,
    width: `${rect.width}px`,
    maxHeight: `${Math.max(120, maxHeight)}px`,
  };
}

window.addEventListener("resize", updateMenuPosition);
window.addEventListener("scroll", updateMenuPosition, true);

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateMenuPosition);
  window.removeEventListener("scroll", updateMenuPosition, true);
});
</script>

<template>
  <div class="editable-combobox">
    <input
      ref="inputEl"
      :value="modelValue"
      :placeholder="placeholder"
      autocomplete="off"
      @blur="closeLater"
      @focus="openMenu"
      @input="updateValue(($event.target as HTMLInputElement).value)"
    />

    <Teleport to="body">
      <div
        v-if="isOpen && visibleOptions.length > 0"
        class="editable-combobox-menu"
        :style="menuStyle"
      >
        <button
          v-for="option in visibleOptions"
          :key="option"
          class="editable-combobox-option"
          type="button"
          @mousedown.prevent="selectOption(option)"
        >
          {{ option }}
        </button>
      </div>
    </Teleport>
  </div>
</template>
