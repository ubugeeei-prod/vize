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

const cells = computed(() => {
  const trackLength = Math.max(0, Math.floor(length ?? viewportSize));
  if (trackLength === 0) return [];
  const boundedViewportSize = Math.max(0, Math.floor(viewportSize));
  const boundedItemCount = Math.max(0, Math.floor(itemCount));
  const maxOffset = Math.max(0, boundedItemCount - boundedViewportSize);
  const thumbSize =
    maxOffset === 0
      ? trackLength
      : Math.max(
          1,
          Math.min(trackLength, Math.floor((trackLength * boundedViewportSize) / boundedItemCount)),
        );
  const travel = trackLength - thumbSize;
  const thumbStart =
    maxOffset === 0 ? 0 : Math.round((Math.min(maxOffset, offset) / maxOffset) * travel);
  return Array.from({ length: trackLength }, (_, position) => ({
    position,
    symbol: position >= thumbStart && position < thumbStart + thumbSize ? thumb : track,
    thumb: position >= thumbStart && position < thumbStart + thumbSize,
  }));
});

const ariaState = computed(() => ({
  orientation,
  valueMin: 0,
  valueMax: Math.max(0, itemCount - viewportSize),
  valueNow: offset,
}));
const accessibilityProps = computed(() => ({
  ariaRole: "scrollbar" as const,
  ariaState: ariaState.value,
}));
</script>

<template>
  <Box :flex-direction="orientation === 'vertical' ? 'column' : 'row'" v-bind="accessibilityProps">
    <Text
      v-for="cell in cells"
      :key="cell.position"
      :content="cell.symbol"
      :fg="cell.thumb ? (thumbFg ?? fg) : fg"
    />
  </Box>
</template>
