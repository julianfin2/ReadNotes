<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { Tag } from "../types/tag";

const props = defineProps<{
  tags: Tag[];
}>();

const container = ref<HTMLElement | null>(null);
const measureBox = ref<HTMLElement | null>(null);
const visibleCount = ref(props.tags.length);
let resizeObserver: ResizeObserver | null = null;

const hiddenCount = computed(() => Math.max(0, props.tags.length - visibleCount.value));
const visibleTags = computed(() => props.tags.slice(0, visibleCount.value));
const title = computed(() => props.tags.map((tag) => `#${tag.name}`).join(" "));

watch(
  () => props.tags,
  () => scheduleMeasure(),
  { deep: true },
);

onMounted(() => {
  resizeObserver = new ResizeObserver(() => scheduleMeasure());
  if (container.value) {
    resizeObserver.observe(container.value);
  }
  scheduleMeasure();
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
});

function scheduleMeasure() {
  void nextTick(() => {
    requestAnimationFrame(measure);
  });
}

function measure() {
  const containerWidth = container.value?.clientWidth ?? 0;
  const measurer = measureBox.value;

  if (!containerWidth || !measurer || props.tags.length === 0) {
    visibleCount.value = props.tags.length;
    return;
  }

  const gap = 6;
  const tagWidths = Array.from(
    measurer.querySelectorAll<HTMLElement>("[data-tag-measure]"),
  ).map((element) => element.offsetWidth);

  const moreWidths = new Map<number, number>();
  for (const element of measurer.querySelectorAll<HTMLElement>("[data-more-measure]")) {
    moreWidths.set(Number(element.dataset.moreMeasure), element.offsetWidth);
  }

  const totalTagWidth = tagWidths.reduce((total, width) => total + width, 0);
  const totalGapWidth = Math.max(0, tagWidths.length - 1) * gap;
  if (totalTagWidth + totalGapWidth <= containerWidth) {
    visibleCount.value = props.tags.length;
    return;
  }

  for (let count = props.tags.length - 1; count >= 0; count -= 1) {
    const remaining = props.tags.length - count;
    const moreWidth = moreWidths.get(remaining) ?? 0;
    const visibleWidth = tagWidths.slice(0, count).reduce((total, width) => total + width, 0);
    const itemCount = count + 1;
    const width = visibleWidth + moreWidth + Math.max(0, itemCount - 1) * gap;

    if (width <= containerWidth || count === 0) {
      visibleCount.value = count;
      return;
    }
  }
}

function tagStyle(tag: Tag) {
  if (!tag.color) {
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
    return normalized;
  }

  const value = hex[1].length === 3
    ? hex[1].split("").map((part) => part + part).join("")
    : hex[1];
  const red = Number.parseInt(value.slice(0, 2), 16);
  const green = Number.parseInt(value.slice(2, 4), 16);
  const blue = Number.parseInt(value.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, 0.14)`;
}
</script>

<template>
  <span ref="container" class="table-tags responsive-tag-list" :title="title">
    <span
      v-for="tag in visibleTags"
      :key="tag.id"
      class="table-tag"
      :style="tagStyle(tag)"
    >
      #{{ tag.name }}
    </span>
    <span v-if="hiddenCount > 0" class="table-tag-more">+{{ hiddenCount }}</span>

    <span ref="measureBox" class="responsive-tag-measure" aria-hidden="true">
      <span
        v-for="tag in tags"
        :key="`measure-tag-${tag.id}`"
        class="table-tag"
        :style="tagStyle(tag)"
        data-tag-measure
      >
        #{{ tag.name }}
      </span>
      <span
        v-for="count in tags.length"
        :key="`measure-more-${count}`"
        class="table-tag-more"
        :data-more-measure="count"
      >
        +{{ count }}
      </span>
    </span>
  </span>
</template>
