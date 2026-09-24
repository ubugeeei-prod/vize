<script setup lang="ts">
import { fieldsetContext } from "./fieldset-context.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";

const { as = "p" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "p"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Group-level help text; set `hasDescription` on the Fieldset so it is referenced. */
  default(props: Record<string, never>): unknown;
}>();

const context = fieldsetContext.use();
</script>

<template>
  <component
    :is="as"
    :id="context.descriptionId.value"
    part="description"
    data-vize-ui="fieldset-description"
  >
    <slot />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
