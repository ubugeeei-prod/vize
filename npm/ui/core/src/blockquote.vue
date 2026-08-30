<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="blockquote"
    :cite="citeState"
    :data-size="sizeState"
    :data-tone="toneState"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  BlockquoteElement,
  BlockquoteExpose,
  BlockquoteSize,
  BlockquoteSlotState,
  BlockquoteTone,
} from "./blockquote-types.ts";
import type { PrimitiveAs } from "./primitive.ts";

const {
  as = "blockquote",
  size = "md",
  tone = "neutral",
  cite,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "blockquote"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer quotation-size token mirrored to `data-size`; no CSS is emitted.
   *
   * @default "md"
   */
  readonly size?: BlockquoteSize;

  /**
   * Consumer tone token mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: BlockquoteTone;

  /**
   * Native citation URL mirrored to the rendered root `cite` attribute.
   *
   * @default undefined
   */
  readonly cite?: string;
}>();

defineSlots<{
  /** Renders quoted content with current typography hook state. */
  default(props: BlockquoteSlotState): unknown;
}>();

const element = useTemplateRef<BlockquoteElement>("element");
const sizeState = computed(() => size);
const toneState = computed(() => tone);
const citeState = computed(() => cite);
const slotState = computed<BlockquoteSlotState>(() => ({
  cite: citeState.value,
  size: sizeState.value,
  tone: toneState.value,
}));

type BlockquoteSetupExpose = Omit<BlockquoteExpose, "cite" | "element" | "size" | "tone"> & {
  readonly cite: typeof citeState;
  readonly element: typeof element;
  readonly size: typeof sizeState;
  readonly tone: typeof toneState;
};

const exposed = {
  cite: citeState,
  element,
  size: sizeState,
  tone: toneState,
} satisfies BlockquoteSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Quote marks, borders, spacing, typography, and citation layout remain consumer-owned. */
</style>
