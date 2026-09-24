<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { panZoomContext } from "./pan-zoom-context.ts";
import { normalizeWheelDelta, pinchTransform, wheelZoomFactor } from "./pan-zoom-transform.ts";
import type { PanZoomPointerPair } from "./pan-zoom-transform.ts";
import type {
  PanZoomPoint,
  PanZoomSlotState,
  PanZoomTransform,
  PanZoomViewportExpose,
} from "./pan-zoom-types.ts";

const {
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  wheelSettleDelay = 150,
} = defineProps<{
  /**
   * Accessible name of the pan and zoom area.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the area.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids of usage instructions (e.g. "Arrow keys pan, plus and minus zoom").
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Quiet period after the last wheel event before `transformEnd` fires.
   *
   * @default 150
   */
  readonly wheelSettleDelay?: number;
}>();

defineSlots<{
  /** PanZoomContent and overlays. Receives the current transform state. */
  default(props: PanZoomSlotState): unknown;
}>();

const context = panZoomContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const pointers = new Map<number, PanZoomPoint>();
let panStart: { readonly point: PanZoomPoint; readonly transform: PanZoomTransform } | null = null;
let pinchStart: { readonly pair: PanZoomPointerPair; readonly transform: PanZoomTransform } | null =
  null;
let wheelTimer: ReturnType<typeof setTimeout> | undefined;

function localPoint(event: MouseEvent): PanZoomPoint {
  if (element.value === null) return { x: event.clientX, y: event.clientY };
  const rect = element.value.getBoundingClientRect();
  return { x: event.clientX - rect.left, y: event.clientY - rect.top };
}

function currentPair(): PanZoomPointerPair | null {
  const [first, second] = [...pointers.values()];
  return first !== undefined && second !== undefined ? { first, second } : null;
}

function restartGesture(): void {
  const pair = currentPair();
  if (pair !== null) {
    pinchStart = { pair, transform: context.transform.value };
    panStart = null;
    context.beginGesture("pinching", "pinch");
    return;
  }
  pinchStart = null;
  const [only] = [...pointers.values()];
  panStart = only === undefined ? null : { point: only, transform: context.transform.value };
  if (only !== undefined) context.beginGesture("panning", "pointer");
}

function onPointerDown(event: PointerEvent): void {
  if (context.disabled.value || (event.pointerType === "mouse" && event.button !== 0)) return;
  pointers.set(event.pointerId, localPoint(event));
  try {
    element.value?.setPointerCapture(event.pointerId);
  } catch {
    // Synthetic or already released pointers cannot be captured.
  }
  restartGesture();
}

function onPointerMove(event: PointerEvent): void {
  if (!pointers.has(event.pointerId)) return;
  pointers.set(event.pointerId, localPoint(event));
  const pair = currentPair();
  if (pinchStart !== null && pair !== null) {
    context.setTransform(
      pinchTransform(pinchStart.transform, pinchStart.pair, pair, context.limits.value),
      "pinch",
    );
  } else if (panStart !== null) {
    const point = pointers.get(event.pointerId) ?? panStart.point;
    context.setTransform(
      {
        x: panStart.transform.x + point.x - panStart.point.x,
        y: panStart.transform.y + point.y - panStart.point.y,
        scale: panStart.transform.scale,
      },
      "pointer",
    );
  }
  if (event.cancelable) event.preventDefault();
}

function onPointerEnd(event: PointerEvent): void {
  if (!pointers.delete(event.pointerId)) return;
  const source = pinchStart !== null ? "pinch" : "pointer";
  if (pointers.size === 0) {
    panStart = null;
    pinchStart = null;
    context.endGesture(source);
    return;
  }
  restartGesture();
}

function onWheel(event: WheelEvent): void {
  if (context.disabled.value || element.value === null) return;
  const mode = context.wheelMode.value;
  const pinch = event.ctrlKey || event.metaKey;
  const rect = element.value.getBoundingClientRect();
  const delta = normalizeWheelDelta(event, { width: rect.width, height: rect.height });
  if (mode === "zoom-with-ctrl" && !pinch) return;
  event.preventDefault();
  if (context.state.value === "idle") context.beginGesture("idle", "wheel");
  if (mode === "pan" && !pinch) {
    context.panBy(-delta.x, -delta.y, "wheel");
  } else {
    context.zoomTo(
      context.transform.value.scale * wheelZoomFactor(delta.y, pinch),
      localPoint(event),
      "wheel",
    );
  }
  if (wheelTimer !== undefined) clearTimeout(wheelTimer);
  wheelTimer = setTimeout(
    () => {
      wheelTimer = undefined;
      context.endGesture("wheel");
    },
    Math.max(0, wheelSettleDelay),
  );
}

function onDoubleClick(event: MouseEvent): void {
  if (context.disabled.value || !context.doubleClickZoom.value) return;
  event.preventDefault();
  context.zoomBy(event.shiftKey ? -1 : 1, localPoint(event), "double-click");
}

function onKeydown(event: KeyboardEvent): void {
  if (context.disabled.value || event.target !== element.value) return;
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  const step = context.panStep.value * (event.shiftKey ? 4 : 1);
  let handled = true;
  switch (event.key) {
    case "ArrowLeft":
      context.panBy(step, 0, "keyboard");
      break;
    case "ArrowRight":
      context.panBy(-step, 0, "keyboard");
      break;
    case "ArrowUp":
      context.panBy(0, step, "keyboard");
      break;
    case "ArrowDown":
      context.panBy(0, -step, "keyboard");
      break;
    case "+":
    case "=":
      context.zoomBy(1, undefined, "keyboard");
      break;
    case "-":
    case "_":
      context.zoomBy(-1, undefined, "keyboard");
      break;
    case "0":
      context.reset("keyboard");
      break;
    case "Home":
      context.fit("keyboard");
      break;
    default:
      handled = false;
  }
  if (handled) event.preventDefault();
}

// Listeners and touch-action attach on the client only, keeping server markup inert.
onMounted(() => {
  context.setViewportElement(element.value);
  if (element.value === null) return;
  element.value.style.touchAction = "none";
  element.value.addEventListener("pointerdown", onPointerDown);
  element.value.addEventListener("pointermove", onPointerMove);
  element.value.addEventListener("pointerup", onPointerEnd);
  element.value.addEventListener("pointercancel", onPointerEnd);
  element.value.addEventListener("wheel", onWheel, { passive: false });
  element.value.addEventListener("dblclick", onDoubleClick);
  element.value.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  if (wheelTimer !== undefined) clearTimeout(wheelTimer);
  context.setViewportElement(null);
  if (element.value === null) return;
  element.value.removeEventListener("pointerdown", onPointerDown);
  element.value.removeEventListener("pointermove", onPointerMove);
  element.value.removeEventListener("pointerup", onPointerEnd);
  element.value.removeEventListener("pointercancel", onPointerEnd);
  element.value.removeEventListener("wheel", onWheel);
  element.value.removeEventListener("dblclick", onDoubleClick);
  element.value.removeEventListener("keydown", onKeydown);
});

type PanZoomViewportSetupExpose = Omit<PanZoomViewportExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies PanZoomViewportSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    role="group"
    :tabindex="context.disabled.value ? undefined : 0"
    :aria-roledescription="context.messages.value.roleDescription"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-disabled="context.disabled.value ? 'true' : undefined"
    data-vize-ui="pan-zoom-viewport"
    part="viewport"
    :data-state="context.state.value"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Consumers own the clipping box, for example:
   overflow: hidden; position: relative; */
</style>
