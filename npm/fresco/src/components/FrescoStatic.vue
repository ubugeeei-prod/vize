<script setup lang="ts" generic="T = unknown">
import { computed } from "vue";

import type { FrescoStyle } from "../protocol.js";

const { items = [], style } = defineProps<{
  items?: T[];
  style?: FrescoStyle;
}>();

defineSlots<{
  default?: (props: { item: T; index: number }) => unknown;
}>();

defineOptions({ name: "Static" });

const containerStyle = computed(() => ({
  position: "absolute" as const,
  flexDirection: "column" as const,
  ...style,
}));

const itemStyle = { flexDirection: "column" } as const;
const renderedItems = computed(() =>
  items.map((item, index) => ({ item, index, key: `static-${index}` })),
);
</script>

<template>
  <div :internal_static="true" :style="containerStyle">
    <div
      v-for="entry in renderedItems"
      :key="entry.key"
      :internal_static_item="true"
      :style="itemStyle"
    >
      <slot :item="entry.item" :index="entry.index" />
    </div>
  </div>
</template>
