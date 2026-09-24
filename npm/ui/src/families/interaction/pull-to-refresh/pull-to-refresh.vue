<script setup lang="ts">
import { computed, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { pullToRefreshContext } from "./pull-to-refresh-context.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type {
  PullToRefreshExpose,
  PullToRefreshHandler,
  PullToRefreshSlotState,
  PullToRefreshSource,
  PullToRefreshState,
} from "./pull-to-refresh-types.ts";

const {
  refreshAction = undefined,
  threshold = 64,
  maxDistance = 128,
  resistance = 0.5,
  allowMouse = false,
  disabled = false,
  id = undefined,
} = defineProps<{
  /**
   * Async refresh handler; return a promise to keep the `refreshing` state until it settles.
   *
   * @default undefined
   */
  readonly refreshAction?: PullToRefreshHandler;

  /**
   * Pull distance (after resistance) that arms a refresh on release, in CSS px.
   *
   * @default 64
   */
  readonly threshold?: number;

  /**
   * Maximum published pull distance, in CSS px.
   *
   * @default 128
   */
  readonly maxDistance?: number;

  /**
   * Fraction of finger travel applied to the pull distance (0–1).
   *
   * @default 0.5
   */
  readonly resistance?: number;

  /**
   * Also accept mouse and pen drags (touch is always accepted).
   *
   * @default false
   */
  readonly allowMouse?: boolean;

  /**
   * Ignore gestures and triggers.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Id of the scroll container, referenced by PullToRefreshTrigger.
   *
   * @default undefined
   */
  readonly id?: string | null;
}>();

const emit = defineEmits<{
  /** Fired when a refresh starts, with its source. */
  refresh: [source: PullToRefreshSource];

  /** Fired when a refresh settles; `error` is the rejection reason, if any. */
  settle: [source: PullToRefreshSource, error: unknown];
}>();

