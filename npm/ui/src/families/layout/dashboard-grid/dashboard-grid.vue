<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { CSSProperties } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { dashboardGridContext } from "./dashboard-grid-context.ts";
import {
  dashboardLayoutRows,
  moveDashboardItem,
  resizeDashboardItem,
} from "./dashboard-grid-layout.ts";
import type {
  DashboardCompaction,
  DashboardItem,
  DashboardLayout,
} from "./dashboard-grid-layout.ts";
import type { DashboardGridSlotState } from "./dashboard-grid-types.ts";

const {
  layout = undefined,
  defaultLayout = [],
  columns = 12,
  rowHeight = 48,
  gap = 8,
  compaction = "vertical",
  disabled = false,
  label = undefined,
} = defineProps<{
  /**
   * Controlled widget placement (`v-model:layout`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly layout?: DashboardLayout;

  /**
   * Initial uncontrolled placement.
   *
   * @default []
   */
  readonly defaultLayout?: DashboardLayout;

  /**
   * Number of grid columns.
   *
   * @default 12
   */
  readonly columns?: number;

  /**
   * Height of one grid row in CSS pixels.
   *
   * @default 48
   */
  readonly rowHeight?: number;

  /**
   * Gap between cells in CSS pixels.
   *
   * @default 8
   */
  readonly gap?: number;

  /**
   * Compaction after each change: pull widgets up (`"vertical"`) or leave gaps (`"none"`).
   *
   * @default "vertical"
   */
  readonly compaction?: DashboardCompaction;

  /**
   * Disable dragging and resizing.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name of the dashboard region.
   *
   * @default undefined
   */
  readonly label?: string;
}>();

const emit = defineEmits<{
  /** Fired with the next placement after a widget moves or resizes. */
  "update:layout": [layout: DashboardLayout];
}>();

defineSlots<{
  /** DashboardGridItem children. Receives the occupied row count. */
  default?(props: DashboardGridSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const state = useControllableState<DashboardLayout>({
  value: () => layout,
  defaultValue: () => defaultLayout,
  onChange: (next) => emit("update:layout", next),
});
const columnCount = computed(() => Math.max(1, Math.floor(columns)));
const rows = computed(() => dashboardLayoutRows(state.value.value));

function withItem(item: DashboardItem): DashboardLayout {
  const current = state.value.value;
  return current.some((entry) => entry.id === item.id) ? current : [...current, item];
}

dashboardGridContext.provide({
  itemOf: (id, fallback) => state.value.value.find((item) => item.id === id) ?? fallback,
  disabled: computed(() => disabled),
  cellSize: () => ({
    width: element.value ? (element.value.clientWidth + gap) / columnCount.value : 0,
    height: rowHeight + gap,
  }),
  move: (item, x, y) => {
    const current = withItem(item);
    const next = moveDashboardItem(current, item.id, x, y, columnCount.value, compaction);
    if (next !== current) state.set(next);
  },
  resize: (item, w, h) => {
    const current = withItem(item);
    const next = resizeDashboardItem(current, item.id, w, h, columnCount.value, compaction);
    if (next !== current) state.set(next);
  },
});

const gridStyle = computed<CSSProperties>(() => ({
  display: "grid",
  gridTemplateColumns: `repeat(${columnCount.value}, minmax(0, 1fr))`,
  gridAutoRows: `${rowHeight}px`,
  gap: `${gap}px`,
}));

defineExpose({ element, layout: state.value, rows });
</script>

<template>
  <div
    ref="element"
    role="region"
    :aria-label="label"
    data-vize-ui="dashboard-grid"
    :data-columns="columnCount"
    :data-disabled="disabled ? '' : undefined"
    :style="gridStyle"
  >
    <slot :rows="rows" />
  </div>
</template>

<style scoped>
/* Headless by design. Widgets are placed with native CSS grid lines. */
</style>
