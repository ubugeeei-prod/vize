<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { fieldContext } from "./field-context.ts";
import type { FieldErrorMessageExpose, FieldErrorMessageSlotState } from "./field-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const { as = "p", forceMount = false } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "p"
   */
  readonly as?: PrimitiveAs;

  /**
   * Keep the error message element in the DOM while the field is valid.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Renders validation text with the resolved error id and current field errors. */
  default(props: FieldErrorMessageSlotState): unknown;
}>();

const context = fieldContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const rendered = computed(() => context.invalid.value || forceMount);
const slotState = computed<FieldErrorMessageSlotState>(() => ({
  errors: context.errors.value,
  id: context.errorMessageProps.value.id,
  invalid: context.invalid.value,
  message: context.errorMessage.value,
  name: context.name.value,
}));

type FieldErrorMessageSetupExpose = Omit<
  FieldErrorMessageExpose,
  "element" | "invalid" | "message"
> & {
  readonly element: typeof element;
  readonly invalid: ComputedRef<boolean>;
  readonly message: ComputedRef<string | undefined>;
};

const exposed = {
  element,
  invalid: context.invalid,
  message: context.errorMessage,
} satisfies FieldErrorMessageSetupExpose;

defineExpose(exposed);
</script>

<template>
  <!-- eslint-disable vue/no-root-v-if -->
  <component
    v-if="rendered"
    :is="as"
    ref="element"
    v-bind="context.errorMessageProps.value"
    data-vize-ui="field-error-message"
    part="error-message"
    :data-state="context.invalid.value ? 'invalid' : 'valid'"
    :data-invalid="context.invalid.value ? 'true' : 'false'"
    :data-name="context.name.value"
  >
    <slot v-bind="slotState">{{ context.errorMessage.value }}</slot>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