defineSlots<{
  /** Indicator and scrollable content with the pull state. */
  default(props: PullToRefreshSlotState): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const rootId = useDeterministicId({ id: () => id, hint: "pull-to-refresh" });
const limit = computed(() => (Number.isFinite(threshold) && threshold > 0 ? threshold : 64));
const ceiling = computed(() =>
  Math.max(limit.value, Number.isFinite(maxDistance) ? maxDistance : 128),
);
const factor = computed(() =>
  Number.isFinite(resistance) && resistance > 0 ? Math.min(resistance, 1) : 0.5,
);
const pull = shallowRef(0);
const gesture = shallowRef(false);
const refreshing = shallowRef(false);
const reducedMotion = shallowRef(false);
let startY: number | undefined;
let pointerId: number | undefined;

const distance = computed(() => (refreshing.value ? limit.value : pull.value));
const progress = computed(() => Math.min(distance.value / limit.value, 1));
const state = computed<PullToRefreshState>(() => {
  if (refreshing.value) return "refreshing";
  if (!gesture.value || pull.value === 0) return "idle";
  return pull.value >= limit.value ? "armed" : "pulling";
});
const style = computed(() => ({
  "--vize-pull-distance": `${Math.round(distance.value)}px`,
  "--vize-pull-progress": String(Math.round(progress.value * 1000) / 1000),
}));

async function refresh(source: PullToRefreshSource): Promise<void> {
  if (refreshing.value || disabled) return;
  refreshing.value = true;
  emit("refresh", source);
  let failure: unknown;
  try {
    await refreshAction?.(source);
  } catch (error) {
    failure = error;
  } finally {
    refreshing.value = false;
    pull.value = 0;
    emit("settle", source, failure);
  }
}

function atTop(): boolean {
  return (root.value?.scrollTop ?? 0) <= 0;
}

function begin(y: number): void {
  if (disabled || refreshing.value || !atTop()) return;
  startY = y;
  gesture.value = true;
  pull.value = 0;
}

/** Returns whether the move was consumed as a pull (and native scrolling should be prevented). */
function update(y: number): boolean {
  if (startY === undefined) return false;
  const travel = y - startY;
  if (travel <= 0) {
    // Upward movement is a normal scroll; stop tracking once content scrolls.
    pull.value = 0;
    if (!atTop()) end(false);
    return false;
  }
  pull.value = Math.min(travel * factor.value, ceiling.value);
  return true;
}

function end(commit: boolean): void {
  const armed = commit && pull.value >= limit.value;
  startY = undefined;
  pointerId = undefined;
  gesture.value = false;
  if (armed) void refresh("gesture");
  else pull.value = 0;
}

function onTouchStart(event: TouchEvent): void {
  const touch = event.touches[0];
  if (event.touches.length === 1 && touch !== undefined) begin(touch.clientY);
}

function onTouchMove(event: TouchEvent): void {
  const touch = event.touches[0];
  if (touch !== undefined && update(touch.clientY) && event.cancelable) event.preventDefault();
}

function onPointerDown(event: PointerEvent): void {
  if (!allowMouse || event.pointerType === "touch" || event.button !== 0) return;
  pointerId = event.pointerId;
  begin(event.clientY);
}

function onPointerMove(event: PointerEvent): void {
  if (event.pointerId === pointerId && update(event.clientY)) event.preventDefault();
}

function onPointerEnd(event: PointerEvent): void {
  if (event.pointerId === pointerId) end(event.type === "pointerup");
}

// Touch moves must be non-passive to stop the browser's own overscroll while pulling.
watch(
  root,
  (element, _previous, onCleanup) => {
    if (element === null) return;
    const touchEnd = () => end(true);
    const touchCancel = () => end(false);
    element.addEventListener("touchstart", onTouchStart, { passive: true });
    element.addEventListener("touchmove", onTouchMove, { passive: false });
    element.addEventListener("touchend", touchEnd);
    element.addEventListener("touchcancel", touchCancel);
    const media = element.ownerDocument.defaultView?.matchMedia?.(
      "(prefers-reduced-motion: reduce)",
    );
    const onMotion = () => {
      reducedMotion.value = media?.matches === true;
    };
    onMotion();
    media?.addEventListener?.("change", onMotion);
    onCleanup(() => {
      element.removeEventListener("touchstart", onTouchStart);
      element.removeEventListener("touchmove", onTouchMove);
      element.removeEventListener("touchend", touchEnd);
      element.removeEventListener("touchcancel", touchCancel);
      media?.removeEventListener?.("change", onMotion);
    });
  },
  // Sync: listeners attach as soon as the element exists, even before the post-flush queue runs.
  { flush: "sync", immediate: true },
);

onScopeDispose(() => {
  startY = undefined;
});

pullToRefreshContext.provide({
  refreshing: computed(() => refreshing.value),
  disabled: computed(() => disabled),
  rootId,
  trigger: () => {
    void refresh("trigger");
  },
});

const slotState = computed<PullToRefreshSlotState>(() => ({
  state: state.value,
  distance: distance.value,
  progress: progress.value,
  refreshing: refreshing.value,
  reducedMotion: reducedMotion.value,
}));

type PullToRefreshSetupExpose = Omit<PullToRefreshExpose, keyof PullToRefreshSlotState | "root"> & {
  readonly [Key in keyof PullToRefreshSlotState]: ComputedRef<PullToRefreshSlotState[Key]>;
} & { readonly root: typeof root };

function field<Key extends keyof PullToRefreshSlotState>(
  key: Key,
): ComputedRef<PullToRefreshSlotState[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  state: field("state"),
  distance: field("distance"),
  progress: field("progress"),
  refreshing: field("refreshing"),
  reducedMotion: field("reducedMotion"),
  root,
  refresh: async () => {
    await refresh("api");
  },
} satisfies PullToRefreshSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="rootId"
    ref="root"
    :aria-busy="refreshing ? 'true' : undefined"
    part="root"
    data-vize-ui="pull-to-refresh"
    :data-state="state"
    :data-reduced-motion="reducedMotion ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :style
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerEnd"
    @pointercancel="onPointerEnd"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Translate an indicator with --vize-pull-distance; make the root the scroll container. */
</style>
