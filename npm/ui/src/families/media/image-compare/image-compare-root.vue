<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { imageCompareContext } from "./image-compare-context.ts";
import type { ImageCompareContextValue } from "./image-compare-context.ts";
import type {
  ImageCompareChangeSource,
  ImageCompareDirection,
  ImageCompareMessages,
  ImageCompareMode,
  ImageCompareOrientation,
  ImageCompareRootExpose,
  ImageCompareSlotState,
  ImageCompareState,
} from "./image-compare-types.ts";
import {
  imageComparePositionFromPoint,
  normalizeImageComparePosition,
} from "./image-compare-value.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = 50,
  orientation = "horizontal",
  dir = "ltr",
  step = 1,
  pageStep = 10,
  mode = "drag",
  disabled = false,
  messages = undefined,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled divider position in percent (`0`–`100`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: number;

  /**
   * Initial uncontrolled divider position in percent.
   *
   * @default 50
   */
  readonly defaultValue?: number;

  /**
   * Divider axis. `horizontal` moves a vertical divider left and right.
   *
   * @default "horizontal"
   */
  readonly orientation?: ImageCompareOrientation;

  /**
   * Reading direction; RTL measures horizontal positions from the right edge.
   *
   * @default "ltr"
   */
  readonly dir?: ImageCompareDirection;

  /**
   * Arrow-key increment and snapping step in percent. `0` disables snapping.
   *
   * @default 1
   */
  readonly step?: number;

  /**
   * PageUp/PageDown increment in percent.
   *
   * @default 10
   */
  readonly pageStep?: number;

  /**
   * Pointer behavior: press-and-drag, or follow the hovering pointer.
   *
   * @default "drag"
   */
  readonly mode?: ImageCompareMode;

  /**
   * Suppress pointer and keyboard changes and remove the handle from the tab order.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Localized handle name and value text. Omitted entries use English defaults.
   *
   * @default undefined
   */
  readonly messages?: ImageCompareMessages;
}>();

const emit = defineEmits<{
  /** Fired with the requested position whenever it changes. */
  "update:modelValue": [position: number];

  /** Fired after every distinct position request with its source. */
  change: [position: number, previous: number, source: ImageCompareChangeSource];

  /** Fired when a pointer drag or keyboard interaction finishes. */
  commit: [position: number];
}>();

defineSlots<{
  /** Before/after images, handle, and labels. Receives the divider state. */
  default(props: ImageCompareSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const handleElement = shallowRef<HTMLElement | null>(null);
const baseId = useDeterministicId({ id: () => id, hint: "image-compare" });
const handleId = computed(() => deriveDeterministicId(baseId.value, "handle"));
const positionState = useControllableState<number>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
});
const position = computed(() => normalizeImageComparePosition(positionState.value.value, 0));
const dragging = shallowRef(false);
const disabledState = computed(() => disabled);
const orientationState = computed(() => orientation);
const dirState = computed(() => dir);
const state = computed<ImageCompareState>(() => {
  if (disabledState.value) return "disabled";
  return dragging.value ? "dragging" : "idle";
});
const slotState = computed<ImageCompareSlotState>(() => ({
  orientation: orientationState.value,
  position: position.value,
  state: state.value,
}));
const rootStyle = computed(() => ({
  "--vize-ui-image-compare-position": `${position.value}%`,
}));
let activePointer: number | null = null;

function currentPosition(): number {
  return position.value;
}

function setPosition(
  requested: number,
  source: ImageCompareChangeSource,
  _event: Event | null = null,
): boolean {
  if (disabledState.value && source !== "api") return false;
  const next = normalizeImageComparePosition(requested, step);
  const previous = currentPosition();
  if (next === previous) return false;
  positionState.set(next);
  emit("update:modelValue", next);
  emit("change", next, previous, source);
  return true;
}

function positionAt(event: PointerEvent): number | null {
  if (element.value === null) return null;
  const rect = element.value.getBoundingClientRect();
  return imageComparePositionFromPoint(rect, event.clientX, event.clientY, {
    dir: dirState.value,
    orientation: orientationState.value,
  });
}

function onPointerDown(event: PointerEvent): void {
  if (disabledState.value || mode !== "drag" || event.button !== 0) return;
  const next = positionAt(event);
  if (next === null) return;
  activePointer = event.pointerId;
  dragging.value = true;
  try {
    element.value?.setPointerCapture(event.pointerId);
  } catch {
    // Capture is best-effort; move events still arrive while the pointer stays inside.
  }
  event.preventDefault();
  handleElement.value?.focus({ preventScroll: true });
  setPosition(next, "pointer", event);
}

function onPointerMove(event: PointerEvent): void {
  if (disabledState.value) return;
  const tracking =
    (mode === "drag" && activePointer === event.pointerId) ||
    (mode === "hover" && event.pointerType !== "touch");
  if (!tracking) return;
  const next = positionAt(event);
  if (next !== null) setPosition(next, "pointer", event);
}

function onPointerEnd(event: PointerEvent): void {
  if (activePointer !== event.pointerId) return;
  activePointer = null;
  dragging.value = false;
  emit("commit", position.value);
}

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  element.value?.addEventListener("pointerdown", onPointerDown);
  element.value?.addEventListener("pointermove", onPointerMove);
  element.value?.addEventListener("pointerup", onPointerEnd);
  element.value?.addEventListener("pointercancel", onPointerEnd);
  element.value?.addEventListener("lostpointercapture", onPointerEnd);
});

onBeforeUnmount(() => {
  element.value?.removeEventListener("pointerdown", onPointerDown);
  element.value?.removeEventListener("pointermove", onPointerMove);
  element.value?.removeEventListener("pointerup", onPointerEnd);
  element.value?.removeEventListener("pointercancel", onPointerEnd);
  element.value?.removeEventListener("lostpointercapture", onPointerEnd);
});

imageCompareContext.provide({
  dir: dirState,
  disabled: disabledState,
  handleElement,
  handleId,
  messages: computed(() => messages ?? {}),
  orientation: orientationState,
  pageStep: computed(() => pageStep),
  position,
  setPosition: (next, source, event) => {
    const changed = setPosition(next, source, event);
    if (source === "keyboard") emit("commit", position.value);
    return changed;
  },
  slotState,
  state,
  step: computed(() => step),
} satisfies ImageCompareContextValue);

type ImageCompareRootSetupExpose = Omit<
  ImageCompareRootExpose,
  keyof ImageCompareSlotState | "element"
> & {
  readonly element: typeof element;
  readonly orientation: ComputedRef<ImageCompareOrientation>;
  readonly position: ComputedRef<number>;
  readonly state: ComputedRef<ImageCompareState>;
};

const exposed = {
  element,
  focus: (options?: FocusOptions) => handleElement.value?.focus(options),
  orientation: orientationState,
  position,
  setPosition: (next: number) => setPosition(next, "api"),
  state,
} satisfies ImageCompareRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    :dir="dirState"
    data-vize-ui="image-compare-root"
    part="root"
    :data-state="state"
    :data-orientation="orientationState"
    :data-mode="mode"
    :data-disabled="disabledState ? 'true' : undefined"
    :style="rootStyle"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Consumers stack the images and clip the "before" side, e.g.
   [data-vize-ui="image-compare-before"] {
     clip-path: inset(0 calc(100% - var(--vize-ui-image-compare-position)) 0 0);
   } */
</style>
