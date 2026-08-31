<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { TableExpose } from "./table-contracts.ts";
import type {
  TableDensity,
  TableLayout,
  TableProps,
  TableSlotState,
  TableStyle,
} from "./table-types.ts";

const { layout = "auto", density = "normal" } = defineProps<TableProps>();

defineSlots<{
  /** Renders caption, section, row, and cell children with table hook state. */
  default(props: TableSlotState): unknown;
}>();

const element = useTemplateRef<HTMLTableElement>("element");
const layoutState = computed<TableLayout>(() => layout);
const densityState = computed<TableDensity>(() => density);
const tableStyle = computed<TableStyle>(() => ({
  "--vize-ui-table-layout": layoutState.value,
  tableLayout: layoutState.value,
}));
const intrinsicProps = computed(() => ({ style: tableStyle.value }));
const slotState = computed<TableSlotState>(() => ({
  density: densityState.value,
  layout: layoutState.value,
  style: tableStyle.value,
}));

type TableSetupExpose = Omit<TableExpose, "density" | "element" | "layout" | "style"> & {
  readonly density: typeof densityState;
  readonly element: typeof element;
  readonly layout: typeof layoutState;
  readonly style: typeof tableStyle;
};

const exposed = {
  density: densityState,
  element,
  layout: layoutState,
  style: tableStyle,
} satisfies TableSetupExpose;

defineExpose(exposed);
</script>

<template>
  <table
    ref="element"
    data-vize-ui="table"
    part="root"
    :data-layout="layoutState"
    :data-density="densityState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </table>
</template>

<style scoped>
/* Headless by design. Borders, spacing, responsive overflow, and typography remain consumer-owned. */
</style>
