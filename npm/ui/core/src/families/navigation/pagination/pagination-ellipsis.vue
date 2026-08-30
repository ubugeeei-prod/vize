<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { paginationContext } from "./pagination-context.ts";
import type {
  PaginationEllipsisExpose,
  PaginationEllipsisPosition,
  PaginationEllipsisSlotState,
} from "./pagination-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

const {
  as = "span",
  position = "end",
  label = "More pages",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "span"
   */
  readonly as?: PrimitiveAs;

  /**
   * Position relative to the current compact range.
   *
   * @default "end"
   */
  readonly position?: PaginationEllipsisPosition;

  /**
   * Accessible label for the non-interactive gap.
   *
   * @default "More pages"
   */
  readonly label?: string;
}>();

defineSlots<{
  /** Ellipsis contents. Receives position and the literal disabled state. */
  default(props: PaginationEllipsisSlotState): unknown;
}>();

paginationContext.use();

const element = useTemplateRef<PrimitiveElement>("element");
const disabled = true as const;
const slotState = computed<PaginationEllipsisSlotState>(() => ({
  disabled,
  position,
}));

type PaginationEllipsisSetupExpose = Omit<
  PaginationEllipsisExpose,
  keyof PaginationEllipsisSlotState | "element"
> & {
  readonly disabled: true;
  readonly element: typeof element;
  readonly position: ComputedRef<PaginationEllipsisPosition>;
};

const exposed = {
  disabled,
  element,
  position: computed(() => position),
} satisfies PaginationEllipsisSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :aria-label="label"
    data-vize-ui="pagination-ellipsis"
    part="ellipsis"
    :data-position="position"
    data-disabled="true"
  >
    <slot v-bind="slotState">...</slot>
  </component>
</template>

<style scoped>
/* Headless by design. Ellipsis visibility, spacing, and hidden-label treatment remain consumer-owned. */
</style>
