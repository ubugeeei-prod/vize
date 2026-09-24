<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { dataGridContext } from "./data-grid-context.ts";
import type { DataGridAlign, DataGridPinSide } from "./data-grid-types.ts";

const {
  rowId,
  columnId,
  colIndex,
  width = 150,
  pin = null,
  pinOffset = 0,
  align = "start",
  editable = false,
} = defineProps<{
  /**
   * Row id this cell belongs to.
   *
   * @default undefined
   */
  readonly rowId: string;

  /**
   * Column id this cell belongs to.
   *
   * @default undefined
   */
  readonly columnId: string;

  /**
   * One-based `aria-colindex`.
   *
   * @default undefined
   */
  readonly colIndex: number;

  /**
   * Column width in CSS pixels, published as `--vize-ui-data-grid-column-width`.
   *
   * @default 150
   */
  readonly width?: number;

  /**
   * Pin side, published as `data-pin`.
   *
   * @default null
   */
  readonly pin?: DataGridPinSide | null;

  /**
   * Sticky inset from the pinned edge, published as `--vize-ui-data-grid-pin-offset`.
   *
   * @default 0
   */
  readonly pinOffset?: number;

  /**
   * Logical alignment hook.
   *
   * @default "start"
   */
  readonly align?: DataGridAlign;

  /**
   * Whether the cell can be edited (Enter/F2 or double click).
   *
   * @default false
   */
  readonly editable?: boolean;
}>();

defineSlots<{
  /** Cell contents rendered by the grid root. */
  default(): unknown;
}>();

const grid = dataGridContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const active = computed(
  () => grid.activeCell.value?.rowId === rowId && grid.activeCell.value.columnId === columnId,
);
const editing = computed(
  () => grid.editing.value?.rowId === rowId && grid.editing.value.columnId === columnId,
);

function onClick(): void {
  grid.focusCell({ rowId, columnId });
}

function onDblclick(): void {
  if (editable) grid.startEdit({ rowId, columnId });
}

const cellProps = computed(() => ({
  role: "gridcell",
  tabindex: active.value ? 0 : -1,
  style: {
    "--vize-ui-data-grid-column-width": `${width}px`,
    "--vize-ui-data-grid-pin-offset": `${pinOffset}px`,
  },
  onClick,
  onDblclick,
}));

defineExpose({ element });
</script>

<template>
  <div
    v-bind="cellProps"
    ref="element"
    :aria-colindex="colIndex"
    :aria-readonly="editable ? undefined : 'true'"
    data-vize-ui="data-grid-cell"
    part="cell"
    :data-row-id="rowId"
    :data-column-id="columnId"
    :data-align="align"
    :data-pin="pin ?? undefined"
    :data-active="active ? 'true' : undefined"
    :data-editing="editing ? 'true' : undefined"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
