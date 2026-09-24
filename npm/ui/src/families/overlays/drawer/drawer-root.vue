<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { dialogContext } from "../dialog/dialog.ts";
import { drawerContext } from "./drawer-context.ts";
import {
  assertDrawerSnapPoints,
  drawerAxis,
  drawerDismissSign,
  drawerSnapOffset,
  drawerSnapPointEquals,
  resolveDrawerRelease,
  stepDrawerSnapPoint,
} from "./drawer-snap.ts";
import type {
  DrawerDragEndEvent,
  DrawerRootExpose,
  DrawerSide,
  DrawerSlotState,
  DrawerSnapPoint,
  DrawerState,
} from "./drawer-types.ts";

interface DragSession {
  readonly pointerId: number;
  readonly startCoordinate: number;
  readonly startOffset: number;
  lastCoordinate: number;
  lastTime: number;
  velocity: number;
  distance: number;
  started: boolean;
}

const dragSlop = 4;
const emptySnapPoints: readonly DrawerSnapPoint[] = Object.freeze([]);

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  modal = true,
  side = "bottom",
  snapPoints = undefined,
  activeSnapPoint = undefined,
  defaultActiveSnapPoint = undefined,
  dismissible = true,
  closeThreshold = 0.25,
  velocityThreshold = 0.5,
} = defineProps<{
  /**
   * Consumer-owned Drawer base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled open state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Show the native `<dialog>` with `showModal()` (top layer, inert page, `::backdrop`).
   *
   * @default true
   */
  readonly modal?: boolean;

  /**
   * Viewport edge the drawer is attached to and dragged toward to dismiss.
   *
   * @default "bottom"
   */
  readonly side?: DrawerSide;

  /**
   * Resting positions: fractions in `(0, 1]` of the drawer size or `"<number>px"` visible sizes.
   * `undefined` means a single fully open position.
   *
   * @default undefined
   */
  readonly snapPoints?: readonly DrawerSnapPoint[];

  /**
   * Controlled active snap point. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly activeSnapPoint?: DrawerSnapPoint | null;

  /**
   * Initial snap point for uncontrolled use. `undefined` selects the first snap point.
   *
   * @default undefined
   */
  readonly defaultActiveSnapPoint?: DrawerSnapPoint;

  /**
   * Whether Escape, backdrop presses, outside presses, and drag gestures may dismiss.
   *
   * @default true
   */
  readonly dismissible?: boolean;

  /**
   * Fraction of the size still visible at the lowest snap point that a drag must pass to dismiss.
   *
   * @default 0.25
   */
  readonly closeThreshold?: number;

  /**
   * Release velocity in px/ms toward the dismiss direction that flicks the drawer closed.
   *
   * @default 0.5
   */
  readonly velocityThreshold?: number;
}>();

const emit = defineEmits<{
  /** Fired when the Drawer requests a controlled open value. */
  "update:open": [value: boolean];

  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];

  /** Fired when the Drawer requests a controlled snap point. */
  "update:activeSnapPoint": [value: DrawerSnapPoint | null];

  /** Fired after any distinct snap point request. */
  "snap-point-change": [
    value: DrawerSnapPoint | null,
    previous: DrawerSnapPoint | null,
    nativeEvent: Event | null,
  ];

  /** Fired once a pointer has moved past the drag slop. */
  "drag-start": [nativeEvent: PointerEvent];

  /** Fired when a started drag gesture ends. */
  "drag-end": [event: DrawerDragEndEvent];
}>();

defineSlots<{
  /** Compound Drawer children. Receives the current open, snap, and drag state. */
  default(props: DrawerSlotState): unknown;
}>();

