<script setup lang="ts">
import { computed, type CSSProperties } from "vue";

import { stackAlignValues, stackJustifyValues, type StackProps } from "./layout-primitive-types.js";

const {
  direction = "vertical",
  gap = 0,
  align = "stretch",
  justify = "start",
  wrap = false,
} = defineProps<StackProps>();

defineOptions({ name: "FrescoStack" });

const stackStyle = computed<CSSProperties>(() => ({
  flexDirection: direction === "horizontal" ? "row" : "column",
  gap,
  alignItems: stackAlignValues[align],
  justifyContent: stackJustifyValues[justify],
  flexWrap: wrap ? "wrap" : "nowrap",
}));
</script>

<template>
  <div :style="stackStyle"><slot /></div>
</template>
