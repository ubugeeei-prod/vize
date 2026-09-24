<script setup lang="ts">
import { computed } from "vue";

import { selectContext } from "./select-context.ts";
import type { SelectValueSlotState } from "./select-types.ts";

const { separator = ", " } = defineProps<{
  /**
   * Text placed between labels when several values are selected.
   *
   * @default ", "
   */
  readonly separator?: string;
}>();

defineSlots<{
  /** Custom selected-value rendering. Receives the selected values, their text, and the placeholder. */
  default?(props: SelectValueSlotState<unknown>): unknown;

  /** Custom placeholder rendering shown while nothing is selected. */
  placeholder?(props: SelectValueSlotState<unknown>): unknown;
}>();

const context = selectContext.use();
const empty = computed(() => context.selected.value.length === 0);
const text = computed(() => context.selectedText.value.join(separator));
const slotState = computed<SelectValueSlotState<unknown>>(() => ({
  empty: empty.value,
  placeholder: context.placeholder.value,
  selected: context.selected.value,
  selectedText: context.selectedText.value,
}));
</script>

<template>
  <span
    :data-vize-ui="`${context.partPrefix}-value`"
    part="value"
    :data-placeholder="empty ? 'true' : undefined"
    :data-count="context.selected.value.length"
  >
    <slot v-if="empty" name="placeholder" v-bind="slotState">{{ context.placeholder.value }}</slot>
    <slot v-else v-bind="slotState">{{ text }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
