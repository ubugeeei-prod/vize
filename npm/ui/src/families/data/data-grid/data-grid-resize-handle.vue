<script setup lang="ts">
import { computed, shallowRef } from "vue";

import { dataGridContext } from "./data-grid-context.ts";

const {
  columnId,
  label = undefined,
  width = 150,
  minWidth = 40,
  maxWidth = Number.POSITIVE_INFINITY,
  step = 10,
} = defineProps<{
  /**
   * Column id resized by this handle.
   *
   * @default undefined
   */
  readonly columnId: string;

  /**
   * Column label used in the handle's accessible name.
   *
   * @default undefined
   */
  readonly label?: string;

  /**
   * Current width, published as `aria-valuenow`.
   *
   * @default 150
   */
  readonly width?: number;

  /**
   * Minimum width, published as `aria-valuemin`.
   *
   * @default 40
   */
  readonly minWidth?: number;

  /**
   * Maximum width, published as `aria-valuemax` when finite.
   *
   * @default Infinity
   */
  readonly maxWidth?: number;

  /**
   * Pixels per Arrow key press.
   *
   * @default 10
   */
  readonly step?: number;
}>();

const grid = dataGridContext.use();
const dragging = shallowRef(false);
let originX = 0;
let originWidth = 0;
// Read the width prop lazily so drags start from the latest committed width.
const currentWidth = (): number => width;

function onPointerdown(event: PointerEvent): void {
  if (event.button !== 0) return;
  event.preventDefault();
  event.stopPropagation();
  dragging.value = true;
  originX = event.clientX;
  originWidth = currentWidth();
  const target = event.currentTarget;
  if (target instanceof Element && "setPointerCapture" in target) {
    try {
      target.setPointerCapture(event.pointerId);
    } catch {
      // Synthetic pointers cannot be captured; movement still arrives by bubbling.
    }
  }
}

function onPointermove(event: PointerEvent): void {
  if (!dragging.value) return;
  grid.resizeColumn(columnId, originWidth + (event.clientX - originX));
}

function onPointerup(): void {
  dragging.value = false;
}

function onKeydown(event: KeyboardEvent): void {
  const delta = event.key === "ArrowRight" ? step : event.key === "ArrowLeft" ? -step : 0;
  if (delta === 0) return;
  event.preventDefault();
  event.stopPropagation();
  grid.resizeColumn(columnId, width + delta);
}

const handleProps = computed(() => ({
  role: "separator",
  tabindex: -1,
  "aria-orientation": "vertical" as const,
  "aria-valuenow": width,
  "aria-valuemin": minWidth,
  ...(Number.isFinite(maxWidth) ? { "aria-valuemax": maxWidth } : {}),
  "aria-label": label ? `Resize ${label}` : "Resize column",
  onClick: (event: MouseEvent) => event.stopPropagation(),
  onKeydown,
  onPointercancel: onPointerup,
  onPointerdown,
  onPointermove,
  onPointerup,
}));
</script>

<template>
  <div
    v-bind="handleProps"
    data-vize-ui="data-grid-resize-handle"
    part="resize-handle"
    :data-dragging="dragging ? 'true' : undefined"
  ></div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
