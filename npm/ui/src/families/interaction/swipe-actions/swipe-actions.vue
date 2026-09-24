<script setup lang="ts">
import { computed, nextTick, onScopeDispose, shallowRef } from "vue";
import type { ComputedRef } from "vue";

import { swipeActionsContext } from "./swipe-actions-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import type {
  SwipeActionsExpose,
  SwipeActionsOpen,
  SwipeActionsSide,
  SwipeActionsSlotState,
  SwipeActionsState,
} from "./swipe-actions-types.ts";

const {
  as = "div",
  open = undefined,
  defaultOpen = null,
  fullSwipe = true,
  fullSwipeThreshold = 0.6,
  dir = "ltr",
  disabled = false,
} = defineProps<{
  /**
   * Element or component to render, for example `"li"`.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Controlled open tray (`v-model:open`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: SwipeActionsOpen;

  /**
   * Initial uncontrolled open tray.
   *
   * @default null
   */
  readonly defaultOpen?: SwipeActionsOpen;

  /**
   * Swiping past `fullSwipeThreshold` of the item width emits `fullSwipe` (for example delete).
   *
   * @default true
   */
  readonly fullSwipe?: boolean;

  /**
   * Fraction of the item width that commits a full swipe.
   *
   * @default 0.6
   */
  readonly fullSwipeThreshold?: number;

  /**
   * Text direction; RTL mirrors leading/trailing and arrow keys.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";

  /**
   * Ignore swipes and keyboard reveals.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the open tray requests a change (`v-model:open`). */
  "update:open": [open: SwipeActionsOpen];

  /** Fired when a release passes the full-swipe threshold, with the committed side. */
  fullSwipe: [side: SwipeActionsSide];
}>();

defineSlots<{
  /** Renders the trays and the swipeable content with the swipe state. */
  default(props: SwipeActionsSlotState): unknown;
}>();

const contentId = useDeterministicId({ hint: "swipe-content" });
const state = useControllableState<SwipeActionsOpen>({
  value: () => open,
  defaultValue: () => defaultOpen,
  onChange: (value) => emit("update:open", value),
});
const trays = new Map<SwipeActionsSide, HTMLElement>();
let content: HTMLElement | null = null;
const dragOffset = shallowRef<number | null>(null);
const traySizes = shallowRef<Readonly<Record<SwipeActionsSide, number>>>({
  leading: 0,
  trailing: 0,
});
// Positive offsets move content toward inline-end, revealing the leading tray.
const sign = computed(() => (dir === "rtl" ? -1 : 1));
const restingOffset = computed(() => {
  if (state.value.value === "leading") return traySizes.value.leading * sign.value;
  if (state.value.value === "trailing") return -traySizes.value.trailing * sign.value;
  return 0;
});
const offset = computed(() => dragOffset.value ?? restingOffset.value);
const sideOf = (value: number): SwipeActionsSide | null => {
  const logical = value * sign.value;
  if (logical > 0) return "leading";
  return logical < 0 ? "trailing" : null;
};
const fullSwipeSide = computed<SwipeActionsSide | null>(() => {
  if (!fullSwipe || dragOffset.value === null || content === null) return null;
  const side = sideOf(dragOffset.value);
  const width = content.offsetWidth;
  if (side === null || width <= 0 || traySizes.value[side] === 0) return null;
  return Math.abs(dragOffset.value) >= width * fullSwipeThreshold ? side : null;
});
const dataState = computed<SwipeActionsState>(() => {
  if (dragOffset.value !== null) return "dragging";
  return state.value.value === null ? "closed" : "open";
});

function measure(): void {
  traySizes.value = {
    leading: trays.get("leading")?.offsetWidth ?? 0,
    trailing: trays.get("trailing")?.offsetWidth ?? 0,
  };
}

function focusTray(side: SwipeActionsSide): void {
  void nextTick(() => {
    const target = trays.get(side)?.querySelector<HTMLElement>("button, [href], [tabindex]");
    target?.focus();
  });
}

function openSide(side: SwipeActionsSide, focus = false): boolean {
  if (disabled || !trays.has(side)) return false;
  measure();
  const changed = state.set(side);
  if (focus) focusTray(side);
  return changed;
}

function close(): boolean {
  const focusInside = [...trays.values()].some((tray) =>
    tray.contains(tray.ownerDocument.activeElement),
  );
  const changed = state.set(null);
  if (focusInside) void nextTick(() => content?.focus());
  return changed;
}

/** Gesture math needs point-in-time numbers, not live refs. */
function snapshotOffset(): number {
  return offset.value;
}

function releaseSnapshot(): {
  readonly released: number;
  readonly committed: SwipeActionsSide | null;
} {
  return { released: dragOffset.value ?? 0, committed: fullSwipeSide.value };
}

let stopDrag: (() => void) | undefined;

