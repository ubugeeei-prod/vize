<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { panZoomContext } from "./pan-zoom-context.ts";
import type { PanZoomContextValue } from "./pan-zoom-context.ts";
import {
  PAN_ZOOM_IDENTITY,
  clampScale,
  clampToBounds,
  coverScale,
  createTransform,
  fitTransform,
  panBy as translate,
  transformEquals,
  zoomAt,
} from "./pan-zoom-transform.ts";
import type {
  PanZoomBounds,
  PanZoomChangeSource,
  PanZoomMessages,
  PanZoomPoint,
  PanZoomRootExpose,
  PanZoomSize,
  PanZoomSlotState,
  PanZoomState,
  PanZoomTransform,
  PanZoomWheelMode,
} from "./pan-zoom-types.ts";

const {
  modelValue = undefined,
  defaultValue = PAN_ZOOM_IDENTITY,
  minScale = 0.25,
  maxScale = 8,
  bounds = "none",
  zoomStep = 1.25,
  panStep = 40,
  wheelMode = "zoom",
  doubleClickZoom = true,
  fitPadding = 0,
  contentSize = undefined,
  disabled = false,
  messages = undefined,
} = defineProps<{
  /**
   * Controlled transform (`v-model`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: PanZoomTransform;

  /**
   * Initial uncontrolled transform, also restored by `reset()` and the `0` key.
   *
   * @default { x: 0, y: 0, scale: 1 }
   */
  readonly defaultValue?: PanZoomTransform;

  /**
   * Smallest zoom factor.
   *
   * @default 0.25
   */
  readonly minScale?: number;

  /**
   * Largest zoom factor.
   *
   * @default 8
   */
  readonly maxScale?: number;

  /**
   * Pan and zoom constraint applied to every change.
   *
   * @default "none"
   */
  readonly bounds?: PanZoomBounds;

  /**
   * Multiplicative factor of one zoom-in or zoom-out step.
   *
   * @default 1.25
   */
  readonly zoomStep?: number;

  /**
   * Viewport pixels moved by one arrow key press (Shift moves four times as far).
   *
   * @default 40
   */
  readonly panStep?: number;

  /**
   * Wheel behavior. Trackpad pinches (`ctrlKey` wheels) always zoom.
   *
   * @default "zoom"
   */
  readonly wheelMode?: PanZoomWheelMode;

  /**
   * Zoom in one step around the pointer on double-click (Shift zooms out).
   *
   * @default true
   */
  readonly doubleClickZoom?: boolean;

  /**
   * Inset kept around the content by `fit()` and the Home key.
   *
   * @default 0
   */
  readonly fitPadding?: number;

  /**
   * Unscaled content size. `undefined` measures the PanZoomContent element.
   *
   * @default undefined
   */
  readonly contentSize?: PanZoomSize;

  /**
   * Suppress every pointer, wheel, keyboard, and button interaction.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Localized role description, button labels, and zoom announcement.
   *
   * @default undefined
   */
  readonly messages?: PanZoomMessages;
}>();

const emit = defineEmits<{
  /** Fired with every requested transform. */
  "update:modelValue": [transform: PanZoomTransform];

  /** Fired after every distinct transform request with its source. */
  change: [transform: PanZoomTransform, previous: PanZoomTransform, source: PanZoomChangeSource];

  /** Fired when a continuous gesture (pan, pinch, wheel burst) starts. */
  transformStart: [source: PanZoomChangeSource];

  /** Fired when a continuous gesture ends. */
  transformEnd: [source: PanZoomChangeSource];
}>();

defineSlots<{
  /** Viewport, content, controls, and status. Receives the current transform state. */
  default(props: PanZoomSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const transformState = useControllableState<PanZoomTransform>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  equals: transformEquals,
});
const transform = computed(() => transformState.value.value);
const gesture = shallowRef<Exclude<PanZoomState, "disabled"> | null>(null);
const settledScale = shallowRef(transform.value.scale);
let viewportElement: HTMLElement | null = null;
let contentElement: HTMLElement | null = null;

const limits = computed(() => ({ minScale, maxScale }));
const disabledState = computed(() => disabled);
const state = computed<PanZoomState>(() => {
  if (disabled) return "disabled";
  return gesture.value === "panning" || gesture.value === "pinching" ? gesture.value : "idle";
});
const canZoomIn = computed(
  () => !disabled && transform.value.scale < clampScale(maxScale, limits.value) - 1e-6,
);
const canZoomOut = computed(() => !disabled && transform.value.scale > effectiveMinScale() + 1e-6);
const slotState = computed<PanZoomSlotState>(() => ({
  canZoomIn: canZoomIn.value,
  canZoomOut: canZoomOut.value,
  scale: transform.value.scale,
  state: state.value,
  transform: transform.value,
}));
const resolvedMessages = computed(() => ({
  fit: messages?.fit ?? "Fit to view",
  reset: messages?.reset ?? "Reset zoom",
  roleDescription: messages?.roleDescription ?? "pan and zoom area",
  zoomIn: messages?.zoomIn ?? "Zoom in",
  zoomOut: messages?.zoomOut ?? "Zoom out",
}));
const statusText = computed(() => {
  const percent = Math.round(settledScale.value * 100);
  return messages?.zoomLevel?.(percent) ?? `${percent}%`;
});

