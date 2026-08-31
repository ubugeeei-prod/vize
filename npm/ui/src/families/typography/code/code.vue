<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="code"
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
  CodeElement,
  CodeExpose,
  CodeSize,
  CodeSlotState,
  CodeTone,
  CodeVariant,
} from "./code-types.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";

const {
  as = "code",
  size = "md",
  variant = "inline",
  tone = "neutral",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "code"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer code-size token mirrored to `data-size`; no CSS is emitted.
   *
   * @default "md"
   */
  readonly size?: CodeSize;

  /**
   * Code presentation token mirrored to `data-variant`; no CSS is emitted.
   *
   * @default "inline"
   */
  readonly variant?: CodeVariant;

  /**
   * Consumer tone token mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: CodeTone;
}>();

defineSlots<{
  /** Renders code content with current typography hook state. */
  default(props: CodeSlotState): unknown;
}>();

const element = useTemplateRef<CodeElement>("element");
const sizeState = computed(() => size);
const variantState = computed(() => variant);
const toneState = computed(() => tone);
const slotState = computed<CodeSlotState>(() => ({
  size: sizeState.value,
  tone: toneState.value,
  variant: variantState.value,
}));

type CodeSetupExpose = Omit<CodeExpose, "element" | "size" | "tone" | "variant"> & {
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
} satisfies CodeSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Code font, wrapping, syntax color, and block layout remain consumer-owned. */
</style>
