<template>
  <component
    :is="as"
    ref="element"
    aria-hidden="true"
    data-vize-ui="breadcrumb-separator"
    part="separator"
    role="presentation"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  BreadcrumbSeparatorExpose,
  BreadcrumbSeparatorSlotState,
} from "./breadcrumb-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

const { as = "span" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "span"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Renders decorative separator content hidden from assistive technology. */
  default(props: BreadcrumbSeparatorSlotState): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const decorativeState = computed(() => true as const);
const slotState = computed<BreadcrumbSeparatorSlotState>(() => ({
  decorative: decorativeState.value,
}));

type BreadcrumbSeparatorSetupExpose = Omit<BreadcrumbSeparatorExpose, "decorative" | "element"> & {
  readonly decorative: typeof decorativeState;
  readonly element: typeof element;
};

const exposed = {
  decorative: decorativeState,
  element,
} satisfies BreadcrumbSeparatorSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Separator glyphs, spacing, and responsive hiding remain consumer-owned. */
</style>
