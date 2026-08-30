<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../../controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { paginationContext } from "./pagination-context.ts";
import {
  createPaginationRange,
  getPaginationPageIdSegment,
  normalizePaginationPage,
  normalizePaginationPageCount,
} from "./pagination-range.ts";
import type {
  PaginationControlState,
  PaginationPageState,
  PaginationRootExpose,
  PaginationRootProps,
  PaginationSlotState,
  PaginationState,
} from "./pagination-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

const {
  id = undefined,
  pageCount,
  modelValue = undefined,
  defaultValue = undefined,
  disabled = false,
  label = "Pagination",
  siblingCount = 1,
  boundaryCount = 1,
  as = "nav",
} = defineProps<
  PaginationRootProps & {
    /**
     * Native element, custom element, or component to render.
     *
     * @default "nav"
     */
    readonly as?: PrimitiveAs;
  }
>();

const emit = defineEmits<{
  /** Fired when the current page requests a new controlled value. */
  "update:modelValue": [page: number];

  /** Fired after any distinct current-page request. */
  change: [page: number, previous: number, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Pagination children. Receives current page, range, and boundary state. */
  default(props: PaginationSlotState): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "pagination" });
const listId = computed(() => deriveDeterministicId(baseId.value, "list"));
const previousId = computed(() => deriveDeterministicId(baseId.value, "previous"));
const nextId = computed(() => deriveDeterministicId(baseId.value, "next"));
const pageCountState = computed(() => normalizePaginationPageCount(pageCount));
const disabledState = computed(() => disabled);
const valueState = useControllableState<number>({
  value: () =>
    modelValue === undefined
      ? undefined
      : normalizePaginationPage(modelValue, pageCountState.value),
  defaultValue: () => normalizePaginationPage(defaultValue, pageCountState.value),
  equals: Object.is,
});
const pageState = computed(() =>
  normalizePaginationPage(valueState.value.value, pageCountState.value),
);
const previousPage = computed(() => (pageState.value > 1 ? pageState.value - 1 : null));
const nextPage = computed(() =>
  pageState.value < pageCountState.value ? pageState.value + 1 : null,
);
const canPrevious = computed(() => !disabledState.value && previousPage.value !== null);
const canNext = computed(() => !disabledState.value && nextPage.value !== null);
const state = computed<PaginationState>(() => (disabledState.value ? "disabled" : "active"));
const range = computed(() =>
  createPaginationRange({
    boundaryCount,
    page: pageState.value,
    pageCount: pageCountState.value,
    siblingCount,
  }),
);
const slotState = computed<PaginationSlotState>(() => ({
  canNext: canNext.value,
  canPrevious: canPrevious.value,
  disabled: disabledState.value,
  nextPage: nextPage.value,
  page: pageState.value,
  pageCount: pageCountState.value,
  previousPage: previousPage.value,
  range: range.value,
  state: state.value,
}));

function getPageId(page: number): string {
  return deriveDeterministicId(
    baseId.value,
    getPaginationPageIdSegment(page, pageCountState.value),
  );
}

function getPageLabel(page: number, current: boolean): string {
  return current ? `Page ${page}, current page` : `Go to page ${page}`;
}

function getPageState(page: number, pageDisabled: boolean): PaginationPageState {
  if (disabledState.value || pageDisabled) return "disabled";
  return pageState.value === page ? "current" : "idle";
}

function getPreviousState(previousDisabled: boolean): PaginationControlState {
  return previousDisabled || !canPrevious.value ? "disabled" : "idle";
}

function getNextState(nextDisabled: boolean): PaginationControlState {
  return nextDisabled || !canNext.value ? "disabled" : "idle";
}

function getCurrentPage(): number {
  return pageState.value;
}

function setPage(page: number, nativeEvent: Event | null = null): boolean {
  if (disabledState.value) return false;
  const next = normalizePaginationPage(page, pageCountState.value);
  const previous = getCurrentPage();
  if (Object.is(previous, next)) return false;
  const changed = valueState.set(next);
  if (!changed) return false;
  emit("update:modelValue", next);
  emit("change", next, previous, nativeEvent);
  return true;
}

function goPrevious(nativeEvent: Event | null = null): boolean {
  return previousPage.value === null ? false : setPage(previousPage.value, nativeEvent);
}

function goNext(nativeEvent: Event | null = null): boolean {
  return nextPage.value === null ? false : setPage(nextPage.value, nativeEvent);
}

function reset(): boolean {
  return setPage(defaultValue ?? 1, null);
}

function focus(options?: FocusOptions): void {
  focusCurrent(options);
}

function focusCurrent(options?: FocusOptions): void {
  const root = element.value instanceof Element ? element.value : null;
  const target = root?.ownerDocument.getElementById(getPageId(pageState.value));
  if (target instanceof HTMLElement && root?.contains(target)) target.focus(options);
}

const context = paginationContext.provide({
  canNext,
  canPrevious,
  disabled: disabledState,
  focusCurrent,
  getNextState,
  getPageId,
  getPageLabel,
  getPageState,
  getPreviousState,
  goNext,
  goPrevious,
  id: baseId,
  listId,
  nextId,
  nextPage,
  page: pageState,
  pageCount: pageCountState,
  previousId,
  previousPage,
  range,
  setPage,
  state,
});

type PaginationRootSetupExpose = Omit<
  PaginationRootExpose,
  keyof PaginationSlotState | "element" | "id" | "listId"
> & {
  readonly canNext: ComputedRef<boolean>;
  readonly canPrevious: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly nextPage: ComputedRef<number | null>;
  readonly page: ComputedRef<number>;
  readonly pageCount: ComputedRef<number>;
  readonly previousPage: ComputedRef<number | null>;
  readonly range: typeof range;
  readonly state: ComputedRef<PaginationState>;
};

const exposed = {
  canNext,
  canPrevious,
  disabled: disabledState,
  element,
  focus,
  goNext,
  goPrevious,
  id: baseId,
  listId,
  nextPage,
  page: pageState,
  pageCount: pageCountState,
  previousPage,
  range,
  reset,
  setPage,
  state,
} satisfies PaginationRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="context.id.value"
    ref="element"
    :aria-label="label"
    data-vize-ui="pagination"
    part="root"
    :data-state="state"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-page="pageState"
    :data-page-count="pageCountState"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Landmarks, layout, and responsive page-window styling remain consumer-owned. */
</style>
