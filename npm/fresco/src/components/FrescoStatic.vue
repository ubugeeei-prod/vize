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
</script>

<template>
  <div :internal_static="true" :style="containerStyle">
    <div
      v-for="(item, index) in items"
      :key="index"
      :internal_static_item="true"
      :style="itemStyle"
    >
      <slot :item="item" :index="index" />
    </div>
  </div>
</template>
