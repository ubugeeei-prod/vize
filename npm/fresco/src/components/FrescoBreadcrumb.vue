<script setup lang="ts">
import type { BreadcrumbItem, BreadcrumbProps } from "./presentational-primitive-types.js";

const {
  items,
  separator = " > ",
  fg = "gray",
  activeFg = "white",
  separatorFg = "gray",
} = defineProps<BreadcrumbProps>();
defineEmits<{ select: [key: string] }>();

defineOptions({ name: "FrescoBreadcrumb" });

const rowStyle = { flexDirection: "row" } as const;
const iconLabel = (item: BreadcrumbItem) => `${item.icon} `;
</script>

<template>
  <div :style="rowStyle">
    <template v-for="(item, index) in items" :key="item.key">
      <text v-if="item.icon" :fg="index === items.length - 1 ? activeFg : fg">{{
        iconLabel(item)
      }}</text>
      <text
        :fg="index === items.length - 1 ? activeFg : fg"
        :bold="index === items.length - 1"
        :underline="index !== items.length - 1"
        >{{ item.label }}</text
      >
      <text v-if="index !== items.length - 1" :fg="separatorFg">{{ separator }}</text>
    </template>
  </div>
</template>
