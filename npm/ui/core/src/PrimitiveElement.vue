<script setup lang="ts">
import { useSlots, useTemplateRef } from "vue";

import type { PrimitiveAs, PrimitiveElement } from "./primitive.ts";

const { as = "div" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;
}>();

const slots = useSlots();
const element = useTemplateRef<PrimitiveElement>("element");

function getSlotNames(): string[] {
  return Object.keys(slots);
}

defineExpose({ element });
</script>

<template>
  <component :is="as" ref="element" data-vize-ui="primitive">
    <template v-for="name in getSlotNames()" #[name]>
      <slot :name="name" />
    </template>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
