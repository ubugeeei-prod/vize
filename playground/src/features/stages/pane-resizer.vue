<script setup lang="ts">
import { onMounted, onUnmounted, ref, useTemplateRef } from "vue";

const { label } = defineProps<{ label: string }>();
const emit = defineEmits<{ resize: [widths: number[]]; reset: [] }>();
const handle = useTemplateRef<HTMLElement>("handle");
const dragging = ref(false);
const value = ref(0);
const minimum = ref(0);
const maximum = ref(0);
let observer: ResizeObserver | null = null;
let drag: { pointer: number; x: number; layout: Layout } | null = null;

interface Layout {
  widths: number[];
  minimums: number[];
  index: number;
}

function readLayout(): Layout | null {
  const workspace = handle.value?.closest(".dv-workspace");
  if (!workspace) return null;
  const panes = [...workspace.querySelectorAll<HTMLElement>("[data-pane]")];
  const index = panes.indexOf(handle.value!.closest<HTMLElement>("[data-pane]")!) - 1;
  if (index < 0) return null;
  return {
    index,
    widths: panes.map((pane) => pane.getBoundingClientRect().width),
    minimums: panes.map((_, i) => (i === 0 ? 180 : 160)),
  };
}

function measure() {
  const layout = readLayout();
  if (!layout) return;
  const { widths, minimums, index } = layout;
  value.value = Math.round(widths.slice(0, index + 1).reduce((sum, width) => sum + width, 0));
  minimum.value = minimums.slice(0, index + 1).reduce((sum, width) => sum + width, 0);
  maximum.value = Math.round(
    widths.reduce((sum, width) => sum + width, 0) -
      minimums.slice(index + 1).reduce((sum, width) => sum + width, 0),
  );
}

function resize(layout: Layout, delta: number) {
  const { widths, minimums, index } = layout;
  const next = [...widths];
  const direction = delta > 0 ? 1 : -1;
  const recipient = delta > 0 ? index : index + 1;
  let remaining = Math.abs(delta);
  // Keep pushing adjacent panes when one reaches its minimum width.
  for (
    let i = delta > 0 ? index + 1 : index;
    i >= 0 && i < widths.length && remaining > 0;
    i += direction
  ) {
    const change = Math.min(Math.max(0, widths[i] - minimums[i]), remaining);
    next[i] -= change;
    next[recipient] += change;
    remaining -= change;
  }
  emit("resize", next);
}

function start(event: PointerEvent) {
  if (event.button !== 0) return;
  const layout = readLayout();
  if (!layout || !handle.value) return;
  event.preventDefault();
  handle.value.focus({ preventScroll: true });
  handle.value.setPointerCapture(event.pointerId);
  drag = { pointer: event.pointerId, x: event.clientX, layout };
  dragging.value = true;
}

function move(event: PointerEvent) {
  if (drag?.pointer === event.pointerId) resize(drag.layout, event.clientX - drag.x);
}

function finish(event: PointerEvent) {
  if (drag?.pointer !== event.pointerId) return;
  drag = null;
  dragging.value = false;
  if (handle.value?.hasPointerCapture(event.pointerId)) {
    handle.value.releasePointerCapture(event.pointerId);
  }
}

function onKey(event: KeyboardEvent) {
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  if (!["ArrowLeft", "ArrowRight", "Home", "End", "Enter"].includes(event.key)) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.key === "Enter") {
    emit("reset");
    return;
  }
  const layout = readLayout();
  if (!layout) return;
  const step = event.shiftKey ? 50 : 10;
  const delta =
    event.key === "Home"
      ? -Infinity
      : event.key === "End"
        ? Infinity
        : event.key === "ArrowLeft"
          ? -step
          : step;
  resize(layout, delta);
}

onMounted(() => {
  observer = new ResizeObserver(measure);
  handle.value
    ?.closest(".dv-workspace")
    ?.querySelectorAll("[data-pane]")
    .forEach((pane) => {
      observer!.observe(pane);
    });
  measure();
});
onUnmounted(() => observer?.disconnect());
</script>

<template>
  <!-- A focusable separator is an interactive ARIA widget, with keyboard resizing above. -->
  <!-- eslint-disable-next-line a11y/no-static-element-interactions -->
  <div
    ref="handle"
    :class="['dv-resizer', { dragging }]"
    role="separator"
    tabindex="0"
    aria-orientation="vertical"
    :aria-label="label"
    :aria-valuenow="value"
    :aria-valuemin="minimum"
    :aria-valuemax="maximum"
    :aria-valuetext="`${value} pixels from the left`"
    title="Drag to resize · Arrow keys to adjust · Double-click or Enter to reset widths"
    @pointerdown="start"
    @pointermove="move"
    @pointerup="finish"
    @pointercancel="finish"
    @lostpointercapture="finish"
    @keydown="onKey"
    @dblclick="emit('reset')"
  ></div>
</template>
