<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { fieldContext } from "./field-context.ts";
import type { FieldDescriptionExpose, FieldDescriptionSlotState } from "./field-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "./primitive.ts";

const { as = "p" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "p"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Renders descriptive help text with the resolved description id. */
  default(props: FieldDescriptionSlotState): unknown;
}>();

const context = fieldContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const slotState = computed<FieldDescriptionSlotState>(() => ({
  id: context.descriptionProps.value.id,
  invalid: context.invalid.value,
  name: context.name.value,
}));

type FieldDescriptionSetupExpose = Omit<FieldDescriptionExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
} satisfies FieldDescriptionSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    v-bind="context.descriptionProps.value"
    data-vize-ui="field-description"
    part="description"
    :data-state="context.invalid.value ? 'invalid' : 'valid'"
    :data-invalid="context.invalid.value ? 'true' : 'false'"
    :data-name="context.name.value"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
