<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { TableHeaderExpose } from "./table-contracts.ts";
import type {
  TableCellAlign,
  TableCellStyle,
  TableHeaderProps,
  TableHeaderScope,
  TableHeaderSlotState,
} from "./table-types.ts";

const {
  scope = "col",
  abbr = undefined,
  colspan = undefined,
  rowspan = undefined,
  align = "start",
} = defineProps<TableHeaderProps>();

defineSlots<{
  /** Renders native header cell content with scope and alignment state. */
  default(props: TableHeaderSlotState): unknown;
}>();

const element = useTemplateRef<HTMLTableCellElement>("element");
const scopeState = computed<TableHeaderScope>(() => scope);
const abbrState = computed<string | undefined>(() => abbr);
const colspanState = computed<number | undefined>(() => colspan);
const rowspanState = computed<number | undefined>(() => rowspan);
const alignState = computed<TableCellAlign>(() => align);
const cellStyle = computed<TableCellStyle>(() => ({
  "--vize-ui-table-cell-align": alignState.value,
  textAlign: alignState.value,
}));
const intrinsicProps = computed(() => ({ style: cellStyle.value }));
const slotState = computed<TableHeaderSlotState>(() => ({
  abbr: abbrState.value,
  align: alignState.value,
  colspan: colspanState.value,
  rowspan: rowspanState.value,
  scope: scopeState.value,
  style: cellStyle.value,
}));

type TableHeaderSetupExpose = Omit<
  TableHeaderExpose,
  "abbr" | "align" | "colspan" | "element" | "rowspan" | "scope" | "style"
> & {
  readonly abbr: typeof abbrState;
  readonly align: typeof alignState;
  readonly colspan: typeof colspanState;
  readonly element: typeof element;
  readonly rowspan: typeof rowspanState;
  readonly scope: typeof scopeState;
  readonly style: typeof cellStyle;
};

const exposed = {
  abbr: abbrState,
  align: alignState,
  colspan: colspanState,
  element,
  rowspan: rowspanState,
  scope: scopeState,
  style: cellStyle,
} satisfies TableHeaderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <th
    ref="element"
    :scope="scopeState"
    :abbr="abbrState"
    :colspan="colspanState"
    :rowspan="rowspanState"
    data-vize-ui="table-header"
    part="header"
    :data-scope="scopeState"
    :data-align="alignState"
    :data-colspan="colspanState"
    :data-rowspan="rowspanState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </th>
</template>

<style scoped>
/* Headless by design. Header weight, sorting glyphs, and sticky offsets remain consumer-owned. */
</style>
