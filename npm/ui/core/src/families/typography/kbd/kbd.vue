<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="kbd"
    :data-size="sizeState"
    :data-variant="variantState"
    :data-tone="toneState"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  KbdElement,
  KbdExpose,
  KbdSize,
  KbdSlotState,
  KbdTone,
  KbdVariant,
} from "./kbd-types.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";

const {
  as = "kbd",
  size = "md",
  variant = "key",
  tone = "neutral",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "kbd"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer visual-size token mirrored to `data-size`; no CSS is emitted.
   *
   * @default "md"
   */
  readonly size?: KbdSize;

  /**
   * Keyboard presentation token mirrored to `data-variant`; no CSS is emitted.
   *
   * @default "key"
   */
  readonly variant?: KbdVariant;

  /**
   * Consumer tone token mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: KbdTone;
}>();

defineSlots<{
  /** Renders keyboard input content with current presentation hook state. */
  default(props: KbdSlotState): unknown;
}>();

const element = useTemplateRef<KbdElement>("element");
const sizeState = computed(() => size);
const variantState = computed(() => variant);
const toneState = computed(() => tone);
const slotState = computed<KbdSlotState>(() => ({
  size: sizeState.value,
  tone: toneState.value,
  variant: variantState.value,
}));

type KbdSetupExpose = Omit<KbdExpose, "element" | "size" | "tone" | "variant"> & {
  readonly element: typeof element;
  readonly size: typeof sizeState;
  readonly tone: typeof toneState;
  readonly variant: typeof variantState;
};

const exposed = {
  element,
  size: sizeState,
  tone: toneState,
  variant: variantState,
} satisfies KbdSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Keycap styling, spacing, and separators remain consumer-owned. */
</style>
