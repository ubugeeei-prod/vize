<script setup lang="ts">
import { computed, ref, watch } from "vue";

import { useMouse } from "../composables/useMouse.js";
import { getLastRenderLayout, type DOMElement } from "../layoutMetrics.js";
import type { MouseEvent } from "../protocol.js";
import { Box } from "./Box.js";
import { Text } from "./Text.js";
import type { ScrollBarProps } from "./scroll-types.js";

const {
  itemCount,
  viewportSize,
  offset,
  length,
  orientation = "vertical",
  track = "│",
  thumb = "█",
  fg,
  thumbFg,
  disabled = false,
} = defineProps<ScrollBarProps>();

const emit = defineEmits<{
  "update:offset": [offset: number];
}>();

const root = ref<DOMElement | null>(null);
const dragGrabOffset = ref<number | null>(null);

watch(
  () => disabled,
  (value) => {
    if (value) dragGrabOffset.value = null;
  },
);

function nonNegativeInteger(value: number): number {
  const floored = Math.floor(value);
  return Number.isSafeInteger(floored) && floored >= 0 ? floored : 0;
}

const metrics = computed(() => {
  const trackLength = nonNegativeInteger(length ?? viewportSize);
  const boundedViewportSize = nonNegativeInteger(viewportSize);
  const boundedItemCount = nonNegativeInteger(itemCount);
  const maxOffset = Math.max(0, boundedItemCount - boundedViewportSize);
  const boundedOffset = Number.isFinite(offset) ? Math.min(maxOffset, Math.max(0, offset)) : 0;
  const thumbSize =
    maxOffset === 0
      ? trackLength
      : Math.max(
          1,
          Math.min(trackLength, Math.floor((trackLength * boundedViewportSize) / boundedItemCount)),
        );
  const travel = trackLength - thumbSize;
  const thumbStart = maxOffset === 0 ? 0 : Math.round((boundedOffset / maxOffset) * travel);
  return {
    trackLength,
    boundedViewportSize,
    boundedItemCount,
    maxOffset,
    boundedOffset,
    thumbSize,
    travel,
    thumbStart,
  };
});

const cells = computed(() => {
  const { trackLength, thumbSize, thumbStart } = metrics.value;
  if (trackLength === 0) return [];
  return Array.from({ length: trackLength }, (_, position) => ({
    position,
    symbol: position >= thumbStart && position < thumbStart + thumbSize ? thumb : track,
    thumb: position >= thumbStart && position < thumbStart + thumbSize,
  }));
});

const ariaState = computed(() => ({
  orientation,
  valueMin: 0,
  valueMax: metrics.value.maxOffset,
  valueNow: metrics.value.boundedOffset,
}));
const accessibilityProps = computed(() => ({
  ariaRole: "scrollbar" as const,
  ariaState: ariaState.value,
}));
const cellAccessibilityProps = { ariaHidden: true } as const;

function pointerPosition(event: MouseEvent, allowOutsideTrack: boolean): number | undefined {
  const layout = getLastRenderLayout(root.value);
  if (!layout) return undefined;

  const along = orientation === "vertical" ? event.y - layout.y : event.x - layout.x;
  const across = orientation === "vertical" ? event.x - layout.x : event.y - layout.y;
  const acrossSize = orientation === "vertical" ? layout.width : layout.height;
  if (!allowOutsideTrack && (across < 0 || across >= acrossSize)) return undefined;
  if (!allowOutsideTrack && (along < 0 || along >= metrics.value.trackLength)) return undefined;
  return along;
}

function offsetAt(pointer: number, grabOffset: number): number {
  const { maxOffset, travel } = metrics.value;
  if (maxOffset === 0 || travel === 0) return 0;
  const thumbStart = Math.min(travel, Math.max(0, pointer - grabOffset));
  return Math.round((thumbStart / travel) * maxOffset);
}

function publishPointerOffset(pointer: number, grabOffset: number): void {
  const nextOffset = offsetAt(pointer, grabOffset);
  if (nextOffset !== metrics.value.boundedOffset) emit("update:offset", nextOffset);
}

useMouse(
  (event) => {
    if (disabled) {
      dragGrabOffset.value = null;
      return;
    }
    if (event.button !== "left") return;

    if (event.kind === "down") {
      const pointer = pointerPosition(event, false);
      if (pointer === undefined) return;
      const { thumbStart, thumbSize } = metrics.value;
      const insideThumb = pointer >= thumbStart && pointer < thumbStart + thumbSize;
      const grabOffset = insideThumb ? pointer - thumbStart : Math.floor(thumbSize / 2);
      dragGrabOffset.value = grabOffset;
      publishPointerOffset(pointer, grabOffset);
      return;
    }

    if (event.kind === "drag" && dragGrabOffset.value !== null) {
      const pointer = pointerPosition(event, true);
      if (pointer !== undefined) publishPointerOffset(pointer, dragGrabOffset.value);
      return;
    }

    if (event.kind === "up") dragGrabOffset.value = null;
  },
  { isActive: computed(() => !disabled) },
);
</script>

<template>
  <Box
    ref="root"
    :flex-direction="orientation === 'vertical' ? 'column' : 'row'"
    v-bind="accessibilityProps"
  >
    <Text
      v-for="cell in cells"
      :key="cell.position"
      :content="cell.symbol"
      :fg="cell.thumb ? (thumbFg ?? fg) : fg"
      v-bind="cellAccessibilityProps"
    />
  </Box>
</template>
