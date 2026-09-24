<script setup lang="ts">
import { computed } from "vue";

import { selectContext, selectItemContext } from "./select-context.ts";

const { forceMount = false } = defineProps<{
  /**
   * Render while the option is unselected too, publishing `data-state` for CSS transitions.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Indicator content such as a check mark. Receives the option selection state. */
  default(props: { readonly selected: boolean }): unknown;
}>();

const context = selectContext.use();
const item = selectItemContext.use();
const visible = computed(() => forceMount || item.selected.value);
</script>

<template>
  <span
    :hidden="visible ? undefined : true"
    aria-hidden="true"
    :data-vize-ui="`${context.partPrefix}-item-indicator`"
    part="item-indicator"
    :data-state="item.selected.value ? 'checked' : 'unchecked'"
  >
    <slot v-if="visible" :selected="item.selected.value" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
