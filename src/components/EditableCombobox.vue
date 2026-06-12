<script setup lang="ts">
import { computed, ref } from "vue";

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
  isOpen.value = true;
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
</script>

<template>
  <div class="editable-combobox">
    <input
      :value="modelValue"
      :placeholder="placeholder"
      autocomplete="off"
      @blur="closeLater"
      @focus="isOpen = true"
      @input="updateValue(($event.target as HTMLInputElement).value)"
    />

    <div v-if="isOpen && visibleOptions.length > 0" class="editable-combobox-menu">
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
  </div>
</template>
