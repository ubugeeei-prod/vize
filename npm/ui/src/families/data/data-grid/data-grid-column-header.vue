<script setup lang="ts">
import { computed } from "vue";

import { dataGridContext } from "./data-grid-context.ts";
import DataGridResizeHandle from "./data-grid-resize-handle.vue";
import type { DataGridAlign, DataGridPinSide, DataGridSortDirection } from "./data-grid-types.ts";

const {
  columnId,
  colIndex,
  label = undefined,
  width = 150,
  minWidth = 40,
  maxWidth = Number.POSITIVE_INFINITY,
  pin = null,
  pinOffset = 0,
  align = "start",
  sortable = false,
  sortDirection = null,
  sortIndex = null,
  resizable = false,
} = defineProps<{
  /**
   * Column id.
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
   * Header text, also the resize handle's accessible name.
   *
   * @default undefined
   */
  readonly label?: string;

  /**
   * Column width in CSS pixels.
   *
   * @default 150
   */
  readonly width?: number;

  /**
   * Minimum width announced by the resize handle.
   *
   * @default 40
   */
  readonly minWidth?: number;

  /**
   * Maximum width announced by the resize handle.
   *
   * @default Infinity
   */
  readonly maxWidth?: number;

  /**
   * Pin side.
   *
   * @default null
   */
  readonly pin?: DataGridPinSide | null;

  /**
   * Sticky inset from the pinned edge.
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
   * Whether clicking, Enter, or Space toggles sorting (Shift adds a sort key).
   *
   * @default false
   */
  readonly sortable?: boolean;

  /**
   * Current sort direction.
   *
   * @default null
   */
  readonly sortDirection?: DataGridSortDirection | null;

  /**
   * One-based priority among sort keys.
   *
   * @default null
   */
  readonly sortIndex?: number | null;

  /**
   * Render a pointer and keyboard resize handle.
   *
   * @default false
   */
  readonly resizable?: boolean;
}>();

defineSlots<{
  /** Header contents rendered by the grid root. */
  default(): unknown;
}>();

const grid = dataGridContext.use();
const active = computed(
  () => grid.activeCell.value?.rowId === null && grid.activeCell.value.columnId === columnId,
);
const tabStop = computed(() => active.value || (grid.activeCell.value === null && colIndex === 1));

function onClick(event: MouseEvent): void {
  grid.focusCell({ rowId: null, columnId });
  if (sortable) grid.toggleSort(columnId, event.shiftKey);
}

const headerProps = computed(() => ({
  role: "columnheader",
  tabindex: tabStop.value ? 0 : -1,
  style: {
    "--vize-ui-data-grid-column-width": `${width}px`,
    "--vize-ui-data-grid-pin-offset": `${pinOffset}px`,
  },
  onClick,
}));
</script>

<template>
  <div
    v-bind="headerProps"
    :aria-colindex="colIndex"
    :aria-sort="sortable ? (sortDirection ?? 'none') : undefined"
    data-vize-ui="data-grid-column-header"
    part="column-header"
    :data-column-id="columnId"
    :data-align="align"
    :data-pin="pin ?? undefined"
    :data-sort="sortDirection ?? undefined"
    :data-sort-index="sortIndex ?? undefined"
    :data-sortable="sortable ? 'true' : undefined"
    :data-active="active ? 'true' : undefined"
  >
    <slot />
    <DataGridResizeHandle
      v-if="resizable"
      :column-id
      :label="label ?? columnId"
      :width
      :min-width
      :max-width
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
