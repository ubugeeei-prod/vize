<script setup lang="ts">
import { computed, watch } from "vue";

import { useInput } from "../composables/useInput.js";
import { useMouse } from "../composables/useMouse.js";
import {
  createVirtualViewport,
  type VirtualScrollAlignment,
  type VirtualViewportRange,
} from "../composables/useVirtualViewport.js";
import { Box } from "./Box.js";
import ScrollBar from "./ScrollBar.vue";
import type { ScrollBoxExposed, ScrollBoxItemSlot, ScrollBoxProps } from "./scroll-types.js";

const {
  itemCount,
  viewportSize,
  modelValue,
  overscan = 2,
  focused = false,
  disabled = false,
  wheelStep = 3,
  showScrollbar = true,
  followOutput = false,
  scrollbarTrack = "│",
  scrollbarThumb = "█",
} = defineProps<ScrollBoxProps>();

const emit = defineEmits<{
  "update:modelValue": [offset: number];
  "scroll": [range: VirtualViewportRange];
}>();

defineSlots<{
  /** Rows retained by the virtual viewport. */
  default(props: ScrollBoxItemSlot): unknown;
  /** Content kept mounted before the viewport. */
  start(): unknown;
  /** Content kept mounted after the viewport. */
  end(): unknown;
}>();

const viewport = createVirtualViewport({
  itemCount,
  viewportSize,
  overscan,
  initialOffset: modelValue ?? 0,
});
const isActive = computed(() => focused && !disabled);
const normalizedWheelStep = computed(() => {
  const truncated = Math.trunc(wheelStep);
  return Number.isSafeInteger(truncated) && truncated > 0 ? truncated : 1;
});
const retainedIndexes = computed(() => {
  const range = viewport.range.value;
  return Array.from(
    { length: range.overscanEnd - range.overscanStart },
    (_, index) => range.overscanStart + index,
  );
});
const rootAriaState = computed(() => ({
  disabled,
}));
const rootAccessibilityProps = computed(() => ({
  ariaRole: "list" as const,
  ariaState: rootAriaState.value,
}));

function publish(changed: boolean): boolean {
  if (!changed) return false;
  emit("update:modelValue", viewport.offset.value);
  emit("scroll", viewport.range.value);
  return true;
}

function scrollTo(offset: number): boolean {
  return publish(viewport.scrollTo(offset));
}

function scrollBy(delta: number): boolean {
  return publish(viewport.scrollBy(delta));
}

function scrollIntoView(index: number, alignment: VirtualScrollAlignment = "nearest"): boolean {
  return publish(viewport.scrollIntoView(index, alignment));
}

function isVisible(index: number): boolean {
  const range = viewport.range.value;
  return index >= range.visibleStart && index < range.visibleEnd;
}

function itemSlot(index: number): ScrollBoxItemSlot {
  return { ...viewport.range.value, index, visible: isVisible(index) };
}

watch(
  [() => itemCount, () => viewportSize, () => modelValue],
  (
    [itemCount, viewportSize, modelValue],
    [previousItemCount, previousViewportSize, previousModelValue],
  ) => {
    const itemCountChanged = itemCount !== previousItemCount;
    const viewportSizeChanged = viewportSize !== previousViewportSize;
    const modelValueChanged = modelValue !== previousModelValue;
    const followedEnd = itemCountChanged && followOutput && viewport.atEnd.value;
    const previousOffset = viewport.offset.value;

    if (itemCountChanged) viewport.setItemCount(itemCount);
    if (viewportSizeChanged) viewport.setViewportSize(viewportSize);
    if (followedEnd && !modelValueChanged) viewport.scrollTo(viewport.maxOffset.value);
    if (
      modelValue !== undefined &&
      (modelValueChanged || ((itemCountChanged || viewportSizeChanged) && !followedEnd))
    ) {
      viewport.scrollTo(modelValue);
    }

    const offsetChanged = previousOffset !== viewport.offset.value;
    const requestedOffsetWasClamped =
      modelValue !== undefined && modelValue !== viewport.offset.value;
    if (requestedOffsetWasClamped || (offsetChanged && !modelValueChanged)) {
      emit("update:modelValue", viewport.offset.value);
    }
    if (offsetChanged && (itemCountChanged || viewportSizeChanged || requestedOffsetWasClamped)) {
      emit("scroll", viewport.range.value);
    }
  },
);
watch(
  () => overscan,
  (value) => viewport.setOverscan(value),
);

useInput(
  (_input, key) => {
    if (key.eventType === "release") return;
    if (key.upArrow) scrollBy(-1);
    else if (key.downArrow) scrollBy(1);
    else if (key.pageUp) scrollBy(-Math.max(1, viewportSize - 1));
    else if (key.pageDown) scrollBy(Math.max(1, viewportSize - 1));
    else if (key.home) scrollTo(0);
    else if (key.end) scrollTo(viewport.maxOffset.value);
  },
  { isActive },
);
useMouse(
  (event) => {
    if (event.kind === "scroll-up") scrollBy(-normalizedWheelStep.value);
    else if (event.kind === "scroll-down") scrollBy(normalizedWheelStep.value);
  },
  { isActive },
);

const exposed: ScrollBoxExposed = {
  scrollTo,
  scrollBy,
  scrollIntoView,
  range: () => viewport.range.value,
};
defineExpose(exposed);
</script>

<template>
  <Box flex-direction="column" v-bind="rootAccessibilityProps">
    <slot name="start" />
    <Box :flex-direction="showScrollbar ? 'row' : 'column'">
      <Box flex-direction="column" :height="viewportSize" overflow="hidden">
        <Box
          v-for="index in retainedIndexes"
          :key="index"
          :display="isVisible(index) ? 'flex' : 'none'"
          :aria-hidden="!isVisible(index)"
          :data-scroll-index="index"
        >
          <slot v-bind="itemSlot(index)" />
        </Box>
      </Box>
      <ScrollBar
        v-if="showScrollbar && itemCount > viewportSize"
        :item-count
        :viewport-size
        :offset="viewport.offset.value"
        :track="scrollbarTrack"
        :thumb="scrollbarThumb"
        :disabled
        @update:offset="scrollTo"
      />
    </Box>
    <slot name="end" />
  </Box>
</template>
