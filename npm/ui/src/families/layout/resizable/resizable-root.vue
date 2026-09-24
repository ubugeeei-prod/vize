<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { resizableContext } from "./resizable-context.ts";
import {
  constrainResizableSize,
  resizableSizeEquals,
  resolveResizableAspectRatio,
} from "./resizable-geometry.ts";
import type { ResizableConstraints } from "./resizable-geometry.ts";
import type {
  ResizableDirection,
  ResizablePhysicalEdge,
  ResizableResizeEvent,
  ResizableRootExpose,
  ResizableSize,
  ResizableSlotState,
  ResizableSource,
  ResizableState,
} from "./resizable-types.ts";

interface ResizableSession {
  readonly edge: ResizablePhysicalEdge;
  readonly source: ResizableSource;
  readonly initialSize: ResizableSize;
}

const {
  id = undefined,
  size = undefined,
  defaultSize = { height: 240, width: 320 },
  minWidth = 0,
  maxWidth = Number.POSITIVE_INFINITY,
  minHeight = 0,
  maxHeight = Number.POSITIVE_INFINITY,
  lockAspectRatio = false,
  step = 10,
  largeStep = 50,
  disabled = false,
  dir = "ltr",
} = defineProps<{
  /**
   * Consumer-owned root id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled size in CSS pixels. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly size?: ResizableSize;

  /**
   * Initial size for uncontrolled use.
   *
   * @default { width: 320, height: 240 }
   */
  readonly defaultSize?: ResizableSize;

  /**
   * Minimum width in CSS pixels.
   *
   * @default 0
   */
  readonly minWidth?: number;

  /**
   * Maximum width in CSS pixels.
   *
   * @default Infinity
   */
  readonly maxWidth?: number;

  /**
   * Minimum height in CSS pixels.
   *
   * @default 0
   */
  readonly minHeight?: number;

  /**
   * Maximum height in CSS pixels.
   *
   * @default Infinity
   */
  readonly maxHeight?: number;

  /**
   * Preserve the aspect ratio: `true` keeps the ratio from the start of each
   * interaction, a number fixes `width / height`.
   *
   * @default false
   */
  readonly lockAspectRatio?: boolean | number;

  /**
   * Arrow-key step in CSS pixels.
   *
   * @default 10
   */
  readonly step?: number;

  /**
   * Shift+Arrow step in CSS pixels.
   *
   * @default 50
   */
  readonly largeStep?: number;

  /**
   * Ignore pointer and keyboard resizing.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Reading direction used to resolve `start` and `end` handles.
   *
   * @default "ltr"
   */
  readonly dir?: ResizableDirection;
}>();

const emit = defineEmits<{
  /** Fired when the size requests a controlled value. */
  "update:size": [value: ResizableSize];

  /** Fired when a pointer or keyboard resize begins. */
  "resize-start": [event: ResizableResizeEvent];

  /** Fired for every distinct size produced by an interaction. */
  resize: [event: ResizableResizeEvent];

  /** Fired when a pointer or keyboard resize ends. */
  "resize-end": [event: ResizableResizeEvent];
}>();

defineSlots<{
  /** Resizable contents and ResizableHandle parts. Receives the current size. */
  default(props: ResizableSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const rootId = useDeterministicId({ id: () => id, hint: "resizable" });
const sizeState = useControllableState<ResizableSize>({
  value: () => size,
  defaultValue: () => defaultSize,
  equals: resizableSizeEquals,
});
const session = shallowRef<ResizableSession | null>(null);
const currentSize = computed(() => sizeState.value.value);
const disabledState = computed(() => disabled);
const dirState = computed(() => dir);
const stepState = computed(() => (Number.isFinite(step) && step > 0 ? step : 1));
const largeStepState = computed(() =>
  Number.isFinite(largeStep) && largeStep > 0 ? largeStep : stepState.value,
);
const constraints = computed<ResizableConstraints>(() => ({
  aspectRatio: resolveResizableAspectRatio(
    lockAspectRatio,
    readSession()?.initialSize ?? readSize(),
  ),
  maxHeight,
  maxWidth,
  minHeight,
  minWidth,
}));
const state = computed<ResizableState>(() => (session.value === null ? "idle" : "resizing"));
const slotState = computed<ResizableSlotState>(() => ({
  disabled: disabledState.value,
  size: currentSize.value,
  state: state.value,
}));
const style = computed(() => ({
  "--vize-resizable-height": `${currentSize.value.height}px`,
  "--vize-resizable-width": `${currentSize.value.width}px`,
  height: `${currentSize.value.height}px`,
  width: `${currentSize.value.width}px`,
}));

function readSession(): ResizableSession | null {
  return session.value;
}

function readSize(): ResizableSize {
  return currentSize.value;
}

function eventFor(active: ResizableSession, originalEvent: Event | null): ResizableResizeEvent {
  return Object.freeze({
    edge: active.edge,
    initialSize: active.initialSize,
    originalEvent,
    size: readSize(),
    source: active.source,
  });
}

function commit(next: ResizableSize): boolean {
  if (!sizeState.set(next)) return false;
  emit("update:size", next);
  return true;
}

function begin(edge: ResizablePhysicalEdge, source: ResizableSource, event: Event | null): void {
  if (disabledState.value) return;
  const active = { edge, initialSize: readSize(), source };
  session.value = active;
  emit("resize-start", eventFor(active, event));
}

function update(
  target: ResizableSize,
  event: Event | null,
  driver: "height" | "width" = "width",
): void {
  const active = readSession();
  if (active === null) return;
  const next = constrainResizableSize(target, constraints.value, driver);
  if (!commit(next)) return;
  emit("resize", Object.freeze({ ...eventFor(active, event), size: next }));
}

function end(event: Event | null): void {
  const active = readSession();
  if (active === null) return;
  session.value = null;
  emit("resize-end", eventFor(active, event));
}

function setSize(next: ResizableSize): boolean {
  return commit(constrainResizableSize(next, constraints.value));
}

resizableContext.provide({
  begin,
  constraints,
  dir: dirState,
  disabled: disabledState,
  end,
  id: rootId,
  initialSize: () => session.value?.initialSize ?? null,
  largeStep: largeStepState,
  size: currentSize,
  state,
  step: stepState,
  update,
});

type ResizableRootSetupExpose = Omit<
  ResizableRootExpose,
  "disabled" | "element" | "size" | "state"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly size: ComputedRef<ResizableSize>;
  readonly state: ComputedRef<ResizableState>;
};

const exposed = {
  disabled: disabledState,
  element,
  setSize,
  size: currentSize,
  state,
} satisfies ResizableRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="rootId"
    ref="element"
    :dir
    :style
    data-vize-ui="resizable-root"
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