function viewportSize(): PanZoomSize {
  if (viewportElement === null) return { width: 0, height: 0 };
  const rect = viewportElement.getBoundingClientRect();
  return { width: rect.width, height: rect.height };
}

function measuredContentSize(): PanZoomSize {
  if (contentSize !== undefined) return contentSize;
  if (contentElement === null) return { width: 0, height: 0 };
  if (contentElement.offsetWidth > 0 && contentElement.offsetHeight > 0) {
    return { width: contentElement.offsetWidth, height: contentElement.offsetHeight };
  }
  const rect = contentElement.getBoundingClientRect();
  return {
    width: rect.width / transform.value.scale,
    height: rect.height / transform.value.scale,
  };
}

function effectiveMinScale(): number {
  const base = clampScale(minScale, limits.value);
  if (bounds !== "cover") return base;
  return Math.max(base, coverScale(viewportSize(), measuredContentSize()));
}

function constrain(next: PanZoomTransform): PanZoomTransform {
  const scale = Math.min(
    clampScale(maxScale, limits.value),
    Math.max(clampScale(minScale, limits.value), next.scale),
  );
  const scaled = createTransform(next.x, next.y, scale);
  return clampToBounds(scaled, viewportSize(), measuredContentSize(), bounds);
}

function center(): PanZoomPoint {
  const size = viewportSize();
  return { x: size.width / 2, y: size.height / 2 };
}

function currentTransform(): PanZoomTransform {
  return transform.value;
}

function setTransform(next: PanZoomTransform, source: PanZoomChangeSource): boolean {
  if (disabled && source !== "api") return false;
  const previous = currentTransform();
  const constrained = constrain(next);
  if (transformEquals(previous, constrained)) return false;
  transformState.set(constrained);
  emit("update:modelValue", constrained);
  emit("change", constrained, previous, source);
  if (gesture.value === null) settledScale.value = constrained.scale;
  return true;
}

function zoomTo(scale: number, point: PanZoomPoint | undefined, source: PanZoomChangeSource) {
  const clamped = clampScale(scale, limits.value);
  return setTransform(zoomAt(currentTransform(), clamped, point ?? center()), source);
}

function zoomBy(steps: number, point: PanZoomPoint | undefined, source: PanZoomChangeSource) {
  const step = zoomStep > 1 ? zoomStep : 1.25;
  return zoomTo(currentTransform().scale * step ** steps, point, source);
}

function pan(dx: number, dy: number, source: PanZoomChangeSource): boolean {
  return setTransform(translate(currentTransform(), dx, dy), source);
}

function reset(source: PanZoomChangeSource): boolean {
  return setTransform(defaultValue, source);
}

function fit(source: PanZoomChangeSource): boolean {
  return setTransform(
    fitTransform(viewportSize(), measuredContentSize(), limits.value, fitPadding),
    source,
  );
}

function beginGesture(next: Exclude<PanZoomState, "disabled">, source: PanZoomChangeSource) {
  const wasIdle = gesture.value === null;
  gesture.value = next;
  if (wasIdle) emit("transformStart", source);
}

function endGesture(source: PanZoomChangeSource): void {
  if (gesture.value === null) return;
  gesture.value = null;
  settledScale.value = currentTransform().scale;
  emit("transformEnd", source);
}

panZoomContext.provide({
  beginGesture,
  disabled: disabledState,
  doubleClickZoom: computed(() => doubleClickZoom),
  endGesture,
  fit,
  limits,
  messages: resolvedMessages,
  minScale: computed(() => minScale),
  panBy: pan,
  panStep: computed(() => panStep),
  reset,
  setContentElement(next) {
    contentElement = next;
  },
  setTransform,
  setViewportElement(next) {
    viewportElement = next;
  },
  slotState,
  state,
  statusText,
  transform,
  wheelMode: computed(() => wheelMode),
  zoomBy,
  zoomTo,
} satisfies PanZoomContextValue);

type PanZoomRootSetupExpose = Omit<PanZoomRootExpose, keyof PanZoomSlotState | "element"> & {
  readonly canZoomIn: ComputedRef<boolean>;
  readonly canZoomOut: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly scale: ComputedRef<number>;
  readonly state: ComputedRef<PanZoomState>;
  readonly transform: ComputedRef<PanZoomTransform>;
};

const exposed = {
  canZoomIn,
  canZoomOut,
  element,
  fit: () => fit("api"),
  panBy: (dx: number, dy: number) => pan(dx, dy, "api"),
  reset: () => reset("api"),
  scale: computed(() => transform.value.scale),
  setTransform: (next: PanZoomTransform) => setTransform(next, "api"),
  state,
  transform,
  zoomIn: (point?: PanZoomPoint) => zoomBy(1, point, "api"),
  zoomOut: (point?: PanZoomPoint) => zoomBy(-1, point, "api"),
  zoomTo: (scale: number, point?: PanZoomPoint) => zoomTo(scale, point, "api"),
} satisfies PanZoomRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="pan-zoom-root"
    part="root"
    :data-state="state"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
