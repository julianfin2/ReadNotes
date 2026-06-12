<script setup lang="ts">
import { computed } from "vue";

type TagColorPreset = {
  name: string;
  value: string;
};

const defaultTagColor = "#2e6f62";
const defaultPresets: TagColorPreset[] = [
  { name: "松绿", value: "#2e6f62" },
  { name: "湖蓝", value: "#2f748f" },
  { name: "靛蓝", value: "#4f6f9f" },
  { name: "紫藤", value: "#7a5c9e" },
  { name: "陶红", value: "#b05d4f" },
  { name: "琥珀", value: "#c28a2c" },
  { name: "橄榄", value: "#6b7b3d" },
  { name: "石墨", value: "#5f6b72" },
];

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const normalizedValue = computed(() => props.modelValue.trim().toLowerCase());
const pickerValue = computed(() => props.modelValue || defaultTagColor);

function setColor(value: string) {
  emit("update:modelValue", value);
}

function updateFromPicker(event: Event) {
  const input = event.target as HTMLInputElement;
  setColor(input.value);
}
</script>

<template>
  <div class="tag-color-field">
    <div class="tag-color-presets" aria-label="预设颜色">
      <button
        v-for="preset in defaultPresets"
        :key="preset.value"
        class="tag-color-preset"
        :class="{ active: normalizedValue === preset.value.toLowerCase() }"
        :style="{ backgroundColor: preset.value }"
        type="button"
        :aria-label="`选择${preset.name}`"
        :title="preset.name"
        @click="setColor(preset.value)"
      />
    </div>

    <div class="tag-color-custom-row">
      <label class="tag-color-custom-picker">
        <span>自定义</span>
        <input type="color" :value="pickerValue" @input="updateFromPicker" />
      </label>
      <span class="tag-color-value">{{ modelValue || "未设置" }}</span>
      <button class="text-action" type="button" @click="setColor('')">清除</button>
    </div>
  </div>
</template>
