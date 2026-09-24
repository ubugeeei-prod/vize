<script setup lang="ts">
import { computed, shallowRef } from "vue";
import type { CSSProperties } from "vue";

import { dashboardGridContext } from "./dashboard-grid-context.ts";
import { createDashboardItem } from "./dashboard-grid-layout.ts";
import type { DashboardItem } from "./dashboard-grid-layout.ts";
import type {
  DashboardGridItemSlotState,
  DashboardHandleProps,
  DashboardResizeHandleProps,
} from "./dashboard-grid-types.ts";

const {
  id,
  label = undefined,
  defaultPosition = { x: 0, y: 0, w: 1, h: 1 },
} = defineProps<{
  /** Widget id matching its layout entry. @default required */
  readonly id: string;

  /**
   * Accessible name of the widget.
   *
   * @default undefined
   */
  readonly label?: string;

  /**
   * Placement used until the layout contains this widget.
   *
   * @default { x: 0, y: 0, w: 1, h: 1 }
   */
  readonly defaultPosition?: Omit<DashboardItem, "id">;
}>();

defineSlots<{
  /** Widget content. Receives placement, drag state, and handle props. */
  default?(props: DashboardGridItemSlotState): unknown;
}>();

const context = dashboardGridContext.use();
const item = computed(() => context.itemOf(id, createDashboardItem(id, defaultPosition)));
const dragging = shallowRef(false);

function track(
  event: PointerEvent,
  origin: DashboardItem,
  apply: (columns: number, rows: number, origin: DashboardItem) => void,
): void {
  if (event.button !== 0 || context.disabled.value || origin.static) return;
  const startX = event.clientX;
  const startY = event.clientY;
  const size = context.cellSize();
  const target =
    event.currentTarget instanceof Element ? event.currentTarget.ownerDocument : document;
  dragging.value = true;
  const onMove = (next: PointerEvent): void => {
    const columns = size.width > 0 ? Math.round((next.clientX - startX) / size.width) : 0;
    const rows = size.height > 0 ? Math.round((next.clientY - startY) / size.height) : 0;
    apply(columns, rows, origin);
  };
  const onUp = (): void => {
    dragging.value = false;
    target.removeEventListener("pointermove", onMove);
    target.removeEventListener("pointerup", onUp);
    target.removeEventListener("pointercancel", onUp);
  };
  target.addEventListener("pointermove", onMove);
  target.addEventListener("pointerup", onUp);
  target.addEventListener("pointercancel", onUp);
  event.preventDefault();
}

const handleProps = computed<DashboardHandleProps>(() => ({
  "data-part": "drag-handle",
  tabindex: 0,
  "aria-keyshortcuts":
    "ArrowUp ArrowDown ArrowLeft ArrowRight Shift+ArrowUp Shift+ArrowDown Shift+ArrowLeft Shift+ArrowRight",
  onPointerdown: (event) =>
    track(event, item.value, (columns, rows, origin) =>
      context.move(origin, origin.x + columns, origin.y + rows),
    ),
  onKeydown: (event) => stepWithKeyboard(event, item.value),
}));

const keyboardDeltas: Readonly<Record<string, readonly [number, number]>> = {
  ArrowUp: [0, -1],
  ArrowDown: [0, 1],
  ArrowLeft: [-1, 0],
  ArrowRight: [1, 0],
};

function stepWithKeyboard(event: KeyboardEvent, current: DashboardItem): void {
  const delta = keyboardDeltas[event.key];
  if (!delta || context.disabled.value || current.static) return;
  event.preventDefault();
  if (event.shiftKey) context.resize(current, current.w + delta[0], current.h + delta[1]);
  else context.move(current, current.x + delta[0], current.y + delta[1]);
}

const resizeHandleProps = computed<DashboardResizeHandleProps>(() => ({
  "data-part": "resize-handle",
  "aria-hidden": "true",
  onPointerdown: (event) =>
    track(event, item.value, (columns, rows, origin) =>
      context.resize(origin, origin.w + columns, origin.h + rows),
    ),
}));

const itemStyle = computed<CSSProperties>(() => ({
  gridColumn: `${item.value.x + 1} / span ${item.value.w}`,
  gridRow: `${item.value.y + 1} / span ${item.value.h}`,
}));

const slotState = computed<DashboardGridItemSlotState>(() => ({
  item: item.value,
  dragging: dragging.value,
  handleProps: handleProps.value,
  resizeHandleProps: resizeHandleProps.value,
}));

defineExpose({ item, dragging });
</script>

<template>
  <div
    role="group"
    aria-roledescription="dashboard widget"
    :aria-label="label"
    data-vize-ui="dashboard-grid-item"
    :data-dragging="dragging ? '' : undefined"
    :data-static="item.static ? '' : undefined"
    :style="itemStyle"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Placement uses native CSS grid lines. */
</style>
