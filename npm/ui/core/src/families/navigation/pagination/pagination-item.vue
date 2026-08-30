<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { paginationContext } from "./pagination-context.ts";
import { toPaginationPageInRange } from "./pagination-range.ts";
import type {
  PaginationItemExpose,
  PaginationItemSlotState,
  PaginationPageState,
} from "./pagination-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

const {
  as = "li",
  page = undefined,
  disabled = false,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "li"
   */
  readonly as?: PrimitiveAs;

  /**
   * Page represented by the item. Omit for previous, next, or ellipsis wrappers.
   *
   * @default undefined
   */
  readonly page?: number;

  /**
   * Disable this item for styling state. Interactive child controls own activation.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Item contents. Receives current-page and disabled state. */
  default(props: PaginationItemSlotState): unknown;
}>();

const context = paginationContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const itemPage = computed(() =>
  page === undefined ? undefined : (toPaginationPageInRange(page, context.pageCount.value) ?? page),
);
const current = computed(
  () =>
    page !== undefined &&
    toPaginationPageInRange(page, context.pageCount.value) === context.page.value,
);
const itemDisabled = computed(
  () =>
    context.disabled.value ||
    disabled ||
    (page !== undefined && toPaginationPageInRange(page, context.pageCount.value) === null),
);
const itemState = computed<PaginationPageState>(() => {
  if (itemDisabled.value) return "disabled";
  return current.value ? "current" : "idle";
});
const slotState = computed<PaginationItemSlotState>(() => ({
  current: current.value,
  disabled: itemDisabled.value,
  page: itemPage.value,
  state: itemState.value,
}));

type PaginationItemSetupExpose = Omit<
  PaginationItemExpose,
  keyof PaginationItemSlotState | "element"
> & {
  readonly current: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly page: typeof itemPage;
  readonly state: ComputedRef<PaginationPageState>;
};

const exposed = {
  current,
  disabled: itemDisabled,
  element,
  page: itemPage,
  state: itemState,
} satisfies PaginationItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="pagination-item"
    part="item"
    :data-state="itemState"
    :data-page="itemPage"
    :data-current="current ? 'true' : undefined"
    :data-disabled="itemDisabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Item layout, truncation, and current-page styling remain consumer-owned. */
</style>
