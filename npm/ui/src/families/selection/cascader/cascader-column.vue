<script setup lang="ts">
import { computed } from "vue";

import { cascaderColumnContext, cascaderContext } from "./cascader-context.ts";
import type { CascaderColumnSlotState } from "./cascader-types.ts";

const { level, ariaLabel = undefined } = defineProps<{
  /** Zero-based depth of the options this column lists. @default required */
  readonly level: number;

  /**
   * Accessible name; defaults to the parent option's text (or "Options" at the top level).
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** `CascaderItem` options for this level; render them from the root `columns` state. */
  default?(props: CascaderColumnSlotState): unknown;
}>();

const context = cascaderContext.use();
const column = computed(() => context.columns.value[level]);
const loading = computed(() => column.value?.loading === true);
const label = computed(() => {
  if (ariaLabel !== undefined) return ariaLabel;
  const parent = column.value?.parent;
  return parent === null || parent === undefined ? "Options" : context.textOf(parent);
});
const listboxProps = computed(() => ({
  role: "listbox" as const,
  onMousedown: keepTriggerFocus,
  onPointerdown: keepTriggerFocus,
}));

// Options never take DOM focus: the trigger keeps it and exposes the
// highlighted option through aria-activedescendant.
function keepTriggerFocus(event: MouseEvent | PointerEvent): void {
  if (event.button === 0) event.preventDefault();
}

cascaderColumnContext.provide({ level: computed(() => level) });
</script>

<template>
  <div
    :id="context.columnId(level)"
    v-bind="listboxProps"
    :aria-label="label"
    :aria-busy="loading ? 'true' : undefined"
    :aria-multiselectable="context.multiple.value ? 'true' : undefined"
    data-vize-ui="cascader-column"
    part="column"
    :data-level="level"
    :data-loading="loading ? 'true' : undefined"
  >
    <slot :level="level" :loading="loading" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
