<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import type {
  EmptyStateDensity,
  EmptyStateElement,
  EmptyStateExpose,
  EmptyStateOrientation,
  EmptyStateSlotState,
  EmptyStateState,
  EmptyStateTone,
} from "./empty-state-types.ts";

const {
  as = "section",
  tone = "neutral",
  density = "comfortable",
  orientation = "block",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "section"
   */
  readonly as?: PrimitiveAs;

  /**
   * Styling tone mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: EmptyStateTone;

  /**
   * Spacing density mirrored to `data-density`; no CSS is emitted.
   *
   * @default "comfortable"
   */
  readonly density?: EmptyStateDensity;

  /**
   * Layout orientation mirrored to `data-orientation`; no CSS is emitted.
   *
   * @default "block"
   */
  readonly orientation?: EmptyStateOrientation;
}>();

defineSlots<{
  /** Renders empty-state content with the current styling and state hooks. */
  default(props: EmptyStateSlotState): unknown;
}>();

const element = useTemplateRef<EmptyStateElement>("element");
const state = "empty" satisfies EmptyStateState;

type EmptyStateSetupExpose = Omit<EmptyStateExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  get density() {
    return density;
  },
  get orientation() {
    return orientation;
  },
  state,
  get tone() {
    return tone;
  },
} satisfies EmptyStateSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="empty-state"
    data-state="empty"
    :data-tone="tone"
    :data-density="density"
    :data-orientation="orientation"
  >
    <slot :density="density" :orientation="orientation" :state="state" :tone="tone" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
