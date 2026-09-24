<script setup lang="ts">
import { computed } from "vue";

import { fieldsetContext } from "./fieldset-context.ts";
import type { FieldsetErrorMessageSlotState } from "./fieldset-types.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";

const { as = "p", forceMount = false } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "p"
   */
  readonly as?: PrimitiveAs;

  /**
   * Keep the element in the DOM while the group is valid.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Group validation text; defaults to the first matching error message. */
  default?(props: FieldsetErrorMessageSlotState): unknown;
}>();

const context = fieldsetContext.use();
const rendered = computed(() => context.invalid.value || forceMount);
const slotState = computed<FieldsetErrorMessageSlotState>(() => ({
  id: context.errorMessageId.value,
  message: context.errorMessage.value,
  errors: context.errors.value,
}));
</script>

<template>
  <!-- eslint-disable vue/no-root-v-if -->
  <component
    :is="as"
    v-if="rendered"
    :id="context.errorMessageId.value"
    part="error-message"
    data-vize-ui="fieldset-error-message"
    :data-invalid="context.invalid.value ? 'true' : 'false'"
  >
    <slot v-bind="slotState">{{ context.errorMessage.value }}</slot>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
