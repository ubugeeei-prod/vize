<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs } from "./primitive.ts";
import type {
  BadgeElement,
  BadgeExpose,
  BadgeSlotState,
  BadgeTone,
  BadgeVariant,
} from "./badge-types.ts";

const {
  as = "span",
  variant = "label",
  tone = "neutral",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "span"
   */
  readonly as?: PrimitiveAs;

  /**
   * Usage variant mirrored to `data-variant` for consumer-owned styling.
   *
   * @default "label"
   */
  readonly variant?: BadgeVariant;

  /**
   * Styling tone mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: BadgeTone;
}>();

defineSlots<{
  /** Renders badge content with the current variant and tone. */
  default(props: BadgeSlotState): unknown;
}>();

const element = useTemplateRef<BadgeElement>("element");
const variantState = computed(() => variant);
const toneState = computed(() => tone);
const slotState = computed<BadgeSlotState>(() => ({
  tone: toneState.value,
  variant: variantState.value,
}));

type BadgeSetupExpose = Omit<BadgeExpose, "element" | "tone" | "variant"> & {
  readonly element: typeof element;
  readonly tone: typeof toneState;
  readonly variant: typeof variantState;
};

const exposed = {
  element,
  tone: toneState,
  variant: variantState,
} satisfies BadgeSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="badge"
    :data-variant="variantState"
    :data-tone="toneState"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
