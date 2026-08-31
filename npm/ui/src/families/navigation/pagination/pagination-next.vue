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
  ariaLabel = "Go to next page",
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
   * @default "Go to next page"
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
  /** Fired before this control requests the next page. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Next control contents. Receives target page, disabled, and state tokens. */
  default(props: PaginationControlSlotState): unknown;
}>();

const context = paginationContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const controlDisabled = computed(() => disabled || context.getNextState(false) === "disabled");
const controlState = computed<PaginationControlState>(() => context.getNextState(disabled));
const slotState = computed<PaginationControlSlotState>(() => ({
  disabled: controlDisabled.value,
  state: controlState.value,
  targetPage: context.nextPage.value,
}));

function onClick(event: MouseEvent): void {
  if (controlDisabled.value) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) context.goNext(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function select(): boolean {
  if (controlDisabled.value) return false;
  return context.goNext(null);
}

type PaginationNextSetupExpose = Omit<
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
  id: context.nextId,
  select,
  state: controlState,
  targetPage: context.nextPage,
} satisfies PaginationNextSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.nextId.value"
    ref="element"
    :type
    :disabled="controlDisabled"
    :aria-label="ariaLabelledby === undefined ? ariaLabel : undefined"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="pagination-next"
    part="next"
    :data-state="controlState"
    :data-target-page="context.nextPage.value ?? undefined"
    :data-disabled="controlDisabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Next-control iconography, spacing, and focus ring remain consumer-owned. */
</style>
