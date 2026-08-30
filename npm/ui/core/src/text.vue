<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="text"
    :data-size="sizeState"
    :data-weight="weightState"
    :data-tone="toneState"
    :data-truncate="truncateState ? 'true' : 'false'"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs } from "./primitive.ts";
import type {
  TextElement,
  TextExpose,
  TextSlotState,
  TextSize,
  TextTone,
  TextWeight,
} from "./text-types.ts";

const {
  as = "span",
  size = "md",
  weight = "regular",
  tone = "neutral",
  truncate = false,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "span"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer text-size token mirrored to `data-size`; no CSS is emitted.
   *
   * @default "md"
   */
  readonly size?: TextSize;

  /**
   * Consumer font-weight token mirrored to `data-weight`; no CSS is emitted.
   *
   * @default "regular"
   */
  readonly weight?: TextWeight;

  /**
   * Consumer tone token mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: TextTone;

  /**
   * Consumer truncation hook mirrored to `data-truncate`; no CSS is emitted.
   *
   * @default false
   */
  readonly truncate?: boolean;
}>();

defineSlots<{
  /** Renders text content with current typography hook state. */
  default(props: TextSlotState): unknown;
}>();

const element = useTemplateRef<TextElement>("element");
const sizeState = computed(() => size);
const weightState = computed(() => weight);
const toneState = computed(() => tone);
const truncateState = computed(() => truncate);
const slotState = computed<TextSlotState>(() => ({
  size: sizeState.value,
  tone: toneState.value,
  truncate: truncateState.value,
  weight: weightState.value,
}));

type TextSetupExpose = Omit<TextExpose, "element" | "size" | "tone" | "truncate" | "weight"> & {
  readonly element: typeof element;
  readonly size: typeof sizeState;
  readonly tone: typeof toneState;
  readonly truncate: typeof truncateState;
  readonly weight: typeof weightState;
};

const exposed = {
  element,
  size: sizeState,
  tone: toneState,
  truncate: truncateState,
  weight: weightState,
} satisfies TextSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Typography and truncation CSS remain consumer-owned. */
</style>
