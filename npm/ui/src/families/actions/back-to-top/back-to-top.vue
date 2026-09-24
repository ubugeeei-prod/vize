<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type {
  BackToTopBehavior,
  BackToTopExpose,
  BackToTopSlotState,
  BackToTopState,
  BackToTopTarget,
} from "./back-to-top-types.ts";

const {
  threshold = 400,
  target = null,
  behavior = "smooth",
  focusTarget = null,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Scroll offset in CSS pixels after which the button becomes visible.
   *
   * @default 400
   */
  readonly threshold?: number;

  /**
   * Scroll container: an element, a CSS selector, or `null` for the window.
   *
   * @default null
   */
  readonly target?: BackToTopTarget;

  /**
   * Scroll animation. `"smooth"` downgrades to `"auto"` when the user prefers reduced motion.
   *
   * @default "smooth"
   */
  readonly behavior?: BackToTopBehavior;

  /**
   * Element or CSS selector that receives focus after scrolling, so keyboard and
   * screen-reader users land at the top too. `null` focuses the scroll container
   * element, or leaves focus on the button for the window.
   *
   * @default null
   */
  readonly focusTarget?: HTMLElement | string | null;

  /**
   * Accessible name when the slot does not provide visible text.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before scrolling. Call `preventDefault()` to keep the position. */
  click: [nativeEvent: MouseEvent];

  /** Fired after scrolling to the top was requested. */
  "scroll-top": [container: HTMLElement | Window];
}>();

defineSlots<{
  /** Button contents. Defaults to "Back to top". */
  default?(props: BackToTopSlotState): unknown;
}>();

const element = useTemplateRef<HTMLButtonElement>("element");
const scrollTop = shallowRef(0);
const focused = shallowRef(false);
const thresholdState = computed(() => (Number.isFinite(threshold) ? Math.max(0, threshold) : 0));
const visible = computed(() => scrollTop.value > 0 && scrollTop.value >= thresholdState.value);
const state = computed<BackToTopState>(() => (visible.value ? "visible" : "hidden"));
const slotState = computed<BackToTopSlotState>(() => ({
  scrollTop: scrollTop.value,
  state: state.value,
  visible: visible.value,
}));
let container: HTMLElement | Window | null = null;

function resolveElement(value: HTMLElement | string | null, ownerDocument: Document) {
  if (value === null) return null;
  if (typeof value !== "string") return value;
  const found = ownerDocument.querySelector(value);
  return found instanceof HTMLElement ? found : null;
}

function resolveContainer(): HTMLElement | Window | null {
  const ownerDocument = element.value?.ownerDocument;
  if (!ownerDocument) return null;
  if (target === null) return ownerDocument.defaultView;
  return resolveElement(target, ownerDocument);
}

function readOffset(source: HTMLElement | Window): number {
  return "scrollY" in source ? source.scrollY : source.scrollTop;
}

function refresh(): number {
  scrollTop.value = container === null ? 0 : readOffset(container);
  return scrollTop.value;
}

function onScroll(): void {
  refresh();
}

function detach(): void {
  container?.removeEventListener("scroll", onScroll);
  container = null;
}

function attach(): void {
  detach();
  container = resolveContainer();
  container?.addEventListener("scroll", onScroll, { passive: true });
  refresh();
}

function prefersReducedMotion(view: Window | null): boolean {
  return view?.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true;
}

function scrollToTop(): boolean {
  const scroller = container ?? resolveContainer();
  const ownerDocument = element.value?.ownerDocument;
  if (scroller === null || !ownerDocument) return false;
  const reduced = prefersReducedMotion(ownerDocument.defaultView);
  scroller.scrollTo({ behavior: behavior === "smooth" && !reduced ? "smooth" : "auto", top: 0 });
  const focusElement =
    resolveElement(focusTarget, ownerDocument) ?? ("scrollY" in scroller ? null : scroller);
  if (focusElement) {
    if (!focusElement.hasAttribute("tabindex") && focusElement.tabIndex < 0) {
      focusElement.setAttribute("tabindex", "-1");
    }
    focusElement.focus({ preventScroll: true });
  }
  emit("scroll-top", scroller);
  return true;
}

function onFocus(): void {
  focused.value = true;
}

function onBlur(): void {
  focused.value = false;
}

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) scrollToTop();
}

watch(() => target, attach, { flush: "post" });
onMounted(attach);
onScopeDispose(detach);

type BackToTopSetupExpose = Omit<BackToTopExpose, "element" | "scrollTop" | "state" | "visible"> & {
  readonly element: typeof element;
  readonly scrollTop: Readonly<ShallowRef<number>>;
  readonly state: ComputedRef<BackToTopState>;
  readonly visible: ComputedRef<boolean>;
};

const exposed = {
  element,
  refresh,
  scrollToTop,
  scrollTop,
  state,
  visible,
} satisfies BackToTopSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :hidden="visible || focused ? undefined : true"
    :aria-label="ariaLabel"
    data-vize-ui="back-to-top"
    part="root"
    :data-state="state"
    @click="onClick"
    @focus="onFocus"
    @blur="onBlur"
  >
    <slot v-bind="slotState">Back to top</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
