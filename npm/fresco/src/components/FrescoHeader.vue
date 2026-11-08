<script setup lang="ts">
import type { HeaderProps } from "./presentational-primitive-types.js";

const {
  title,
  subtitle,
  left,
  right,
  bg,
  titleFg = "white",
  subtitleFg = "gray",
  borderBottom = false,
} = defineProps<HeaderProps>();

defineOptions({ name: "FrescoHeader" });

const rootStyle = { flexDirection: "column" } as const;
const contentStyle = {
  flexDirection: "row",
  justifyContent: "space-between",
  alignItems: "center",
  width: "100%",
  padding: 1,
} as const;
const rowStyle = { flexDirection: "row" } as const;
const borderLabel = "─".repeat(80);
</script>

<template>
  <div :style="rootStyle">
    <div :bg :style="contentStyle">
      <div :style="rowStyle">
        <slot name="left"
          ><text v-if="left">{{ left }}</text></slot
        >
      </div>
      <div :style="rowStyle">
        <text :fg="titleFg" :bold="true">{{ title }}</text>
        <text v-if="subtitle" :fg="subtitleFg" :dim="true"> - {{ subtitle }}</text>
      </div>
      <div :style="rowStyle">
        <slot name="right"
          ><text v-if="right">{{ right }}</text></slot
        >
      </div>
    </div>
    <text v-if="borderBottom" :dim="true">{{ borderLabel }}</text>
  </div>
</template>
