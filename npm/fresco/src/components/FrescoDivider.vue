<script setup lang="ts">
import { computed } from "vue";

import type { DividerProps } from "./layout-primitive-types.js";

const { direction = "horizontal", char, title, fg = "gray", titleFg } = defineProps<DividerProps>();

defineOptions({ name: "FrescoDivider" });

const dividerChar = computed(() => char ?? (direction === "horizontal" ? "─" : "│"));
const titleLabel = computed(() => ` ${title} `);
const titleRowStyle = { flexDirection: "row", alignItems: "center" } as const;
</script>

<template>
  <text v-if="direction === 'vertical'" :fg>{{ dividerChar }}</text>
  <div v-else-if="title" :style="titleRowStyle">
    <text :fg>{{ dividerChar.repeat(3) }}</text>
    <text :fg="titleFg ?? fg" :bold="true">{{ titleLabel }}</text>
    <text :fg>{{ dividerChar.repeat(3) }}</text>
  </div>
  <text v-else :fg>{{ dividerChar.repeat(40) }}</text>
</template>