const snapPointList = computed<readonly DrawerSnapPoint[]>(() => snapPoints ?? emptySnapPoints);
watch(snapPointList, assertDrawerSnapPoints, { immediate: true });

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const snapState = useControllableState<DrawerSnapPoint | null>({
  value: () => activeSnapPoint,
  defaultValue: () => defaultActiveSnapPoint ?? snapPointList.value[0] ?? null,
  equals: drawerSnapPointEquals,
});
const isOpen = openState.value;
const modalState = computed(() => modal);
const sideState = computed(() => side);
const dismissibleState = computed(() => dismissible);
const state = computed<DrawerState>(() => (isOpen.value ? "open" : "closed"));
const activeSnap = computed<DrawerSnapPoint | null>(() => {
  const current = snapState.value.value;
  if (snapPointList.value.length === 0) return null;
  return snapPointList.value.some((point) => Object.is(point, current))
    ? current
    : (snapPointList.value[0] ?? null);
});
const baseId = useDeterministicId({ id: () => id, hint: "drawer" });
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const titleId = computed(() => deriveDeterministicId(baseId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(baseId.value, "description"));
const dragging = shallowRef(false);
const dragOffset = shallowRef(0);
const size = shallowRef(0);
const dialogElement = shallowRef<HTMLDialogElement | null>(null);
const snapOffset = computed(() => drawerSnapOffset(activeSnap.value, size.value));
const slotState = computed<DrawerSlotState>(() => ({
  activeSnapPoint: activeSnap.value,
  dragging: dragging.value,
  modal: modalState.value,
  open: isOpen.value,
  side: sideState.value,
  state: state.value,
}));
let session: DragSession | null = null;

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  const previous = isOpen.value;
  const changed = openState.set(value);
  if (changed) {
    emit("update:open", value);
    emit("open-change", value, previous, nativeEvent);
  }
  return changed;
}

function snapTo(snapPoint: DrawerSnapPoint, nativeEvent: Event | null = null): boolean {
  if (!snapPointList.value.some((point) => Object.is(point, snapPoint))) return false;
  return commitSnapPoint(snapPoint, activeSnap.value, nativeEvent);
}

function commitSnapPoint(
  snapPoint: DrawerSnapPoint,
  previous: DrawerSnapPoint | null,
  nativeEvent: Event | null,
): boolean {
  if (drawerSnapPointEquals(previous, snapPoint)) return false;
  snapState.set(snapPoint);
  emit("update:activeSnapPoint", snapPoint);
  emit("snap-point-change", snapPoint, previous, nativeEvent);
  return true;
}

function stepSnapPoint(direction: 1 | -1, wrap: boolean, nativeEvent: Event | null = null) {
  const next = stepDrawerSnapPoint(
    snapPointList.value,
    activeSnap.value,
    direction,
    size.value,
    wrap,
  );
  return next === null ? false : snapTo(next, nativeEvent);
}

function measure(): void {
  measureElement(dialogElement.value);
}

function measureElement(element: HTMLDialogElement | null): void {
  if (!element) return;
  const rect = element.getBoundingClientRect();
  size.value = drawerAxis(sideState.value) === "x" ? rect.width : rect.height;
}

function coordinate(event: PointerEvent): number {
  return drawerAxis(sideState.value) === "x" ? event.clientX : event.clientY;
}

function startDrag(event: PointerEvent): boolean {
  if (!isOpen.value || session !== null) return false;
  if (event.pointerType === "mouse" && event.button !== 0) return false;
  measure();
  const start = coordinate(event);
  session = {
    pointerId: event.pointerId,
    startCoordinate: start,
    startOffset: snapOffset.value,
    lastCoordinate: start,
    lastTime: event.timeStamp,
    velocity: 0,
    distance: 0,
    started: false,
  };
  return true;
}

function moveDrag(event: PointerEvent): void {
  const active = session;
  if (active === null || active.pointerId !== event.pointerId) return;
  const sign = drawerDismissSign(sideState.value);
  const current = coordinate(event);
  const distance = (current - active.startCoordinate) * sign;
  if (!active.started) {
    if (Math.abs(distance) < dragSlop) return;
    active.started = true;
    dragging.value = true;
    emit("drag-start", event);
  }
  const time = event.timeStamp;
  const elapsed = time - active.lastTime;
  if (elapsed > 0) active.velocity = ((current - active.lastCoordinate) * sign) / elapsed;
  active.lastCoordinate = current;
  active.lastTime = time;
  const total = active.startOffset + distance;
  // Rubber-band past the fully open position instead of detaching from the edge.
  active.distance = total < 0 ? -active.startOffset - Math.sqrt(-total) : distance;
  dragOffset.value = active.distance;
  if (event.cancelable) event.preventDefault();
}

