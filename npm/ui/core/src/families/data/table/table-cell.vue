<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { TableCellExpose } from "./table-contracts.ts";
import type {
  TableCellAlign,
  TableCellProps,
  TableCellSlotState,
  TableCellStyle,
} from "./table-types.ts";

const {
  headers = undefined,
  colspan = undefined,
  rowspan = undefined,
  align = "start",
} = defineProps<TableCellProps>();

defineSlots<{
  /** Renders native data cell content with headers and alignment state. */
  default(props: TableCellSlotState): unknown;
}>();

const element = useTemplateRef<HTMLTableCellElement>("element");
const headersState = computed<string | undefined>(() => headers);
const colspanState = computed<number | undefined>(() => colspan);
const rowspanState = computed<number | undefined>(() => rowspan);
const alignState = computed<TableCellAlign>(() => align);
const cellStyle = computed<TableCellStyle>(() => ({
  "--vize-ui-table-cell-align": alignState.value,
  textAlign: alignState.value,
}));
const intrinsicProps = computed(() => ({ style: cellStyle.value }));
const slotState = computed<TableCellSlotState>(() => ({
  align: alignState.value,
  colspan: colspanState.value,
  headers: headersState.value,
  rowspan: rowspanState.value,
  style: cellStyle.value,
}));

type TableCellSetupExpose = Omit<
  TableCellExpose,
  "align" | "colspan" | "element" | "headers" | "rowspan" | "style"
> & {
  readonly align: typeof alignState;
  readonly colspan: typeof colspanState;
  readonly element: typeof element;
  readonly headers: typeof headersState;
  readonly rowspan: typeof rowspanState;
  readonly style: typeof cellStyle;
};

const exposed = {
  align: alignState,
  colspan: colspanState,
  element,
  headers: headersState,
  rowspan: rowspanState,
  style: cellStyle,
} satisfies TableCellSetupExpose;

defineExpose(exposed);
</script>

<template>
  <td
    ref="element"
    :headers="headersState"
    :colspan="colspanState"
    :rowspan="rowspanState"
    data-vize-ui="table-cell"
    part="cell"
    :data-align="alignState"
    :data-colspan="colspanState"
    :data-rowspan="rowspanState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </td>
</template>

<style scoped>
/* Headless by design. Alignment, numeric formatting, wrapping, and truncation remain consumer-owned. */
</style>
