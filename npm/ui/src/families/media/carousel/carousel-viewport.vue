<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useVisibilityObserver } from "../../interaction/measure/measure-runtime.ts";
import { carouselContext } from "./carousel-context.ts";
import {
  nearestSlideIndex,
  scrollEdges,
  scrollPositionForSlide,
  slideIndexAfterDrag,
} from "./carousel-geometry.ts";
import type { CarouselRect, CarouselScrollMetrics } from "./carousel-geometry.ts";
import type { CarouselSlotState, CarouselViewportExpose } from "./carousel-types.ts";

const { settleDelay = 120, dragThreshold = 40 } = defineProps<{
  /**
   * Quiet period after the last `scroll` event before a user scroll selects the
   * nearest slide, for browsers without `scrollend`.
   *
   * @default 120
   */
  readonly settleDelay?: number;

  /**
   * Pixels a mouse drag must travel to page to the adjacent slide even when the
   * track would snap back to the starting slide.
   *
   * @default 40
   */
  readonly dragThreshold?: number;
}>();

defineSlots<{
  /** CarouselSlide children. Receives the carousel state. */
  default(props: CarouselSlotState): unknown;
}>();

const DRAG_START_DISTANCE = 3;

const context = carouselContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
let programmatic = false;
let settleTimer: ReturnType<typeof setTimeout> | undefined;
let suppressClick = false;
let drag: {
  readonly pointerId: number;
  readonly startCoordinate: number;
  readonly startScroll: number;
  readonly startIndex: number;
  readonly snapType: string;
  moved: boolean;
} | null = null;

function axis() {
  return { orientation: context.orientation.value, dir: context.dir.value };
}

function vertical(): boolean {
  return context.orientation.value === "vertical";
}

function rectOf(target: Element): CarouselRect {
  const rect = target.getBoundingClientRect();
  return { top: rect.top, right: rect.right, bottom: rect.bottom, left: rect.left };
}

function metricsOf(viewport: HTMLElement): CarouselScrollMetrics {
  return vertical()
    ? {
        position: viewport.scrollTop,
        clientSize: viewport.clientHeight,
        scrollSize: viewport.scrollHeight,
      }
    : {
        position: viewport.scrollLeft,
        clientSize: viewport.clientWidth,
        scrollSize: viewport.scrollWidth,
      };
}

function scrollPosition(viewport: HTMLElement): number {
  return vertical() ? viewport.scrollTop : viewport.scrollLeft;
}

function setScrollPosition(viewport: HTMLElement, position: number): void {
  if (vertical()) viewport.scrollTop = position;
  else viewport.scrollLeft = position;
}

function slideRects(): (CarouselRect | null)[] {
  const rects: (CarouselRect | null)[] = [];
  for (let index = 0; index < context.slideCount.value; index += 1) {
    const slide = context.slideElements.value.get(index);
    rects.push(slide === undefined ? null : rectOf(slide));
  }
  return rects;
}

function nearestIndex(viewport: HTMLElement): number {
  return nearestSlideIndex(rectOf(viewport), slideRects(), axis());
}

function scrollToIndex(index: number, smooth: boolean): void {
  const slide = context.slideElements.value.get(index);
  if (element.value === null || slide === undefined) return;
  const target = scrollPositionForSlide(
    metricsOf(element.value),
    rectOf(element.value),
    rectOf(slide),
    axis(),
  );
  if (Math.abs(target - scrollPosition(element.value)) < 1) return;
  programmatic = true;
  const behavior: ScrollBehavior = smooth && !context.reducedMotion.value ? "smooth" : "auto";
  if (typeof element.value.scrollTo === "function") {
    element.value.scrollTo(vertical() ? { top: target, behavior } : { left: target, behavior });
  } else {
    setScrollPosition(element.value, target);
  }
}

function settle(): void {
  if (settleTimer !== undefined) clearTimeout(settleTimer);
  settleTimer = undefined;
  if (element.value === null || drag !== null) return;
  context.setEdges(scrollEdges(metricsOf(element.value)));
  if (programmatic) {
    programmatic = false;
    return;
  }
  const nearest = nearestIndex(element.value);
  if (nearest >= 0) context.goTo(nearest, "scroll");
}

function onScroll(): void {
  if (element.value !== null) context.setEdges(scrollEdges(metricsOf(element.value)));
  if (settleTimer !== undefined) clearTimeout(settleTimer);
  settleTimer = setTimeout(settle, Math.max(0, settleDelay));
}

function onKeydown(event: KeyboardEvent): void {
  if (event.target !== element.value || event.altKey || event.ctrlKey || event.metaKey) return;
  const forward = vertical()
    ? "ArrowDown"
    : context.dir.value === "rtl"
      ? "ArrowLeft"
      : "ArrowRight";
  const backward = vertical()
    ? "ArrowUp"
    : context.dir.value === "rtl"
      ? "ArrowRight"
      : "ArrowLeft";
  let handled = true;
  if (event.key === forward) context.step(1, "keyboard");
  else if (event.key === backward) context.step(-1, "keyboard");
  else if (event.key === "Home") context.goTo(0, "keyboard");
  else if (event.key === "End") context.goTo(context.slideCount.value - 1, "keyboard");
  else handled = false;
  if (handled) event.preventDefault();
}