function endDrag(event: PointerEvent, cancelled: boolean): boolean {
  const active = session;
  if (active === null || active.pointerId !== event.pointerId) return false;
  session = null;
  if (!active.started) return false;
  const distance = active.distance;
  dragging.value = false;
  dragOffset.value = 0;
  if (cancelled) {
    emit("drag-end", {
      distance,
      outcome: "cancel",
      snapPoint: activeSnap.value,
      velocity: active.velocity,
    });
    return true;
  }
  const result = resolveDrawerRelease({
    closeThreshold,
    dismissible: dismissibleState.value,
    distance,
    size: size.value,
    snapPoints: snapPointList.value,
    startOffset: active.startOffset,
    velocity: active.velocity,
    velocityThreshold,
  });
  if (result.outcome === "dismiss") {
    emit("drag-end", { distance, outcome: "dismiss", snapPoint: null, velocity: active.velocity });
    setOpen(false, event);
    return true;
  }
  if (result.snapPoint !== null) snapTo(result.snapPoint, event);
  emit("drag-end", {
    distance,
    outcome: "snap",
    snapPoint: activeSnap.value,
    velocity: active.velocity,
  });
  return true;
}

watch(isOpen, (value) => {
  if (value) return;
  session = null;
  dragging.value = false;
  dragOffset.value = 0;
});

const dialog = dialogContext.provide({
  id: baseId,
  contentId,
  titleId,
  descriptionId,
  open: isOpen,
  // Drawer motion is driven by its own drag/snap state and consumer CSS, so it
  // opts out of Dialog's optional-stylesheet exit animation: content unmounts
  // as soon as the drawer closes.
  exiting: shallowRef(false),
  modal: modalState,
  state,
  triggerElement: shallowRef<HTMLButtonElement | null>(null),
  overlayElement: shallowRef<HTMLElement | null>(null),
  contentElement: shallowRef<HTMLDivElement | null>(null),
  completeExit: () => {},
  setOpen,
  openDialog: (nativeEvent = null) => setOpen(true, nativeEvent),
  close: (nativeEvent = null) => setOpen(false, nativeEvent),
  toggle: (nativeEvent = null) => setOpen(!isOpen.value, nativeEvent),
});

drawerContext.provide({
  activeSnapPoint: activeSnap,
  dialogElement,
  dismissible: dismissibleState,
  dragOffset,
  dragging,
  endDrag,
  measure,
  moveDrag,
  side: sideState,
  size,
  snapOffset,
  snapPoints: snapPointList,
  snapTo,
  startDrag,
  stepSnapPoint,
});

type DrawerRootSetupExpose = Omit<
  DrawerRootExpose,
  keyof DrawerSlotState | "contentId" | "descriptionId" | "id" | "snapPoints" | "titleId"
> & {
  readonly activeSnapPoint: ComputedRef<DrawerSnapPoint | null>;
  readonly contentId: ComputedRef<string>;
  readonly descriptionId: ComputedRef<string>;
  readonly dragging: typeof dragging;
  readonly id: ComputedRef<string>;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly side: ComputedRef<DrawerSide>;
  readonly snapPoints: ComputedRef<readonly DrawerSnapPoint[]>;
  readonly state: ComputedRef<DrawerState>;
  readonly titleId: ComputedRef<string>;
};

const exposed = {
  activeSnapPoint: activeSnap,
  close: dialog.close,
  contentId,
  descriptionId,
  dragging,
  id: baseId,
  modal: modalState,
  open: isOpen,
  openDrawer: dialog.openDialog,
  setOpen,
  side: sideState,
  snapPoints: snapPointList,
  snapTo,
  state,
  titleId,
  toggle: dialog.toggle,
} satisfies DrawerRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="drawer-root"
    part="root"
    :data-state="state"
    :data-side="side"
    :data-modal="modal ? 'true' : 'false'"
    :data-dragging="dragging ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
