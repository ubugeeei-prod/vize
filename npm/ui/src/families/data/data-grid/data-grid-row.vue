<script setup lang="ts">
import { computed } from "vue";

import { dataGridContext } from "./data-grid-context.ts";

const {
  rowId,
  rowIndex,
  selected = false,
  depth = 0,
  expandable = false,
  expanded = false,
  start = null,
} = defineProps<{
  /**
   * Row id.
   *
   * @default undefined
   */
  readonly rowId: string;

  /**
   * One-based `aria-rowindex` (the header row is `1`).
   *
   * @default undefined
   */
  readonly rowIndex: number;

  /**
   * Whether the row is selected.
   *
   * @default false
   */
  readonly selected?: boolean;

  /**
   * Zero-based tree depth, published as `aria-level` in tree grids.
   *
   * @default 0
   */
  readonly depth?: number;

  /**
   * Whether the row has sub rows.
   *
   * @default false
   */
  readonly expandable?: boolean;

  /**
   * Whether the sub rows are shown.
   *
   * @default false
   */
  readonly expanded?: boolean;

  /**
   * Virtualized offset in CSS pixels, published as `--vize-ui-data-grid-row-start`.
   * `null` means the row is laid out in normal flow.
   *
   * @default null
   */
  readonly start?: number | null;
}>();

defineSlots<{
  /** Row cells rendered by the grid root. */
  default(): unknown;
}>();

const grid = dataGridContext.use();
const rowProps = computed(() => ({
  role: "row",
  onClick: (event: MouseEvent) => grid.rowClick(event, rowId),
  ...(start === null ? {} : { style: { "--vize-ui-data-grid-row-start": `${start}px` } }),
}));
</script>

<template>
  <div
    v-bind="rowProps"
    :aria-rowindex="rowIndex"
    :aria-selected="grid.selectionMode.value === 'none' ? undefined : selected ? 'true' : 'false'"
    :aria-level="grid.treeGrid.value ? depth + 1 : undefined"
    :aria-expanded="grid.treeGrid.value && expandable ? (expanded ? 'true' : 'false') : undefined"
    data-vize-ui="data-grid-row"
    part="row"
    :data-row-id="rowId"
    :data-selected="selected ? 'true' : undefined"
    :data-depth="depth"
    :data-virtual="start === null ? undefined : 'true'"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
