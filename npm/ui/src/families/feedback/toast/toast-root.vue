<script setup lang="ts" generic="Data = unknown">
import { computed, nextTick, onMounted, shallowRef, useTemplateRef, watch } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { usePresence } from "../../overlays/presence/presence-runtime.ts";
import { toastProviderContext, toastRootContext, toastViewportContext } from "./toast-context.ts";
import type {
  ToastDismissReason,
  ToastRecord,
  ToastRootExpose,
  ToastSlotState,
  ToastSwipeState,
} from "./toast-types.ts";
import { useToast } from "./use-toast.ts";

const { toast } = defineProps<{
  /** Toast snapshot to render, usually from the ToastViewport slot. @default required */
  readonly toast: ToastRecord<Data>;
}>();

defineSlots<{
  /** Toast parts. Receives the toast snapshot, open state, and swipe phase. */
  default?(props: ToastSlotState<Data>): unknown;
}>();

interface SwipeOffset {
  readonly x: number;
  readonly y: number;
}

const provider = toastProviderContext.use();
const viewport = toastViewportContext.useOptional();
const store = useToast<Data>();
const element = useTemplateRef<HTMLLIElement>("element");
const baseId = useDeterministicId({ hint: "toast" });
const titleId = computed(() => deriveDeterministicId(baseId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(baseId.value, "description"));
const record = computed(() => toast);
const swipe = shallowRef<ToastSwipeState | null>(null);
const offset = shallowRef<SwipeOffset>({ x: 0, y: 0 });
const presence = usePresence({
  present: () => toast.open,
  onExitComplete: () => {
    store.remove(toast.id);
  },
});
const slotState = computed<ToastSlotState<Data>>(() => ({
  state: toast.state,
  swipe: swipe.value,
  toast,
}));
const swipeStyle = computed(() => {
  if (swipe.value === null) return undefined;
  const phase = swipe.value === "end" ? "end" : "move";
  return {
    [`--vize-toast-swipe-${phase}-x`]: `${offset.value.x}px`,
    [`--vize-toast-swipe-${phase}-y`]: `${offset.value.y}px`,
  };
});
let start: { readonly x: number; readonly y: number; readonly pointerId: number } | null = null;

function dismiss(reason: ToastDismissReason = "api"): boolean {
  if (!toast.dismissible && reason !== "api" && reason !== "action") return false;
  return store.dismiss(toast.id, reason);
}

toastRootContext.provide({
  toast: record,
  titleId,
  descriptionId,
  swipe: computed(() => swipe.value),
  dismiss,
});

function hasRunningMotion(target: HTMLElement): boolean {
  const view = target.ownerDocument.defaultView;
  if (!view) return false;
  const style = view.getComputedStyle(target);
  const animated = style.animationName !== "" && style.animationName !== "none";
  const transitioned = style.transitionDuration
    .split(",")
    .some((value) => Number.parseFloat(value) > 0);
  return animated || transitioned;
}

function finishExitWithoutMotion(): void {
  if (presence.status.value !== "exiting") return;
  if (!element.value || !hasRunningMotion(element.value)) presence.completeAnimation();
}

watch(
  () => presence.status.value,
  (status) => {
    if (status === "exiting") void nextTick(finishExitWithoutMotion);
  },
  { flush: "post" },
);

onMounted(() => {
  if (!toast.open) store.remove(toast.id);
});

function axisDistance(dx: number, dy: number): number {
  switch (provider.swipeDirection.value) {
    case "left":
      return -dx;
    case "up":
      return -dy;
    case "down":
      return dy;
    default:
      return dx;
  }
}

function offsetFor(distance: number): SwipeOffset {
  switch (provider.swipeDirection.value) {
    case "left":
      return { x: -distance, y: 0 };
    case "up":
      return { x: 0, y: -distance };
    case "down":
      return { x: 0, y: distance };
    default:
      return { x: distance, y: 0 };
  }
}

function onPointerdown(event: PointerEvent): void {
  if (event.button !== 0 || !toast.dismissible || !toast.open) return;
  start = { x: event.clientX, y: event.clientY, pointerId: event.pointerId };
  swipe.value = null;
  offset.value = { x: 0, y: 0 };
}

function onPointermove(event: PointerEvent): void {
  if (!start || event.pointerId !== start.pointerId) return;
  const distance = Math.max(0, axisDistance(event.clientX - start.x, event.clientY - start.y));
  if (distance === 0 && swipe.value === null) return;
  if (swipe.value === null) {
    swipe.value = "start";
    store.pause("swipe");
    if (element.value && typeof element.value.setPointerCapture === "function") {
      try {
        element.value.setPointerCapture(event.pointerId);
      } catch {
        // Pointer capture is best-effort; synthetic pointers cannot be captured.
      }
    }
  } else {
    swipe.value = "move";
  }
  offset.value = offsetFor(distance);
}

function finishSwipe(event: PointerEvent, cancelled: boolean): void {
  if (!start || event.pointerId !== start.pointerId) return;
  const distance = Math.max(0, axisDistance(event.clientX - start.x, event.clientY - start.y));
  start = null;
  if (swipe.value === null) return;
  store.resume("swipe");
  if (!cancelled && distance >= provider.swipeThreshold.value) {
    offset.value = offsetFor(distance);
    swipe.value = "end";
    dismiss("swipe");
    return;
  }
  swipe.value = "cancel";
  offset.value = { x: 0, y: 0 };
}

function onPointerup(event: PointerEvent): void {
  finishSwipe(event, false);
}

function onPointercancel(event: PointerEvent): void {
  finishSwipe(event, true);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== "Escape" || event.defaultPrevented || event.isComposing) return;
  if (!dismiss("escape")) return;
  event.preventDefault();
  viewport?.focus();
}

const interactionProps = {
  ...presence.presenceProps,
  onKeydown,
  onPointercancel,
  onPointerdown,
  onPointermove,
  onPointerup,
} as const;

type ToastRootSetupExpose = Omit<ToastRootExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, dismiss } satisfies ToastRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <li
    ref="element"
    role="status"
    aria-live="off"
    aria-atomic="true"
    tabindex="0"
    data-vize-ui="toast"
    part="root"
    :data-state="toast.state"
    :data-type="toast.type"
    :data-priority="toast.priority"
    :data-presence="presence.status.value"
    :data-swipe="swipe ?? undefined"
    :data-swipe-direction="provider.swipeDirection.value"
    :style="swipeStyle"
    v-bind="interactionProps"
  >
    <slot v-bind="slotState" />
  </li>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
