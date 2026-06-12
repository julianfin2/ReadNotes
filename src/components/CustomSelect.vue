<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";

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
const rootRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLButtonElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const menuRect = ref({
  left: 0,
  top: 0,
  width: 0,
  maxHeight: 240,
});

const menuStyle = computed(() => ({
  left: `${menuRect.value.left}px`,
  top: `${menuRect.value.top}px`,
  width: `${menuRect.value.width}px`,
  maxHeight: `${menuRect.value.maxHeight}px`,
}));

function selectOption(value: string) {
  emit("update:modelValue", value);
  emit("change", value);
  open.value = false;
}

async function toggleMenu() {
  open.value = !open.value;

  if (open.value) {
    await nextTick();
    updateMenuPosition();
  }
}

function updateMenuPosition() {
  const trigger = triggerRef.value;

  if (!trigger) {
    return;
  }

  const rect = trigger.getBoundingClientRect();
  const viewportPadding = 12;
  const gap = 6;
  const preferredHeight = 240;
  const availableBelow = window.innerHeight - rect.bottom - gap - viewportPadding;
  const availableAbove = rect.top - gap - viewportPadding;
  const shouldOpenUp = availableBelow < 160 && availableAbove > availableBelow;
  const availableSpace = shouldOpenUp ? availableAbove : availableBelow;
  const maxHeight = Math.max(120, Math.min(preferredHeight, availableSpace));

  menuRect.value = {
    left: rect.left,
    top: shouldOpenUp ? rect.top - gap - maxHeight : rect.bottom + gap,
    width: rect.width,
    maxHeight,
  };
}

function handleDocumentPointerDown(event: PointerEvent) {
  const target = event.target;

  if (!(target instanceof Node)) {
    return;
  }

  if (rootRef.value?.contains(target) || menuRef.value?.contains(target)) {
    return;
  }

  open.value = false;
}

function handleEscape(event: KeyboardEvent) {
  if (event.key === "Escape") {
    open.value = false;
  }
}

function handleViewportChange() {
  if (open.value) {
    updateMenuPosition();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", handleDocumentPointerDown);
  document.addEventListener("keydown", handleEscape);
  window.addEventListener("resize", handleViewportChange);
  window.addEventListener("scroll", handleViewportChange, true);
});

onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", handleDocumentPointerDown);
  document.removeEventListener("keydown", handleEscape);
  window.removeEventListener("resize", handleViewportChange);
  window.removeEventListener("scroll", handleViewportChange, true);
});
</script>

<template>
  <div ref="rootRef" class="custom-select">
    <button
      ref="triggerRef"
      class="custom-select-trigger"
      type="button"
      aria-haspopup="listbox"
      :aria-expanded="open"
      @click="toggleMenu"
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
  </div>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menuRef"
      class="custom-select-menu"
      role="listbox"
      :style="menuStyle"
    >
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
  </Teleport>
</template>
