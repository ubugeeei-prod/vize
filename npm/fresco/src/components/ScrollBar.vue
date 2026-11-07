<script setup lang="ts">
import { computed } from "vue";

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
} = defineProps<ScrollBarProps>();

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
  return { trackLength, boundedViewportSize, boundedItemCount, maxOffset, boundedOffset };
});

const cells = computed(() => {
  const { trackLength, boundedViewportSize, boundedItemCount, maxOffset, boundedOffset } =
    metrics.value;
  if (trackLength === 0) return [];
  const thumbSize =
    maxOffset === 0
      ? trackLength
      : Math.max(
          1,
          Math.min(trackLength, Math.floor((trackLength * boundedViewportSize) / boundedItemCount)),
        );
  const travel = trackLength - thumbSize;
  const thumbStart = maxOffset === 0 ? 0 : Math.round((boundedOffset / maxOffset) * travel);
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
</script>

<template>
  <Box :flex-direction="orientation === 'vertical' ? 'column' : 'row'" v-bind="accessibilityProps">
    <Text
      v-for="cell in cells"
      :key="cell.position"
      :content="cell.symbol"
      :fg="cell.thumb ? (thumbFg ?? fg) : fg"
      v-bind="cellAccessibilityProps"
    />
  </Box>
</template>
