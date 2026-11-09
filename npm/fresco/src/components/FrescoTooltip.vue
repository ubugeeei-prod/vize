<script setup lang="ts">
import { computed } from "vue";

import { Box } from "./Box.js";
import type { TooltipProps } from "./presentational-primitive-types.js";
import { Text } from "./Text.js";

interface TooltipPanelPosition {
  readonly top?: string;
  readonly right?: string;
  readonly bottom?: string;
  readonly left?: string;
  readonly marginTop?: number;
  readonly marginRight?: number;
  readonly marginBottom?: number;
  readonly marginLeft?: number;
}

const {
  text,
  visible = true,
  position = "top",
  offset = 0,
  border = "rounded",
  bg = "white",
  fg = "black",
} = defineProps<TooltipProps>();

defineSlots<{
  default?: () => unknown;
}>();

defineOptions({ name: "FrescoTooltip" });

const panelPosition = computed<TooltipPanelPosition>(() => {
  const gap = Number.isFinite(offset) ? Math.max(0, offset) : 0;
  switch (position) {
    case "bottom":
      return { top: "100%", marginTop: gap };
    case "right":
      return { left: "100%", marginLeft: gap };
    case "left":
      return { right: "100%", marginRight: gap };
    case "top":
    default:
      return { bottom: "100%", marginBottom: gap };
  }
});
</script>

<template>
  <Box position="relative">
    <slot />
    <Box
      v-if="visible"
      position="absolute"
      :top="panelPosition.top"
      :right="panelPosition.right"
      :bottom="panelPosition.bottom"
      :left="panelPosition.left"
      :margin-top="panelPosition.marginTop"
      :margin-right="panelPosition.marginRight"
      :margin-bottom="panelPosition.marginBottom"
      :margin-left="panelPosition.marginLeft"
      :border="border === 'none' ? undefined : border"
      :bg
      :fg
      :padding-x="1"
    >
      <Text :content="text" :fg :bg />
    </Box>
  </Box>
</template>
