<script setup lang="ts">
defineProps<{
  open: boolean;
  title: string;
}>();

defineEmits<{
  close: [];
}>();
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" @click.self="$emit('close')">
      <section class="modal-panel" role="dialog" aria-modal="true" :aria-label="title">
        <header class="modal-header">
          <h2>{{ title }}</h2>
          <button class="icon-button" type="button" aria-label="关闭" @click="$emit('close')">
            ×
          </button>
        </header>

        <div class="modal-body">
          <slot />
        </div>
      </section>
    </div>
  </Teleport>
</template>
