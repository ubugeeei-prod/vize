<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  onScopeDispose,
  shallowRef,
  useTemplateRef,
  watch,
} from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { carouselContext } from "./carousel-context.ts";
import type { CarouselContextValue, CarouselViewportController } from "./carousel-context.ts";
import { resolveSlideIndex } from "./carousel-geometry.ts";
import type { CarouselScrollEdges } from "./carousel-geometry.ts";
import type {
  CarouselAutoplayState,
  CarouselChangeReason,
  CarouselDirection,
  CarouselFocusBehavior,
  CarouselOrientation,
  CarouselRootExpose,
  CarouselSlotState,
} from "./carousel-types.ts";

const {
  id = undefined,
  slideCount,
  modelValue = undefined,
  defaultValue = 0,
  loop = false,
  orientation = "horizontal",
  dir = "ltr",
  draggable = true,
  autoplay = false,
  playing = undefined,
  interval = 5000,
  pauseOnHover = true,
  focusBehavior = "stop",
  respectReducedMotion = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Consumer-owned carousel base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Number of slides. Declared up front so slide labels ("2 of 5"), navigation
   * availability, and indicators render correctly on the server. @default required
   */
  readonly slideCount: number;

  /**
   * Controlled zero-based active slide index. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: number;

  /**
   * Initial active slide for uncontrolled use.
   *
   * @default 0
   */
  readonly defaultValue?: number;

  /**
   * Wrap previous/next navigation and keyboard movement at both ends.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Scroll axis of the track.
   *
   * @default "horizontal"
   */
  readonly orientation?: CarouselOrientation;

  /**
   * Reading direction used for horizontal arrow keys and scroll offsets.
   *
   * @default "ltr"
   */
  readonly dir?: CarouselDirection;

  /**
   * Let mouse pointers drag the track. Touch and pen swipes always use native scrolling.
   *
   * @default true
   */
  readonly draggable?: boolean;

  /**
   * Initial automatic rotation intent for uncontrolled use.
   *
   * @default false
   */
  readonly autoplay?: boolean;

  /**
   * Controlled automatic rotation intent (`v-model:playing`). `undefined` selects {@link autoplay}.
   *
   * @default undefined
   */
  readonly playing?: boolean;

  /**
   * Milliseconds each slide stays active while rotating.
   *
   * @default 5000
   */
  readonly interval?: number;

  /**
   * Pause rotation while a mouse or pen pointer hovers the carousel.
   *
   * @default true
   */
  readonly pauseOnHover?: boolean;

  /**
   * How keyboard focus entering the carousel affects rotation. `stop` follows the
   * WAI-ARIA carousel pattern and turns rotation off until explicitly restarted.
   *
   * @default "stop"
   */
  readonly focusBehavior?: CarouselFocusBehavior;

  /**
   * Pause rotation while `prefers-reduced-motion: reduce` matches, and scroll
   * without smooth animation.
   *
   * @default true
   */
  readonly respectReducedMotion?: boolean;

  /**
   * Accessible carousel name when no visible heading supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the carousel.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired when the active slide requests a new controlled index. */
  "update:modelValue": [index: number];

  /** Fired after any distinct active-slide request, with its cause. */
  change: [index: number, previous: number, reason: CarouselChangeReason];

  /** Fired when rotation intent requests a new controlled value. */
  "update:playing": [playing: boolean];
}>();

defineSlots<{
  /** Viewport, slides, controls, and indicators. Receives the carousel state. */
  default(props: CarouselSlotState): unknown;
}>();

const element = useTemplateRef<HTMLElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "carousel" });
const viewportId = computed(() => deriveDeterministicId(baseId.value, "viewport"));
const count = computed(() => Math.max(0, Math.floor(slideCount)));
const loopState = computed(() => loop && count.value > 1);
const orientationState = computed(() => orientation);
const dirState = computed(() => dir);
const indexState = useControllableState<number>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
});
const activeIndex = computed(() => resolveSlideIndex(indexState.value.value, count.value, false));
const playingState = useControllableState<boolean>({
  value: () => playing,
  defaultValue: () => autoplay,
});
const edges = shallowRef<CarouselScrollEdges>({ atStart: null, atEnd: null });
const inView = shallowRef<ReadonlyMap<number, boolean>>(new Map());
const slideElements = shallowRef<ReadonlyMap<number, HTMLElement>>(new Map());
const indicatorElements = new Map<number, HTMLButtonElement>();
const dragging = shallowRef(false);
const hovered = shallowRef(false);
const focusWithin = shallowRef(false);
const documentHidden = shallowRef(false);
// Read on mount so server and hydration renders agree; the query updates live afterwards.
const reducedMotionQuery = shallowRef(false);
let reducedMotionList: MediaQueryList | null = null;
const reducedMotion = computed(() => respectReducedMotion && reducedMotionQuery.value);
let viewport: CarouselViewportController | null = null;
let pendingReason: CarouselChangeReason = "api";
let pointerInitiatedFocus = false;
let mounted = false;
let timer: ReturnType<typeof setTimeout> | undefined;

