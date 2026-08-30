<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="list"
    :data-marker="markerState"
    :data-spacing="spacingState"
    :data-tone="toneState"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  ListElement,
  ListExpose,
  ListMarker,
  ListSlotState,
  ListSpacing,
  ListTone,
} from "./list-types.ts";
import type { PrimitiveAs } from "./primitive.ts";

const {
  as = "ul",
  marker = "disc",
  spacing = "normal",
  tone = "neutral",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "ul"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer marker token mirrored to `data-marker`; no CSS is emitted.
   *
   * @default "disc"
   */
  readonly marker?: ListMarker;

  /**
   * Consumer spacing token mirrored to `data-spacing`; no CSS is emitted.
   *
   * @default "normal"
   */
  readonly spacing?: ListSpacing;

  /**
   * Consumer tone token mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: ListTone;
}>();

defineSlots<{
  /** Renders list items with current presentation hook state. */
  default(props: ListSlotState): unknown;
}>();

const element = useTemplateRef<ListElement>("element");
const markerState = computed(() => marker);
const spacingState = computed(() => spacing);
const toneState = computed(() => tone);
const slotState = computed<ListSlotState>(() => ({
  marker: markerState.value,
  spacing: spacingState.value,
  tone: toneState.value,
}));

type ListSetupExpose = Omit<ListExpose, "element" | "marker" | "spacing" | "tone"> & {
  readonly element: typeof element;
  readonly marker: typeof markerState;
  readonly spacing: typeof spacingState;
  readonly tone: typeof toneState;
};

const exposed = {
  element,
  marker: markerState,
  spacing: spacingState,
  tone: toneState,
} satisfies ListSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Marker style, item spacing, nesting rhythm, and color remain consumer-owned. */
</style>
