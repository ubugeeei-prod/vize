<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs } from "./primitive.ts";
import type { SeparatorElement, SeparatorExpose, SeparatorOrientation } from "./separator-types.ts";

const {
  as = "hr",
  orientation = "horizontal",
  decorative = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "hr"
   */
  readonly as?: PrimitiveAs;

  /**
   * Logical separator axis announced to assistive technology.
   *
   * @default "horizontal"
   */
  readonly orientation?: SeparatorOrientation;

  /**
   * Hide the separator from assistive technology when it is purely visual.
   *
   * @default false
   */
  readonly decorative?: boolean;

  /**
   * Accessible name for a semantic separator when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label a semantic separator.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const element = useTemplateRef<SeparatorElement>("element");
const decorativeState = computed(() => decorative);
const orientationState = computed(() => orientation);

type SeparatorSetupExpose = Omit<SeparatorExpose, "decorative" | "element" | "orientation"> & {
  readonly decorative: typeof decorativeState;
  readonly element: typeof element;
  readonly orientation: typeof orientationState;
};

const exposed = {
  decorative: decorativeState,
  element,
  orientation: orientationState,
} satisfies SeparatorSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :role="decorative ? 'presentation' : 'separator'"
    :aria-hidden="decorative ? 'true' : undefined"
    :aria-orientation="decorative ? undefined : orientation"
    :aria-label="decorative ? undefined : ariaLabel"
    :aria-labelledby="decorative ? undefined : ariaLabelledby"
    data-vize-ui="separator"
    :data-state="decorative ? 'decorative' : 'semantic'"
    :data-orientation="orientation"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
