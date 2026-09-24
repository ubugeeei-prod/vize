<script setup lang="ts">
import { computed, onBeforeUnmount, useTemplateRef, watch } from "vue";

import DialogContent from "../../overlays/dialog/dialog-content.vue";
import DialogPortal from "../../overlays/dialog/dialog-portal.vue";
import { lightboxContext } from "./lightbox-context.ts";
import { classifyLightboxSwipe } from "./lightbox-state.ts";
import type { LightboxContentExpose, LightboxPartSlotState } from "./lightbox-types.ts";

const {
  to = "body",
  portalDisabled = false,
  swipeThreshold = 50,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * CSS selector or element the viewer layer is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render in place instead of teleporting.
   *
   * @default false
   */
  readonly portalDisabled?: boolean;

  /**
   * Minimum swipe travel in CSS pixels along the dominant axis.
   *
   * @default 50
   */
  readonly swipeThreshold?: number;

  /**
   * Accessible dialog name. Defaults to the `dialog` message unless labelled by ids.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the dialog, e.g. a visible caption.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** Viewer content: item, controls, counter, thumbnails. Receives the viewer state. */
  default(props: LightboxPartSlotState): unknown;
}>();

const context = lightboxContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const label = computed<string | undefined>(() =>
  ariaLabelledby === undefined ? (ariaLabel ?? context.messages.value.dialog) : undefined,
);
let swipe: { readonly pointerId: number; readonly x: number; readonly y: number } | null = null;

function isEditable(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

function onKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return;
  if (isEditable(event.target)) return;
  const rtl = context.dir.value === "rtl";
  let handled = true;
  if (event.key === (rtl ? "ArrowLeft" : "ArrowRight")) context.step(1, "keyboard");
  else if (event.key === (rtl ? "ArrowRight" : "ArrowLeft")) context.step(-1, "keyboard");
  else if (event.key === "Home") context.goTo(0, "keyboard");
  else if (event.key === "End") context.goTo(context.count.value - 1, "keyboard");
  else handled = false;
  if (handled) event.preventDefault();
}

function onPointerDown(event: PointerEvent): void {
  if (event.pointerType === "mouse" || !event.isPrimary) return;
  swipe = { pointerId: event.pointerId, x: event.clientX, y: event.clientY };
}

function onPointerUp(event: PointerEvent): void {
  if (swipe === null || event.pointerId !== swipe.pointerId) return;
  const outcome = classifyLightboxSwipe({
    closeOnSwipeDown: context.closeOnSwipeDown.value,
    deltaX: event.clientX - swipe.x,
    deltaY: event.clientY - swipe.y,
    dir: context.dir.value,
    threshold: Math.max(1, swipeThreshold),
  });
  swipe = null;
  if (outcome === "next") context.step(1, "swipe");
  else if (outcome === "previous") context.step(-1, "swipe");
  else if (outcome === "close") context.close();
}

function onPointerCancel(): void {
  swipe = null;
}

function attach(target: HTMLDivElement): void {
  target.addEventListener("keydown", onKeydown);
  target.addEventListener("pointerdown", onPointerDown);
  target.addEventListener("pointerup", onPointerUp);
  target.addEventListener("pointercancel", onPointerCancel);
}

function detach(target: HTMLDivElement): void {
  target.removeEventListener("keydown", onKeydown);
  target.removeEventListener("pointerdown", onPointerDown);
  target.removeEventListener("pointerup", onPointerUp);
  target.removeEventListener("pointercancel", onPointerCancel);
}

// The stage exists only while the dialog is open, so listeners follow the element.
watch(
  element,
  (next, previous) => {
    if (previous) detach(previous);
    if (next) attach(next);
  },
  { flush: "post" },
);

onBeforeUnmount(() => {
  if (element.value !== null) detach(element.value);
});

const exposed = { element } satisfies Omit<LightboxContentExpose, "element"> & {
  readonly element: typeof element;
};

defineExpose(exposed);
</script>

<template>
  <DialogPortal :to :disabled="portalDisabled">
    <DialogContent :aria-label="label" :aria-labelledby="ariaLabelledby ?? null">
      <div
        ref="element"
        :dir="context.dir.value"
        data-vize-ui="lightbox-content"
        part="content"
        :data-state="context.state.value"
        :data-index="context.index.value"
      >
        <slot v-bind="context.slotState.value" />
      </div>
    </DialogContent>
  </DialogPortal>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