const canScrollPrev = computed(() => count.value > 1 && (loopState.value || activeIndex.value > 0));
const canScrollNext = computed(
  () =>
    count.value > 1 &&
    (loopState.value || (activeIndex.value < count.value - 1 && edges.value.atEnd !== true)),
);
const suspended = computed(
  () =>
    dragging.value ||
    documentHidden.value ||
    reducedMotion.value ||
    (pauseOnHover && hovered.value) ||
    (focusBehavior === "pause" && focusWithin.value),
);
const autoplayState = computed<CarouselAutoplayState>(() => {
  if (!playingState.value.value || count.value < 2) return "stopped";
  return suspended.value ? "paused" : "playing";
});
const slotState = computed<CarouselSlotState>(() => ({
  autoplay: autoplayState.value,
  canScrollNext: canScrollNext.value,
  canScrollPrev: canScrollPrev.value,
  dragging: dragging.value,
  index: activeIndex.value,
  orientation: orientationState.value,
  slideCount: count.value,
}));

function getSlideId(index: number): string {
  return deriveDeterministicId(baseId.value, `slide-${index}`);
}

function currentIndex(): number {
  return activeIndex.value;
}

function goTo(requested: number, reason: CarouselChangeReason): boolean {
  const target = resolveSlideIndex(requested, count.value, false);
  const previous = currentIndex();
  if (target === previous) {
    // Re-align after drags, scroll snapping mismatches, or unmeasured jumps.
    if (reason !== "scroll") viewport?.scrollToIndex(target, true);
    return false;
  }
  pendingReason = reason;
  indexState.set(target);
  emit("update:modelValue", target);
  emit("change", target, previous, reason);
  if (reason === "scroll") {
    // A controlled parent may reject a user scroll; scroll back to its index.
    void nextTick(() => {
      if (activeIndex.value !== target) viewport?.scrollToIndex(activeIndex.value, true);
    });
  }
  return true;
}

function step(delta: 1 | -1, reason: CarouselChangeReason): boolean {
  if (delta > 0 ? !canScrollNext.value : !canScrollPrev.value) return false;
  return goTo(resolveSlideIndex(activeIndex.value + delta, count.value, loopState.value), reason);
}

function setPlaying(next: boolean): boolean {
  if (playingState.value.value === next) return false;
  playingState.set(next);
  emit("update:playing", next);
  return true;
}

function clearTimer(): void {
  if (timer !== undefined) clearTimeout(timer);
  timer = undefined;
}

function advance(): void {
  if (canScrollNext.value) step(1, "autoplay");
  else goTo(0, "autoplay");
}

function schedule(): void {
  clearTimer();
  if (!mounted || autoplayState.value !== "playing") return;
  timer = setTimeout(
    () => {
      timer = undefined;
      advance();
    },
    Math.max(0, interval),
  );
}

watch(
  activeIndex,
  (index) => {
    if (pendingReason !== "scroll") viewport?.scrollToIndex(index, true);
    pendingReason = "api";
  },
  { flush: "post" },
);
watch([autoplayState, activeIndex, () => interval], schedule);
onScopeDispose(clearTimer);

function onPointerEnter(event: PointerEvent): void {
  if (event.pointerType !== "touch") hovered.value = true;
}

function onPointerLeave(): void {
  hovered.value = false;
}

function onPointerDown(): void {
  pointerInitiatedFocus = true;
  setTimeout(() => {
    pointerInitiatedFocus = false;
  }, 0);
}

function onFocusIn(): void {
  if (pointerInitiatedFocus) return;
  if (focusBehavior === "stop") setPlaying(false);
  focusWithin.value = true;
}

function onFocusOut(event: FocusEvent): void {
  if (event.relatedTarget instanceof Node && element.value?.contains(event.relatedTarget)) return;
  focusWithin.value = false;
}

function onReducedMotionChange(event: MediaQueryListEvent): void {
  reducedMotionQuery.value = event.matches;
}

