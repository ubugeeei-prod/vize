<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="breadcrumb-item"
    part="item"
    :data-current="currentState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { BreadcrumbItemExpose, BreadcrumbItemSlotState } from "./breadcrumb-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const { as = "li", current = false } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "li"
   */
  readonly as?: PrimitiveAs;

  /**
   * Whether this item represents the current route segment.
   *
   * @default false
   */
  readonly current?: boolean;
}>();

defineSlots<{
  /** Renders item contents with current-route state. */
  default(props: BreadcrumbItemSlotState): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const currentState = computed(() => current);
const slotState = computed<BreadcrumbItemSlotState>(() => ({ current: currentState.value }));

type BreadcrumbItemSetupExpose = Omit<BreadcrumbItemExpose, "current" | "element"> & {
  readonly current: typeof currentState;
  readonly element: typeof element;
};

const exposed = {
  current: currentState,
  element,
} satisfies BreadcrumbItemSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Current-route color, truncation, and inline layout remain consumer-owned. */
</style>
