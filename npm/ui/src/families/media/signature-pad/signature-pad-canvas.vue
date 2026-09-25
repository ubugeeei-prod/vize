<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { signaturePadContext } from "./signature-pad-context.ts";
import { simulatePressure, strokeOutlinePath } from "./signature-pad-path.ts";
import type {
  SignaturePadCanvasExpose,
  SignaturePadSlotState,
  SignaturePoint,
  SignatureStroke,
} from "./signature-pad-types.ts";

const { ariaLabel = "Signature", ariaDescribedby = undefined } = defineProps<{
  /**
   * Accessible name of the drawing surface. Localize by passing your own text.
   *
   * @default "Signature"
   */
  readonly ariaLabel?: string;

  /**
   * Ids of instructions, e.g. a note offering a typed-name alternative.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Extra SVG content drawn above the strokes, e.g. a baseline. Receives the pad state. */
  default(props: SignaturePadSlotState): unknown;
}>();

const context = signaturePadContext.use();
const element = useTemplateRef<SVGSVGElement>("element");
const viewBox = computed(() => `0 0 ${context.width.value} ${context.height.value}`);
const strokeKeys = new WeakMap<SignatureStroke, number>();
let nextStrokeKey = 0;

function keyOf(stroke: SignatureStroke): number {
  let key = strokeKeys.get(stroke);
  if (key === undefined) {
    key = nextStrokeKey;
    nextStrokeKey += 1;
    strokeKeys.set(stroke, key);
  }
  return key;
}

interface RenderedStroke {
  readonly key: number;
  readonly d: string;
}

const paths = computed<readonly RenderedStroke[]>(() =>
  context.value.value
    .map((stroke) => ({
      d: strokeOutlinePath(stroke, context.strokeOptions.value),
      key: keyOf(stroke),
    }))
    .filter((rendered) => rendered.d.length > 0),
);
const livePath = computed(() =>
  context.liveStroke.value === null
    ? ""
    : strokeOutlinePath(context.liveStroke.value, context.strokeOptions.value),
);
const drawing = computed(() => context.liveStroke.value !== null);
let pointerId: number | null = null;
let startTime = 0;

function samplesOf(event: PointerEvent): readonly PointerEvent[] {
  if (typeof event.getCoalescedEvents !== "function") return [event];
  const coalesced = event.getCoalescedEvents();
  return coalesced.length > 0 ? coalesced : [event];
}

function toPoint(event: PointerEvent, previous: SignaturePoint | undefined): SignaturePoint {
  const rect = element.value?.getBoundingClientRect();
  const scaleX = rect && rect.width > 0 ? context.width.value / rect.width : 1;
  const scaleY = rect && rect.height > 0 ? context.height.value / rect.height : 1;
  const x = (event.clientX - (rect?.left ?? 0)) * scaleX;
  const y = (event.clientY - (rect?.top ?? 0)) * scaleY;
  const time = Math.max(0, event.timeStamp - startTime);
  const mode = context.pressureMode.value;
  const usePointer = mode === "pointer" || (mode === "auto" && event.pointerType === "pen");
  const pressure = usePointer
    ? Math.min(1, Math.max(0, event.pressure))
    : simulatePressure(previous, x, y, time);
  return Object.freeze({ x, y, pressure, time });
}

function onPointerDown(event: PointerEvent): void {
  if (pointerId !== null || !context.interactive.value) return;
  if (event.pointerType === "mouse" && event.button !== 0) return;
  startTime = event.timeStamp;
  if (!context.startStroke(toPoint(event, undefined))) return;
  pointerId = event.pointerId;
  event.preventDefault();
  try {
    element.value?.setPointerCapture(event.pointerId);
  } catch {
    // Capture can fail for synthetic or already-released pointers; drawing continues.
  }
}

function onPointerMove(event: PointerEvent): void {
  if (event.pointerId !== pointerId) return;
  event.preventDefault();
  const points: SignaturePoint[] = [];
  let previous = context.liveStroke.value?.points.at(-1);
  for (const sample of samplesOf(event)) {
    const next = toPoint(sample, previous);
    points.push(next);
    previous = next;
  }
  context.extendStroke(points);
}

function onPointerUp(event: PointerEvent): void {
  if (event.pointerId !== pointerId) return;
  pointerId = null;
  context.finishStroke();
}

function onPointerCancel(event: PointerEvent): void {
  if (event.pointerId !== pointerId) return;
  pointerId = null;
  context.cancelStroke();
}

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  // Drawing must not scroll or zoom the page on touch devices.
  if (element.value !== null) element.value.style.touchAction = "none";
  element.value?.addEventListener("pointerdown", onPointerDown);
  element.value?.addEventListener("pointermove", onPointerMove);
  element.value?.addEventListener("pointerup", onPointerUp);
  element.value?.addEventListener("pointercancel", onPointerCancel);
});

onBeforeUnmount(() => {
  element.value?.removeEventListener("pointerdown", onPointerDown);
  element.value?.removeEventListener("pointermove", onPointerMove);
  element.value?.removeEventListener("pointerup", onPointerUp);
  element.value?.removeEventListener("pointercancel", onPointerCancel);
  if (pointerId !== null) context.cancelStroke();
  pointerId = null;
});

type SignaturePadCanvasSetupExpose = Omit<SignaturePadCanvasExpose, "drawing" | "element"> & {
  readonly drawing: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { drawing, element } satisfies SignaturePadCanvasSetupExpose;

defineExpose(exposed);
</script>

<template>
  <svg
    ref="element"
    xmlns="http://www.w3.org/2000/svg"
    role="img"
    :aria-label="ariaLabel"
    :aria-describedby="ariaDescribedby"
    :viewBox
    data-vize-ui="signature-pad-canvas"
    part="canvas"
    :data-state="context.slotState.value.state"
    :data-drawing="drawing ? 'true' : undefined"
    :data-disabled="context.slotState.value.disabled ? 'true' : undefined"
  >
    <path
      v-for="stroke in paths as readonly RenderedStroke[]"
      :key="stroke.key"
      :d="stroke.d"
      fill="currentColor"
      data-vize-ui="signature-pad-stroke"
    />
    <path
      v-if="livePath.length > 0"
      :d="livePath"
      fill="currentColor"
      data-vize-ui="signature-pad-live-stroke"
    />
    <slot v-bind="context.slotState.value" />
  </svg>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
