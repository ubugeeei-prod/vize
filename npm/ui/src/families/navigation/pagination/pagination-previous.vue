<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { paginationContext } from "./pagination-context.ts";
import type {
  PaginationControlExpose,
  PaginationControlSlotState,
  PaginationControlState,
} from "./pagination-types.ts";

const {
  type = "button",
  disabled = false,
  ariaLabel = "Go to previous page",
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Disable this navigation control.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default "Go to previous page"
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label this navigation control.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe this navigation control.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

const emit = defineEmits<{
  /** Fired before this control requests the previous page. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Previous control contents. Receives target page, disabled, and state tokens. */
  default(props: PaginationControlSlotState): unknown;
}>();

const context = paginationContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const controlDisabled = computed(() => disabled || context.getPreviousState(false) === "disabled");
const controlState = computed<PaginationControlState>(() => context.getPreviousState(disabled));
const slotState = computed<PaginationControlSlotState>(() => ({
  disabled: controlDisabled.value,
  state: controlState.value,
  targetPage: context.previousPage.value,
}));

function onClick(event: MouseEvent): void {
  if (controlDisabled.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) context.goPrevious(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function select(): boolean {
  if (controlDisabled.value) return false;
  return context.goPrevious(null);
}

type PaginationPreviousSetupExpose = Omit<
  PaginationControlExpose,
  keyof PaginationControlSlotState | "element" | "id"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly state: ComputedRef<PaginationControlState>;
  readonly targetPage: ComputedRef<number | null>;
};

const exposed = {
  disabled: controlDisabled,
  element,
  focus,
  id: context.previousId,
  select,
  state: controlState,
  targetPage: context.previousPage,
} satisfies PaginationPreviousSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.previousId.value"
    ref="element"
    :type
    :disabled="controlDisabled"
    :aria-label="ariaLabelledby === undefined ? ariaLabel : undefined"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="pagination-previous"
    part="previous"
    :data-state="controlState"
    :data-target-page="context.previousPage.value ?? undefined"
    :data-disabled="controlDisabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Previous-control iconography, spacing, and focus ring remain consumer-owned. */
</style>
