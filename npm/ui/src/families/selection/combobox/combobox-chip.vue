<script setup lang="ts" generic="T">
import { computed } from "vue";

import { comboboxChipContext, comboboxContext } from "./combobox-context.ts";
import type { ComboboxChipSlotState } from "./combobox-types.ts";

const { value } = defineProps<{
  /** Selected value rendered by this chip. @default required */
  readonly value: T;
}>();

defineSlots<{
  /** Chip content, usually the text plus a `ComboboxChipRemove`. */
  default?(props: ComboboxChipSlotState<T>): unknown;
}>();

const context = comboboxContext.use();
const text = computed(() => context.textOf(value));
const disabled = computed(
  () => context.disabled.value || context.readonly.value || context.isValueDisabled(value),
);

function remove(): boolean {
  if (disabled.value) return false;
  const removed = context.remove(value);
  context.inputElement.value?.focus();
  return removed;
}

const slotState = computed<ComboboxChipSlotState<T>>(() => ({
  disabled: disabled.value,
  remove,
  text: text.value,
  value,
}));

comboboxChipContext.provide({ disabled, remove, text });
</script>

<template>
  <span data-vize-ui="combobox-chip" part="chip" :data-disabled="disabled ? 'true' : undefined">
    <slot v-bind="slotState">{{ text }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
