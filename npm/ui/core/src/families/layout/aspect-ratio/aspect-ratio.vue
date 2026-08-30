<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { isValidAspectRatio, normalizeAspectRatio } from "./aspect-ratio-runtime.ts";
import type {
  AspectRatioElement,
  AspectRatioExpose,
  AspectRatioSlotState,
  AspectRatioStyle,
} from "./aspect-ratio-types.ts";
import type { PrimitiveAs } from "../../../primitive.ts";

const { as = "div", ratio = 1 } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Positive finite width divided by height. Invalid values fall back to `1`.
   *
   * @default 1
   */
  readonly ratio?: number;
}>();

defineSlots<{
  /** Renders the box contents with the normalized ratio state. */
  default(props: AspectRatioSlotState): unknown;
}>();

const element = useTemplateRef<AspectRatioElement>("element");
const normalizedRatio = computed(() => normalizeAspectRatio(ratio));
const invalid = computed(() => !isValidAspectRatio(ratio));
const ratioText = computed(() => String(normalizedRatio.value));
const dataState = computed(() => (invalid.value ? "fallback" : "valid"));
const aspectRatioStyle = computed<AspectRatioStyle>(() => ({
  "--vize-ui-aspect-ratio": ratioText.value,
  aspectRatio: "var(--vize-ui-aspect-ratio)",
}));
const intrinsicProps = computed(() => ({ style: aspectRatioStyle.value }));

type AspectRatioSetupExpose = Omit<AspectRatioExpose, "element" | "invalid" | "ratio"> & {
  readonly element: typeof element;
  readonly invalid: typeof invalid;
  readonly ratio: typeof normalizedRatio;
};

const exposed = {
  element,
  invalid,
  ratio: normalizedRatio,
} satisfies AspectRatioSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="aspect-ratio"
    :data-state="dataState"
    :data-vize-aspect-ratio="ratioText"
    v-bind="intrinsicProps"
  >
    <slot :ratio="normalizedRatio" :invalid="invalid" />
  </component>
</template>

<style scoped>
/* Headless by design. Intrinsic aspect ratio is the only authored style. */
</style>
