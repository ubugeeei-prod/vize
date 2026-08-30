<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { TableRowExpose } from "./table-contracts.ts";
import type { TableRowProps, TableRowSlotState, TableRowState } from "./table-types.ts";

const { state = "default" } = defineProps<TableRowProps>();

defineSlots<{
  /** Renders native header and data cells with row state. */
  default(props: TableRowSlotState): unknown;
}>();

const element = useTemplateRef<HTMLTableRowElement>("element");
const stateState = computed<TableRowState>(() => state);
const selected = computed(() => stateState.value === "selected");
const slotState = computed<TableRowSlotState>(() => ({
  selected: selected.value,
  state: stateState.value,
}));

type TableRowSetupExpose = Omit<TableRowExpose, "element" | "selected" | "state"> & {
  readonly element: typeof element;
  readonly selected: typeof selected;
  readonly state: typeof stateState;
};

const exposed = {
  element,
  selected,
  state: stateState,
} satisfies TableRowSetupExpose;

defineExpose(exposed);
</script>

<template>
  <tr
    ref="element"
    data-vize-ui="table-row"
    part="row"
    :data-state="stateState"
    :data-selected="selected ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </tr>
</template>

<style scoped>
/* Headless by design. Hover, selected, and density presentation remain consumer-owned. */
</style>
