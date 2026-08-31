<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  CardDensity,
  CardElement,
  CardExpose,
  CardSlotState,
  CardTone,
  CardVariant,
} from "./card-types.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";

const {
  as = "section",
  variant = "card",
  density = "comfortable",
  tone = "neutral",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "section"
   */
  readonly as?: PrimitiveAs;

  /**
   * Surface usage variant mirrored to `data-variant` for consumer-owned styling.
   *
   * @default "card"
   */
  readonly variant?: CardVariant;

  /**
   * Density token mirrored to `data-density`; no spacing CSS is emitted.
   *
   * @default "comfortable"
   */
  readonly density?: CardDensity;

  /**
   * Styling tone mirrored to `data-tone`; no color CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: CardTone;
}>();

defineSlots<{
  /** Renders card content with the current variant, density, and tone. */
  default(props: CardSlotState): unknown;
}>();

const element = useTemplateRef<CardElement>("element");
const variantState = computed(() => variant);
const densityState = computed(() => density);
const toneState = computed(() => tone);
const slotState = computed<CardSlotState>(() => ({
  density: densityState.value,
  tone: toneState.value,
  variant: variantState.value,
}));

type CardSetupExpose = Omit<CardExpose, "density" | "element" | "tone" | "variant"> & {
  readonly density: typeof densityState;
  readonly element: typeof element;
  readonly tone: typeof toneState;
  readonly variant: typeof variantState;
};

const exposed = {
  density: densityState,
  element,
  tone: toneState,
  variant: variantState,
} satisfies CardSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="card"
    :data-variant="variantState"
    :data-density="densityState"
    :data-tone="toneState"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