function onPointerdown(event: PointerEvent): void {
  if (disabled || event.button !== 0 || !(event.currentTarget instanceof HTMLElement)) return;
  const element = event.currentTarget;
  const startX = event.clientX;
  const startY = event.clientY;
  const base = snapshotOffset();
  const pointerId = event.pointerId;
  let axis: "none" | "x" | "y" = "none";
  measure();
  stopDrag?.();

  const onMove = (move: PointerEvent) => {
    if (move.pointerId !== pointerId) return;
    const dx = move.clientX - startX;
    const dy = move.clientY - startY;
    if (axis === "none") {
      if (Math.abs(dx) < 6 && Math.abs(dy) < 6) return;
      axis = Math.abs(dx) > Math.abs(dy) ? "x" : "y";
      if (axis === "x") {
        try {
          element.setPointerCapture(pointerId);
        } catch {
          // Synthetic pointers cannot be captured.
        }
      }
    }
    if (axis !== "x") return;
    move.preventDefault();
    const next = base + dx;
    const side = sideOf(next);
    // Sides without a tray do not move; with full swipe enabled a tray may overshoot to the row width.
    if (side === null || traySizes.value[side] === 0) {
      dragOffset.value = 0;
      return;
    }
    const tray = traySizes.value[side];
    const limit = fullSwipe ? Math.max(content?.offsetWidth ?? 0, tray) : tray;
    dragOffset.value = Math.sign(next) * Math.min(Math.abs(next), limit);
  };
  const onEnd = (end: PointerEvent) => {
    if (end.pointerId !== pointerId) return;
    stopDrag?.();
    // Cancels (and non-horizontal gestures) restore the previous resting position.
    if (axis !== "x" || dragOffset.value === null || end.type === "pointercancel") {
      dragOffset.value = null;
      return;
    }
    const { released, committed } = releaseSnapshot();
    dragOffset.value = null;
    const side = sideOf(released);
    if (committed !== null) {
      state.set(null);
      emit("fullSwipe", committed);
    } else if (side !== null && Math.abs(released) >= traySizes.value[side] / 2) {
      state.set(side);
    } else {
      state.set(null);
    }
    // The click that ends a drag must not activate the content.
    element.addEventListener("click", (click) => click.preventDefault(), {
      capture: true,
      once: true,
    });
  };
  element.addEventListener("pointermove", onMove);
  element.addEventListener("pointerup", onEnd);
  element.addEventListener("pointercancel", onEnd);
  stopDrag = () => {
    element.removeEventListener("pointermove", onMove);
    element.removeEventListener("pointerup", onEnd);
    element.removeEventListener("pointercancel", onEnd);
    stopDrag = undefined;
  };
}

function onContentKeydown(event: KeyboardEvent): void {
  if (disabled || event.target !== event.currentTarget) return;
  const toward = event.key === "ArrowLeft" ? -1 : event.key === "ArrowRight" ? 1 : 0;
  if (event.key === "Escape" && state.value.value !== null) {
    event.preventDefault();
    close();
    return;
  }
  if (toward === 0) return;
  // Moving content toward inline-end reveals the leading tray (mirrored in RTL).
  const side: SwipeActionsSide = toward * sign.value > 0 ? "leading" : "trailing";
  event.preventDefault();
  if (state.value.value !== null && state.value.value !== side) close();
  else openSide(side, true);
}

onScopeDispose(() => stopDrag?.());

swipeActionsContext.provide({
  open: computed(() => state.value.value),
  offset,
  disabled: computed(() => disabled),
  contentId,
  registerTray: (side: SwipeActionsSide, element: HTMLElement | null) => {
    if (element === null) trays.delete(side);
    else trays.set(side, element);
  },
  registerContent: (element: HTMLElement | null) => {
    content = element;
  },
  onPointerdown,
  onContentKeydown,
  openSide,
  close,
});

const slotState = computed<SwipeActionsSlotState>(() => ({
  open: state.value.value,
  offset: offset.value,
  dragging: dragOffset.value !== null,
  fullSwipeSide: fullSwipeSide.value,
  state: dataState.value,
}));
const style = computed(() => ({ "--vize-swipe-offset": `${Math.round(offset.value)}px` }));

type SwipeActionsSetupExpose = Omit<SwipeActionsExpose, keyof SwipeActionsSlotState> & {
  readonly [Key in keyof SwipeActionsSlotState]: ComputedRef<SwipeActionsSlotState[Key]>;
};

function field<Key extends keyof SwipeActionsSlotState>(
  key: Key,
): ComputedRef<SwipeActionsSlotState[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  open: field("open"),
  offset: field("offset"),
  dragging: field("dragging"),
  fullSwipeSide: field("fullSwipeSide"),
  state: field("state"),
  openSide,
  close,
} satisfies SwipeActionsSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    part="root"
    data-vize-ui="swipe-actions"
    :data-state="dataState"
    :data-open="state.value.value ?? undefined"
    :data-full-swipe="fullSwipeSide ?? undefined"
    :data-dir="dir"
    :style
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Translate the content with --vize-swipe-offset; position trays behind it. */
</style>
