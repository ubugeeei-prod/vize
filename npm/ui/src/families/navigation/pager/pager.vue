<script setup lang="ts" generic="PageId extends string">
import { computed, nextTick, watch } from "vue";
import type { ComputedRef } from "vue";

import { pagerContext } from "./pager-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type { PagerChangeReason, PagerExpose, PagerSlotState } from "./pager-types.ts";

const {
  pages,
  modelValue = undefined,
  defaultValue = undefined,
  id = undefined,
} = defineProps<{
  /**
   * Page ids in order; their literal union types `v-model` and `change`.
   *
   * @default required
   */
  readonly pages: readonly PageId[];

  /**
   * Controlled active page. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: PageId;

  /**
   * Initial uncontrolled page.
   *
   * @default pages[0]
   */
  readonly defaultValue?: PageId;

  /**
   * Base id for tabs and panels. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;
}>();

const emit = defineEmits<{
  /** Fired when the active page requests a change. */
  "update:modelValue": [page: PageId];

  /** Fired after the active page changes, with the previous page and what caused it. */
  change: [page: PageId, previous: PageId, reason: PagerChangeReason];
}>();

defineSlots<{
  /** Renders PagerTabList and PagerViewport with the typed page state. */
  default(props: PagerSlotState<PageId>): unknown;
}>();

const baseId = useDeterministicId({ id: () => id, hint: "pager" });
const firstPage = computed<PageId>(() => {
  const first = pages[0];
  if (first === undefined) throw new TypeError("VIZE_UI_PAGER_PAGES: pages must not be empty");
  return first;
});
const toPage = (candidate: string | undefined): PageId | undefined =>
  pages.find((page) => page === candidate);
const state = useControllableState<PageId>({
  value: () => (modelValue === undefined ? undefined : (toPage(modelValue) ?? firstPage.value)),
  defaultValue: () => toPage(defaultValue) ?? firstPage.value,
  onChange: (page) => emit("update:modelValue", page),
});
const active = computed(() => toPage(state.value.value) ?? firstPage.value);
const index = computed(() => Math.max(pages.indexOf(active.value), 0));
const tabs = new Map<string, HTMLElement>();
let viewport: HTMLElement | null = null;
// Programmatic scrolls settle with their own scroll events; ignore those until they finish.
let programmaticTarget: number | null = null;

function reducedMotion(element: HTMLElement): boolean {
  return (
    element.ownerDocument.defaultView?.matchMedia?.("(prefers-reduced-motion: reduce)").matches ===
    true
  );
}

function scrollToIndex(target: number, instant = false): void {
  const element = viewport;
  if (element === null || element.clientWidth <= 0) return;
  const rtl = element.ownerDocument.defaultView?.getComputedStyle(element).direction === "rtl";
  const left = target * element.clientWidth * (rtl ? -1 : 1);
  if (Math.abs(element.scrollLeft - left) < 1) return;
  programmaticTarget = target;
  const behavior = instant || reducedMotion(element) ? "auto" : "smooth";
  if (typeof element.scrollTo === "function") element.scrollTo({ left, behavior });
  else element.scrollLeft = left;
}

function select(page: string, reason: PagerChangeReason): boolean {
  const target = toPage(page);
  if (target === undefined || target === active.value) return false;
  const previous = pages[index.value] ?? firstPage.value;
  state.set(target);
  emit("change", target, previous, reason);
  if (reason !== "scroll") scrollToIndex(pages.indexOf(target));
  return true;
}

function settleScroll(): void {
  const element = viewport;
  if (element === null || element.clientWidth <= 0) return;
  const settled = Math.round(Math.abs(element.scrollLeft) / element.clientWidth);
  if (programmaticTarget !== null) {
    if (settled === programmaticTarget) programmaticTarget = null;
    return;
  }
  const page = pages[Math.min(Math.max(settled, 0), pages.length - 1)];
  if (page !== undefined) select(page, "scroll");
}

// Controlled changes from the parent scroll the viewport too.
watch(active, (page) => {
  void nextTick(() => scrollToIndex(pages.indexOf(page)));
});

pagerContext.provide({
  baseId,
  active: computed(() => active.value),
  pages: computed(() => pages),
  select,
  registerTab: (page: string, element: HTMLElement | null) => {
    if (element === null) tabs.delete(page);
    else tabs.set(page, element);
  },
  focusTab: (page: string) => tabs.get(page)?.focus(),
  registerViewport: (element: HTMLElement | null) => {
    viewport = element;
    // The initial page is positioned instantly once the viewport exists (after hydration).
    if (element !== null) void nextTick(() => scrollToIndex(index.value, true));
  },
  settleScroll,
});

const slotState = computed<PagerSlotState<PageId>>(() => ({
  pages,
  active: active.value,
  index: index.value,
  count: pages.length,
}));

type PagerSetupExpose = Omit<PagerExpose<PageId>, keyof PagerSlotState<PageId>> & {
  readonly [Key in keyof PagerSlotState<PageId>]: ComputedRef<PagerSlotState<PageId>[Key]>;
};

function field<Key extends keyof PagerSlotState<PageId>>(
  key: Key,
): ComputedRef<PagerSlotState<PageId>[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  pages: field("pages"),
  active: field("active"),
  index: field("index"),
  count: field("count"),
  goTo: (page: PageId) => select(page, "api"),
  next: () => {
    const page = pages[index.value + 1];
    return page === undefined ? false : select(page, "api");
  },
  previous: () => {
    const page = pages[index.value - 1];
    return page === undefined ? false : select(page, "api");
  },
} satisfies PagerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div :id="baseId" part="root" data-vize-ui="pager" :data-page="active" :data-count="pages.length">
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
