<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useMove } from "../../interaction/move/move.ts";
import { resizableContext } from "./resizable-context.ts";
import {
  constrainResizableSize,
  horizontalSign,
  resizeByDelta,
  resolveResizableEdge,
  verticalSign,
} from "./resizable-geometry.ts";
import type {
  ResizableEdge,
  ResizableHandleExpose,
  ResizablePhysicalEdge,
  ResizableSize,
  ResizableSlotState,
} from "./resizable-types.ts";

const edgeNames: Readonly<Record<ResizablePhysicalEdge, string>> = {
  e: "right edge",
  n: "top edge",
  ne: "top-right corner",
  nw: "top-left corner",
  s: "bottom edge",
  se: "bottom-right corner",
  sw: "bottom-left corner",
  w: "left edge",
};

const { edge = "se", ariaLabel = undefined } = defineProps<{
  /**
   * Edge or corner this handle resizes from. `start`/`end` follow the root `dir`.
   *
   * @default "se"
   */
  readonly edge?: ResizableEdge;

  /**
   * Accessible name. `undefined` describes the resolved edge, e.g. "Resize from right edge".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Optional grip contents. Receives the current Resizable state. */
  default?(props: ResizableSlotState): unknown;
}>();

const context = resizableContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const physicalEdge = computed(() => resolveResizableEdge(edge, context.dir.value));
const sx = computed(() => horizontalSign(physicalEdge.value));
const sy = computed(() => verticalSign(physicalEdge.value));
const controlsWidth = computed(() => sx.value !== 0);
const orientation = computed(() => (controlsWidth.value ? "vertical" : "horizontal"));
const valueNow = computed(() =>
  Math.round(controlsWidth.value ? context.size.value.width : context.size.value.height),
);
const valueMin = computed(() =>
  controlsWidth.value ? context.constraints.value.minWidth : context.constraints.value.minHeight,
);
const valueMax = computed(() => {
  const max = controlsWidth.value
    ? context.constraints.value.maxWidth
    : context.constraints.value.maxHeight;
  return Number.isFinite(max) ? max : undefined;
});
const valueText = computed(
  () =>
    `Width ${Math.round(context.size.value.width)} pixels, height ${Math.round(context.size.value.height)} pixels`,
);
const label = computed(() => ariaLabel ?? `Resize from ${edgeNames[physicalEdge.value]}`);
const slotState = computed<ResizableSlotState>(() => ({
  disabled: context.disabled.value,
  size: context.size.value,
  state: context.state.value,
}));
let pointerDeltaX = 0;
let pointerDeltaY = 0;
let pointerActive = false;

const move = useMove({
  isDisabled: () => context.disabled.value,
  onMoveStart: (event) => {
    pointerActive = true;
    pointerDeltaX = 0;
    pointerDeltaY = 0;
    context.begin(physicalEdge.value, "pointer", event.originalEvent);
  },
  onMove: (event) => {
    const initial = context.initialSize();
    if (initial === null) return;
    pointerDeltaX += event.deltaX;
    pointerDeltaY += event.deltaY;
    applyDelta(initial, pointerDeltaX, pointerDeltaY, event.originalEvent);
  },
  onMoveEnd: (event) => {
    pointerActive = false;
    context.end(event.originalEvent);
  },
});

function applyDelta(
  initial: ResizableSize,
  deltaX: number,
  deltaY: number,
  event: Event | null,
): void {
  const next = resizeByDelta(
    initial,
    physicalEdge.value,
    deltaX,
    deltaY,
    context.constraints.value,
  );
  context.update(next, event, controlsWidth.value ? "width" : "height");
}

function onPointerdown(event: PointerEvent): void {
  if (context.disabled.value) return;
  if (typeof element.value?.setPointerCapture === "function") {
    try {
      element.value.setPointerCapture(event.pointerId);
    } catch {
      // Pointer capture is best effort; move tracking still follows the document.
    }
  }
  move.moveProps.onPointerdown(event);
}

function keyboardTarget(event: KeyboardEvent): ResizableSize | null {
  const current = context.size.value;
  const constraints = context.constraints.value;
  const amount = event.shiftKey ? context.largeStep.value : context.step.value;
  if (event.key === "Home" || event.key === "End") {
    const toMax = event.key === "End";
    return {
      height:
        sy.value === 0 ? current.height : toMax ? constraints.maxHeight : constraints.minHeight,
      width: sx.value === 0 ? current.width : toMax ? constraints.maxWidth : constraints.minWidth,
    };
  }
  let deltaX = 0;
  let deltaY = 0;
  if (event.key === "ArrowRight") deltaX = amount;
  else if (event.key === "ArrowLeft") deltaX = -amount;
  else if (event.key === "ArrowDown") deltaY = amount;
  else if (event.key === "ArrowUp") deltaY = -amount;
  else return null;
  if ((deltaX !== 0 && sx.value === 0) || (deltaY !== 0 && sy.value === 0)) return null;
  return resizeByDelta(current, physicalEdge.value, deltaX, deltaY, constraints);
}

function onKeydown(event: KeyboardEvent): void {
  if (context.disabled.value || pointerActive || event.defaultPrevented) return;
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  const target = keyboardTarget(event);
  if (target === null) return;
  event.preventDefault();
  const driver = controlsWidth.value ? "width" : "height";
  const bounded = constrainResizableSize(
    {
      height: Number.isFinite(target.height) ? target.height : context.size.value.height,
      width: Number.isFinite(target.width) ? target.width : context.size.value.width,
    },
    context.constraints.value,
    driver,
  );
  context.begin(physicalEdge.value, "keyboard", event);
  context.update(bounded, event, driver);
  context.end(event);
}

const separatorProps = computed<{
  readonly role: "separator";
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onPointercancel: (event: PointerEvent) => void;
  readonly onMousedown: (event: MouseEvent) => void;
  readonly onTouchstart: (event: TouchEvent) => void;
  readonly onTouchmove: (event: TouchEvent) => void;
  readonly onTouchend: (event: TouchEvent) => void;
  readonly onTouchcancel: (event: TouchEvent) => void;
  readonly onDragstart: (event: DragEvent) => void;
}>(() => ({
  ...move.moveProps,
  onKeydown,
  onPointerdown,
  role: "separator",
}));

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type ResizableHandleSetupExpose = Omit<ResizableHandleExpose, "element" | "physicalEdge"> & {
  readonly element: typeof element;
  readonly physicalEdge: ComputedRef<ResizablePhysicalEdge>;
};

const exposed = { element, focus, physicalEdge } satisfies ResizableHandleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    v-bind="separatorProps"
    ref="element"
    :tabindex="context.disabled.value ? undefined : 0"
    :aria-label="label"
    :aria-controls="context.id.value"
    :aria-orientation="orientation"
    :aria-valuenow="valueNow"
    :aria-valuemin="valueMin"
    :aria-valuemax="valueMax"
    :aria-valuetext="sx !== 0 && sy !== 0 ? valueText : undefined"
    :aria-disabled="context.disabled.value ? 'true' : undefined"
    data-vize-ui="resizable-handle"
    part="handle"
    :data-edge="physicalEdge"
    :data-state="context.state.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
