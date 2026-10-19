<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import type { AppBarExpose, AppBarSlotState, AppBarState, AppBarTarget } from "./app-bar-types.ts";

const {
  as = "header",
  target = null,
  elevateThreshold = 0,
  hideOnScroll = false,
  hideThreshold = 64,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Host element. A top-level `header` is the page's banner landmark; use `div`
   * when the bar sits inside another landmark that already has a header.
   *
   * @default "header"
   */
  readonly as?: "header" | "div";

  /**
   * Scroll container to follow: an element, a selector, or `null` for the window.
   *
   * @default null
   */
  readonly target?: AppBarTarget;

  /**
   * Scroll offset in CSS pixels past which the bar reports `elevated`.
   *
   * @default 0
   */
  readonly elevateThreshold?: number;

  /**
   * Report `hidden` while the container scrolls down, and visible again as soon as it scrolls up.
   *
   * @default false
   */
  readonly hideOnScroll?: boolean;

  /**
   * Scroll offset in CSS pixels before `hideOnScroll` may hide the bar.
   *
   * @default 64
   */
  readonly hideThreshold?: number;

  /**
   * Accessible name of the landmark when no visible title labels it.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the landmark (overrides `ariaLabel`).
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired when `hideOnScroll` hides or reveals the bar. */
  visibilityChange: [hidden: boolean];
}>();

defineSlots<{
  /** Leading content such as a navigation or back button. */
  start?(props: AppBarSlotState): unknown;
  /** Title and main content. */
  default?(props: AppBarSlotState): unknown;
  /** Trailing actions. */
  end?(props: AppBarSlotState): unknown;
}>();

const element = useTemplateRef<HTMLElement>("element");
const scrollTop = shallowRef(0);
const hidden = shallowRef(false);
const focusWithin = shallowRef(false);
const finite = (value: number, fallback: number) =>
  Number.isFinite(value) ? Math.max(0, value) : fallback;
const elevated = computed(() => scrollTop.value > finite(elevateThreshold, 0));
const state = computed<AppBarState>(() => (hidden.value ? "hidden" : "visible"));
const slotState = computed<AppBarSlotState>(() => ({
  elevated: elevated.value,
  hidden: hidden.value,
  scrollTop: scrollTop.value,
  state: state.value,
}));
let container: HTMLElement | Window | null = null;

function setHidden(next: boolean): void {
  if (hidden.value === next) return;
  hidden.value = next;
  emit("visibilityChange", next);
}

function resolveContainer(): HTMLElement | Window | null {
  const ownerDocument = element.value?.ownerDocument;
  if (!ownerDocument) return null;
  if (target === null) return ownerDocument.defaultView;
  if (typeof target !== "string") return target;
  const found = ownerDocument.querySelector(target);
  return found instanceof HTMLElement ? found : null;
}

function readOffset(source: HTMLElement | Window): number {
  const offset = "scrollY" in source ? source.scrollY : source.scrollTop;
  return Number.isFinite(offset) ? Math.max(0, offset) : 0;
}

function refresh(): number {
  const previous = scrollTop.value;
  const next = container === null ? 0 : readOffset(container);
  scrollTop.value = next;
  if (!hideOnScroll || focusWithin.value) setHidden(false);
  else if (next < previous || next <= finite(hideThreshold, 64)) setHidden(false);
  else if (next > previous) setHidden(true);
  return next;
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
  scrollTop.value = container === null ? 0 : readOffset(container);
}

function onFocusin(): void {
  focusWithin.value = true;
  setHidden(false);
}

function onFocusout(event: FocusEvent): void {
  const next = event.relatedTarget;
  focusWithin.value = next instanceof Node && element.value?.contains(next) === true;
}

function show(): void {
  setHidden(false);
}

onMounted(attach);
watch(
  () => target,
  () => attach(),
);
watch(
  () => hideOnScroll,
  (enabled) => {
    if (!enabled) setHidden(false);
  },
);
onScopeDispose(detach);

defineExpose({
  element,
  elevated,
  hidden,
  refresh,
  scrollTop,
  show,
  state,
} satisfies Omit<AppBarExpose, "element" | "elevated" | "hidden" | "scrollTop" | "state"> & {
  readonly element: typeof element;
  readonly elevated: ComputedRef<boolean>;
  readonly hidden: Readonly<ShallowRef<boolean>>;
  readonly scrollTop: Readonly<ShallowRef<number>>;
  readonly state: ComputedRef<AppBarState>;
});
</script>

<template>
  <component
    :is="as"
    ref="element"
    :aria-label="ariaLabelledby === undefined ? ariaLabel : undefined"
    :aria-labelledby
    part="root"
    data-vize-ui="app-bar"
    :data-state="state"
    :data-elevated="elevated ? 'true' : undefined"
    :data-hidden="hidden ? 'true' : undefined"
    @focusin="onFocusin"
    @focusout="onFocusout"
  >
    <div v-if="$slots.start" part="start" data-vize-ui="app-bar-start">
      <slot name="start" v-bind="slotState" />
    </div>
    <div part="content" data-vize-ui="app-bar-content">
      <slot v-bind="slotState" />
    </div>
    <div v-if="$slots.end" part="end" data-vize-ui="app-bar-end">
      <slot name="end" v-bind="slotState" />
    </div>
  </component>
</template>

<style scoped>
/* Headless by design. Position, elevation, and the hide transition are consumer-owned. */
</style>