function coordinateOf(event: PointerEvent): number {
  return vertical() ? event.clientY : event.clientX;
}

function onPointerDown(event: PointerEvent): void {
  suppressClick = false;
  if (!context.draggable.value || event.pointerType !== "mouse" || event.button !== 0) return;
  if (element.value === null) return;
  drag = {
    moved: false,
    pointerId: event.pointerId,
    snapType: element.value.style.scrollSnapType,
    startCoordinate: coordinateOf(event),
    startIndex: context.index.value,
    startScroll: scrollPosition(element.value),
  };
}

function onPointerMove(event: PointerEvent): void {
  if (drag === null || event.pointerId !== drag.pointerId || element.value === null) return;
  const delta = coordinateOf(event) - drag.startCoordinate;
  if (!drag.moved) {
    if (Math.abs(delta) < DRAG_START_DISTANCE) return;
    drag.moved = true;
    context.setDragging(true);
    // Mandatory snapping would fight every manual scroll write while dragging.
    element.value.style.scrollSnapType = "none";
    try {
      element.value.setPointerCapture(event.pointerId);
    } catch {
      // The pointer may already be released; dragging still follows window events.
    }
  }
  event.preventDefault();
  setScrollPosition(element.value, drag.startScroll - delta);
}

function endDrag(event: PointerEvent): void {
  if (drag === null || event.pointerId !== drag.pointerId) return;
  const current = drag;
  drag = null;
  if (!current.moved || element.value === null) return;
  suppressClick = event.type === "pointerup";
  element.value.style.scrollSnapType = current.snapType;
  context.setDragging(false);
  const traveled = scrollPosition(element.value) - current.startScroll;
  const forwardDelta = !vertical() && context.dir.value === "rtl" ? -traveled : traveled;
  const target = slideIndexAfterDrag({
    count: context.slideCount.value,
    delta: forwardDelta,
    loop: false,
    nearestIndex: nearestIndex(element.value),
    startIndex: current.startIndex,
    threshold: Math.max(0, dragThreshold),
  });
  context.goTo(target, "drag");
}

function onClickCapture(event: MouseEvent): void {
  if (!suppressClick) return;
  suppressClick = false;
  event.preventDefault();
  event.stopPropagation();
}

const visibility = useVisibilityObserver({
  get root() {
    return element.value;
  },
  threshold: 0.5,
  onVisibilityChange(entries) {
    for (const entry of entries) {
      const index = Number(entry.target.getAttribute("data-index"));
      if (Number.isInteger(index)) context.setInView(index, entry.isIntersecting);
    }
  },
});

watch(
  context.slideElements,
  (next, previous) => {
    for (const [index, slide] of previous ?? new Map<number, HTMLElement>()) {
      if (next.get(index) !== slide) visibility.unobserve(slide);
    }
    if (element.value === null) return;
    for (const slide of next.values()) visibility.observe(slide);
  },
  { flush: "post" },
);

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  context.registerViewport({ scrollToIndex });
  if (element.value === null) return;
  element.value.addEventListener("scroll", onScroll, { passive: true });
  element.value.addEventListener("scrollend", settle);
  element.value.addEventListener("keydown", onKeydown);
  element.value.addEventListener("pointerdown", onPointerDown);
  element.value.addEventListener("pointermove", onPointerMove);
  element.value.addEventListener("pointerup", endDrag);
  element.value.addEventListener("pointercancel", endDrag);
  element.value.addEventListener("click", onClickCapture, { capture: true });
  for (const slide of context.slideElements.value.values()) visibility.observe(slide);
  if (context.index.value > 0) scrollToIndex(context.index.value, false);
  context.setEdges(scrollEdges(metricsOf(element.value)));
});

onBeforeUnmount(() => {
  context.registerViewport(null);
  if (settleTimer !== undefined) clearTimeout(settleTimer);
  if (element.value === null) return;
  element.value.removeEventListener("scroll", onScroll);
  element.value.removeEventListener("scrollend", settle);
  element.value.removeEventListener("keydown", onKeydown);
  element.value.removeEventListener("pointerdown", onPointerDown);
  element.value.removeEventListener("pointermove", onPointerMove);
  element.value.removeEventListener("pointerup", endDrag);
  element.value.removeEventListener("pointercancel", endDrag);
  element.value.removeEventListener("click", onClickCapture, { capture: true });
});

type CarouselViewportSetupExpose = Omit<CarouselViewportExpose, "element" | "id"> & {
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
};

const exposed = { element, id: context.viewportId } satisfies CarouselViewportSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.viewportId.value"
    ref="element"
    tabindex="0"
    :aria-live="context.autoplay.value === 'playing' ? 'off' : 'polite'"
    aria-atomic="false"
    data-vize-ui="carousel-viewport"
    part="viewport"
    :data-orientation="context.orientation.value"
    :data-dragging="context.slotState.value.dragging ? 'true' : undefined"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Consumers own the scroll container CSS, for example:
   overflow: auto; scroll-snap-type: x mandatory; display: flex; */
</style>
