<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { fieldContext } from "./field-context.ts";
import type { FieldLabelExpose, FieldLabelSlotState } from "./field-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const { as = "label" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "label"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Renders visible label content with the resolved label and control ids. */
  default(props: FieldLabelSlotState): unknown;
}>();

const context = fieldContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const slotState = computed<FieldLabelSlotState>(() => ({
  for: context.labelProps.value.for,
  id: context.labelProps.value.id,
  invalid: context.invalid.value,
  name: context.name.value,
}));

type FieldLabelSetupExpose = Omit<FieldLabelExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
} satisfies FieldLabelSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    v-bind="context.labelProps.value"
    data-vize-ui="field-label"
    part="label"
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
