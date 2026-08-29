<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs } from "./primitive.ts";
import { resolveSpacerLayout } from "./spacer-runtime.ts";
import type {
  SpacerAxis,
  SpacerDisplay,
  SpacerElement,
  SpacerExpose,
  SpacerSize,
  SpacerStyle,
} from "./spacer-types.ts";

const {
  as = "span",
  axis = "block",
  size = "1rem",
  inlineSize = undefined,
  blockSize = undefined,
  display = undefined,
} = defineProps<{
  /**
   * Native element, custom element, SVG element, or component to render.
   *
   * @default "span"
   */
  readonly as?: PrimitiveAs;

  /**
   * Logical axis that receives `size` when explicit axis sizes are omitted.
   *
   * @default "block"
   */
  readonly axis?: SpacerAxis;

  /**
   * Native CSS size applied to the selected axis.
   *
   * @default "1rem"
   */
  readonly size?: SpacerSize;

  /**
   * Native CSS logical inline size. Overrides `size` for the inline axis.
   *
   * @default undefined
   */
  readonly inlineSize?: SpacerSize;

  /**
   * Native CSS logical block size. Overrides `size` for the block axis.
   *
   * @default undefined
   */
  readonly blockSize?: SpacerSize;

  /**
   * CSS display mode applied to the spacer host.
   *
   * @default "block" for block axis, otherwise "inline-block"
   */
  readonly display?: SpacerDisplay;
}>();

const element = useTemplateRef<SpacerElement>("element");
const layout = computed(() => resolveSpacerLayout({ axis, blockSize, display, inlineSize, size }));
const axisState = computed(() => layout.value.axis);
const blockSizeState = computed(() => layout.value.blockSize);
const displayState = computed(() => layout.value.display);
const inlineSizeState = computed(() => layout.value.inlineSize);
const spacerState = computed(() => layout.value.state);
const spacerStyle = computed<SpacerStyle>(() => layout.value.style);
const intrinsicProps = computed(() => ({ style: spacerStyle.value }));

type SpacerSetupExpose = Omit<
  SpacerExpose,
  "axis" | "blockSize" | "display" | "element" | "inlineSize"
> & {
  readonly axis: typeof axisState;
  readonly blockSize: typeof blockSizeState;
  readonly display: typeof displayState;
  readonly element: typeof element;
  readonly inlineSize: typeof inlineSizeState;
};

const exposed = {
  axis: axisState,
  blockSize: blockSizeState,
  display: displayState,
  element,
  inlineSize: inlineSizeState,
} satisfies SpacerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    aria-hidden="true"
    part="root"
    data-vize-ui="spacer"
    :data-state="spacerState"
    :data-axis="axisState"
    :data-display="displayState"
    :data-vize-spacer-inline-size="inlineSizeState"
    :data-vize-spacer-block-size="blockSizeState"
    v-bind="intrinsicProps"
  />
</template>

<style scoped>
/* Headless by design. Native logical sizing is authored as intrinsic inline style. */
</style>
