<template>
  <component
    :is="resolvedAs"
    ref="element"
    part="root"
    data-vize-ui="heading"
    :data-level="levelState"
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
  HeadingElement,
  HeadingExpose,
  HeadingLevel,
  HeadingSize,
  HeadingSlotState,
  HeadingTone,
  HeadingWeight,
} from "./heading-types.ts";

const {
  as,
  level = 2,
  size = "md",
  weight = "semibold",
  tone = "neutral",
  truncate = false,
} = defineProps<{
  /**
   * Native element, custom element, or component to render. When omitted,
   * Heading renders the native `h${level}` element.
   *
   * @default undefined
   */
  readonly as?: PrimitiveAs;

  /**
   * Semantic heading level used for the default native host and `data-level`.
   *
   * @default 2
   */
  readonly level?: HeadingLevel;

  /**
   * Consumer visual-size token mirrored to `data-size`; no CSS is emitted.
   *
   * @default "md"
   */
  readonly size?: HeadingSize;

  /**
   * Consumer font-weight token mirrored to `data-weight`; no CSS is emitted.
   *
   * @default "semibold"
   */
  readonly weight?: HeadingWeight;

  /**
   * Consumer tone token mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: HeadingTone;

  /**
   * Consumer truncation hook mirrored to `data-truncate`; no CSS is emitted.
   *
   * @default false
   */
  readonly truncate?: boolean;
}>();

defineSlots<{
  /** Renders heading content with current semantic and typography hook state. */
  default(props: HeadingSlotState): unknown;
}>();

const element = useTemplateRef<HeadingElement>("element");
const levelState = computed(() => level);
const sizeState = computed(() => size);
const weightState = computed(() => weight);
const toneState = computed(() => tone);
const truncateState = computed(() => truncate);
const resolvedAs = computed(() => as ?? `h${levelState.value}`);
const slotState = computed<HeadingSlotState>(() => ({
  level: levelState.value,
  size: sizeState.value,
  tone: toneState.value,
  truncate: truncateState.value,
  weight: weightState.value,
}));

type HeadingSetupExpose = Omit<
  HeadingExpose,
  "element" | "level" | "size" | "tone" | "truncate" | "weight"
> & {
  readonly element: typeof element;
  readonly level: typeof levelState;
  readonly size: typeof sizeState;
  readonly tone: typeof toneState;
  readonly truncate: typeof truncateState;
  readonly weight: typeof weightState;
};

const exposed = {
  element,
  level: levelState,
  size: sizeState,
  tone: toneState,
  truncate: truncateState,
  weight: weightState,
} satisfies HeadingSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Heading scale, rhythm, and truncation CSS remain consumer-owned. */
</style>
