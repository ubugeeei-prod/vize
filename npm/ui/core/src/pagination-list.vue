<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { paginationContext } from "./pagination-context.ts";
import type {
  PaginationListExpose,
  PaginationListSlotState,
  PaginationState,
} from "./pagination-types.ts";
import type { PaginationRangeItem } from "./pagination-range.ts";
import type { PrimitiveAs, PrimitiveElement } from "./primitive.ts";

const { as = "ol" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "ol"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Pagination item children. Receives current page, range, and list id. */
  default(props: PaginationListSlotState): unknown;
}>();

const context = paginationContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const slotState = computed<PaginationListSlotState>(() => ({
  canNext: context.canNext.value,
  canPrevious: context.canPrevious.value,
  disabled: context.disabled.value,
  listId: context.listId.value,
  nextPage: context.nextPage.value,
  page: context.page.value,
  pageCount: context.pageCount.value,
  previousPage: context.previousPage.value,
  range: context.range.value,
  state: context.state.value,
}));

type PaginationListSetupExpose = Omit<
  PaginationListExpose,
  keyof PaginationListSlotState | "element"
> & {
  readonly canNext: ComputedRef<boolean>;
  readonly canPrevious: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly listId: ComputedRef<string>;
  readonly nextPage: ComputedRef<number | null>;
  readonly page: ComputedRef<number>;
  readonly pageCount: ComputedRef<number>;
  readonly previousPage: ComputedRef<number | null>;
  readonly range: ComputedRef<readonly PaginationRangeItem[]>;
  readonly state: ComputedRef<PaginationState>;
};

const exposed = {
  canNext: context.canNext,
  canPrevious: context.canPrevious,
  disabled: context.disabled,
  element,
  focus: context.focusCurrent,
  listId: context.listId,
  nextPage: context.nextPage,
  page: context.page,
  pageCount: context.pageCount,
  previousPage: context.previousPage,
  range: context.range,
  state: context.state,
} satisfies PaginationListSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="context.listId.value"
    ref="element"
    data-vize-ui="pagination-list"
    part="list"
    :data-state="context.state.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :data-page="context.page.value"
    :data-page-count="context.pageCount.value"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. List display, wrapping, and gap remain consumer-owned. */
</style>
