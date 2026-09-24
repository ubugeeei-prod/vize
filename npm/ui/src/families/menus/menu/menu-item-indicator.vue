<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { menuItemIndicatorContext } from "./menu-context.ts";
import type { MenuElementExpose, MenuItemIndicatorSlotState } from "./menu-types.ts";

const { forceMount = false } = defineProps<{
  /**
   * Keep the indicator visible and its slot rendered while unchecked, for exit
   * animations or glyphs that style the unchecked state themselves.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Indicator glyph. Receives the owning item's checked-state token. */
  default(props: MenuItemIndicatorSlotState): unknown;
}>();

const indicator = menuItemIndicatorContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");
const visible = computed(() => forceMount || indicator.state.value !== "unchecked");

defineExpose({ element } satisfies Record<keyof MenuElementExpose, unknown>);
</script>

<template>
  <span
    ref="element"
    aria-hidden="true"
    data-vize-ui="menu-item-indicator"
    part="item-indicator"
    :hidden="visible ? undefined : true"
    :data-state="indicator.state.value"
  >
    <slot v-if="visible" :state="indicator.state.value" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
