<template>
  <component :is="as" ref="element" :aria-label="labelState" data-vize-ui="breadcrumb" part="root">
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { BreadcrumbRootExpose, BreadcrumbRootSlotState } from "./breadcrumb-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "./primitive.ts";

const { as = "nav", label = "Breadcrumb" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "nav"
   */
  readonly as?: PrimitiveAs;

  /**
   * Accessible landmark label mirrored to `aria-label`.
   *
   * @default "Breadcrumb"
   */
  readonly label?: string;
}>();

defineSlots<{
  /** Renders breadcrumb list content with the resolved landmark label. */
  default(props: BreadcrumbRootSlotState): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const labelState = computed(() => label);
const slotState = computed<BreadcrumbRootSlotState>(() => ({ label: labelState.value }));

type BreadcrumbRootSetupExpose = Omit<BreadcrumbRootExpose, "element" | "label"> & {
  readonly element: typeof element;
  readonly label: typeof labelState;
};

const exposed = {
  element,
  label: labelState,
} satisfies BreadcrumbRootSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Landmark spacing, separators, and responsive collapse remain consumer-owned. */
</style>
