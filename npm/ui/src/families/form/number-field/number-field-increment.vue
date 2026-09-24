<script setup lang="ts">
import { useNumberFieldTrigger } from "./number-field-trigger.ts";
import type { NumberFieldTriggerProps, NumberFieldTriggerSlotState } from "./number-field-types.ts";

const { ariaLabel = "Increase" } = defineProps<NumberFieldTriggerProps>();

defineSlots<{
  /** Trigger contents, typically an icon. */
  default?(props: NumberFieldTriggerSlotState): unknown;
}>();

const trigger = useNumberFieldTrigger("increment");
</script>

<template>
  <button
    type="button"
    tabindex="-1"
    :aria-label="ariaLabel"
    :aria-controls="trigger.context.inputId.value"
    :disabled="trigger.disabled.value"
    part="increment"
    data-vize-ui="number-field-increment"
    :data-disabled="trigger.disabled.value ? 'true' : undefined"
    :data-holding="trigger.holding.value ? 'true' : undefined"
    @pointerdown="trigger.onPointerdown"
    @pointerup="trigger.onPointerEnd"
    @pointercancel="trigger.onPointerEnd"
    @pointerleave="trigger.onPointerEnd"
    @click="trigger.onClick"
  >
    <slot v-bind="trigger.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
