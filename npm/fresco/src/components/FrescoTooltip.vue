<script setup lang="ts">
import { computed, type CSSProperties } from "vue";

import type { TooltipProps } from "./presentational-primitive-types.js";

const {
  text,
  visible = true,
  position = "top",
  border = "rounded",
  bg = "white",
  fg = "black",
} = defineProps<TooltipProps>();

defineOptions({ name: "FrescoTooltip" });

const containerStyle = computed<CSSProperties>(() => ({
  flexDirection: position === "left" || position === "right" ? "row" : "column",
}));
const tooltipStyle = { paddingLeft: 1, paddingRight: 1 } as const;
const contentComesFirst = computed(() => position === "bottom" || position === "right");
</script>

<template>
  <div v-if="visible" :style="containerStyle">
    <div v-if="contentComesFirst"><slot /></div>
    <div :border="border === 'none' ? undefined : border" :bg :fg :style="tooltipStyle">
      <text :fg :bg>{{ text }}</text>
    </div>
    <div v-if="!contentComesFirst"><slot /></div>
  </div>
  <div v-else><slot /></div>
</template>
