<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  modelValue: string;
  options: Array<{
    value: string;
    label: string;
  }>;
  placeholder?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  change: [value: string];
}>();

const open = ref(false);

function selectOption(value: string) {
  emit("update:modelValue", value);
  emit("change", value);
  open.value = false;
}

function handleBlur(event: FocusEvent) {
  const nextTarget = event.relatedTarget;

  if (nextTarget instanceof Node && event.currentTarget instanceof Node) {
    if (event.currentTarget.contains(nextTarget)) {
      return;
    }
  }

  open.value = false;
}
</script>

<template>
  <div class="custom-select" @focusout="handleBlur">
    <button
      class="custom-select-trigger"
      type="button"
      aria-haspopup="listbox"
      :aria-expanded="open"
      @click="open = !open"
    >
      <span>
        {{
          options.find((option) => option.value === modelValue)?.label ||
          placeholder ||
          "请选择"
        }}
      </span>
      <span class="custom-select-caret" aria-hidden="true"></span>
    </button>

    <div v-if="open" class="custom-select-menu" role="listbox">
      <button
        v-for="option in options"
        :key="option.value"
        class="custom-select-option"
        :class="{ active: option.value === modelValue }"
        type="button"
        role="option"
        :aria-selected="option.value === modelValue"
        @click="selectOption(option.value)"
      >
        {{ option.label }}
      </button>
    </div>
  </div>
</template>