function onVisibilityChange(): void {
  documentHidden.value = document.visibilityState === "hidden";
}

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  mounted = true;
  element.value?.addEventListener("pointerenter", onPointerEnter);
  element.value?.addEventListener("pointerleave", onPointerLeave);
  element.value?.addEventListener("pointerdown", onPointerDown, { capture: true });
  element.value?.addEventListener("focusin", onFocusIn);
  element.value?.addEventListener("focusout", onFocusOut);
  document.addEventListener("visibilitychange", onVisibilityChange);
  onVisibilityChange();
  if (typeof globalThis.matchMedia === "function") {
    reducedMotionList = globalThis.matchMedia("(prefers-reduced-motion: reduce)");
    reducedMotionQuery.value = reducedMotionList.matches;
    reducedMotionList.addEventListener("change", onReducedMotionChange);
  }
  schedule();
});

onBeforeUnmount(() => {
  mounted = false;
  clearTimer();
  element.value?.removeEventListener("pointerenter", onPointerEnter);
  element.value?.removeEventListener("pointerleave", onPointerLeave);
  element.value?.removeEventListener("pointerdown", onPointerDown, { capture: true });
  element.value?.removeEventListener("focusin", onFocusIn);
  element.value?.removeEventListener("focusout", onFocusOut);
  document.removeEventListener("visibilitychange", onVisibilityChange);
  reducedMotionList?.removeEventListener("change", onReducedMotionChange);
  reducedMotionList = null;
});

function updateMap<Value>(
  source: ReadonlyMap<number, Value>,
  index: number,
  value: Value | undefined,
): ReadonlyMap<number, Value> {
  const next = new Map(source);
  if (value === undefined) next.delete(index);
  else next.set(index, value);
  return next;
}

carouselContext.provide({
  autoplay: autoplayState,
  canScrollNext,
  canScrollPrev,
  dir: dirState,
  draggable: computed(() => draggable),
  focusIndicator: (index) => indicatorElements.get(index)?.focus(),
  getSlideId,
  goTo,
  id: baseId,
  index: activeIndex,
  isInView: (index) => inView.value.get(index) ?? null,
  loop: loopState,
  orientation: orientationState,
  playing: computed(() => playingState.value.value),
  reducedMotion,
  registerIndicator(index, indicator) {
    indicatorElements.set(index, indicator);
    return () => {
      if (indicatorElements.get(index) === indicator) indicatorElements.delete(index);
    };
  },
  registerSlide(index, slide) {
    slideElements.value = updateMap(slideElements.value, index, slide);
    return () => {
      if (slideElements.value.get(index) !== slide) return;
      slideElements.value = updateMap(slideElements.value, index, undefined);
      inView.value = updateMap(inView.value, index, undefined);
    };
  },
  registerViewport(controller) {
    viewport = controller;
  },
  setDragging(next) {
    dragging.value = next;
  },
  setEdges(next) {
    if (next.atStart !== edges.value.atStart || next.atEnd !== edges.value.atEnd) {
      edges.value = next;
    }
  },
  setInView(index, visible) {
    if (inView.value.get(index) !== visible) inView.value = updateMap(inView.value, index, visible);
  },
  setPlaying,
  slideCount: count,
  slideElements,
  slotState,
  step,
  viewportId,
} satisfies CarouselContextValue);

type CarouselRootSetupExpose = Omit<CarouselRootExpose, keyof CarouselSlotState | "element"> & {
  readonly autoplay: ComputedRef<CarouselAutoplayState>;
  readonly canScrollNext: ComputedRef<boolean>;
  readonly canScrollPrev: ComputedRef<boolean>;
  readonly dragging: typeof dragging;
  readonly element: typeof element;
  readonly index: ComputedRef<number>;
  readonly orientation: ComputedRef<CarouselOrientation>;
  readonly slideCount: ComputedRef<number>;
};

const exposed = {
  autoplay: autoplayState,
  canScrollNext,
  canScrollPrev,
  dragging,
  element,
  index: activeIndex,
  orientation: orientationState,
  play: () => setPlaying(true),
  scrollNext: () => step(1, "api"),
  scrollPrev: () => step(-1, "api"),
  scrollTo: (index: number) => goTo(index, "api"),
  slideCount: count,
  stop: () => setPlaying(false),
} satisfies CarouselRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <section
    :id="baseId"
    ref="element"
    aria-roledescription="carousel"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :dir="dirState"
    data-vize-ui="carousel-root"
    part="root"
    :data-orientation="orientationState"
    :data-autoplay="autoplayState"
    :data-index="activeIndex"
    :data-dragging="dragging ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </section>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
