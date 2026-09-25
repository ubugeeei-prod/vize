<script setup lang="ts">
import { computed } from "vue";

import { cascaderContext } from "./cascader-context.ts";
import type { CascaderValueSlotState } from "./cascader-types.ts";

const { pathSeparator = ", " } = defineProps<{
  /**
   * Text placed between paths when several are selected.
   *
   * @default ", "
   */
  readonly pathSeparator?: string;
}>();

defineSlots<{
  /** Custom rendering of the selected paths. */
  default?(props: CascaderValueSlotState): unknown;

  /** Custom placeholder rendering while nothing is selected. */
  placeholder?(props: CascaderValueSlotState): unknown;
}>();

const context = cascaderContext.use();
const empty = computed(() => context.selectedText.value.length === 0);
const slotState = computed<CascaderValueSlotState>(() => ({
  empty: empty.value,
  placeholder: context.placeholder.value,
  selectedText: context.selectedText.value,
}));
</script>

<template>
  <span data-vize-ui="cascader-value" part="value" :data-placeholder="empty ? 'true' : undefined">
    <slot v-if="empty" name="placeholder" v-bind="slotState">{{ context.placeholder.value }}</slot>
    <slot v-else v-bind="slotState">{{ context.selectedText.value.join(pathSeparator) }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
