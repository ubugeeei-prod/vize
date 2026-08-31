<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { paginationContext } from "./pagination-context.ts";
import { toPaginationPageInRange } from "./pagination-range.ts";
import type {
  PaginationPageExpose,
  PaginationPageSlotState,
  PaginationPageState,
} from "./pagination-types.ts";

const {
  page,
  type = "button",
  disabled = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Page selected by this control. @default required */
  readonly page: number;

  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Disable this page control.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default computed from page state
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label this page control.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe this page control.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

const emit = defineEmits<{
  /** Fired before this control requests page selection. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Page control contents. Receives current, disabled, and state tokens. */
  default(props: PaginationPageSlotState): unknown;
}>();

const context = paginationContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const targetPage = computed(() => toPaginationPageInRange(page, context.pageCount.value));
const current = computed(
  () => targetPage.value !== null && context.page.value === targetPage.value,
);
const pageDisabled = computed(
  () => context.disabled.value || disabled || targetPage.value === null,
);
const pageState = computed<PaginationPageState>(() =>
  targetPage.value === null ? "disabled" : context.getPageState(targetPage.value, disabled),
);
const pageId = computed(() => context.getPageId(page));
const accessibleLabel = computed(
  () => ariaLabel ?? context.getPageLabel(targetPage.value ?? page, current.value),
);
const slotState = computed<PaginationPageSlotState>(() => ({
  current: current.value,
  disabled: pageDisabled.value,
  page: targetPage.value ?? page,
  state: pageState.value,
}));

function onClick(event: MouseEvent): void {
  if (pageDisabled.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented && targetPage.value !== null)
    context.setPage(targetPage.value, event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function select(): boolean {
  if (pageDisabled.value || targetPage.value === null) return false;
  return context.setPage(targetPage.value, null);
}

type PaginationPageSetupExpose = Omit<
  PaginationPageExpose,
  keyof PaginationPageSlotState | "element" | "id"
> & {
  readonly current: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly page: ComputedRef<number>;
  readonly state: ComputedRef<PaginationPageState>;
};

const exposed = {
  current,
  disabled: pageDisabled,
  element,
  focus,
  id: pageId,
  page: computed(() => targetPage.value ?? page),
  select,
  state: pageState,
} satisfies PaginationPageSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="pageId"
    ref="element"
    :type
    :disabled="pageDisabled"
    :aria-label="ariaLabelledby === undefined ? accessibleLabel : undefined"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-current="current ? 'page' : undefined"
    data-vize-ui="pagination-page"
    part="page"
    :data-state="pageState"
    :data-page="targetPage ?? page"
    :data-current="current ? 'true' : undefined"
    :data-disabled="pageDisabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Page button shape, current marker, and focus ring remain consumer-owned. */
</style>
