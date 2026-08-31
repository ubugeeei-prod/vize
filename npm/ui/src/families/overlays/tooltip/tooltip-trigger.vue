<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { useHover } from "../../interaction/hover/hover.ts";
import { tooltipContext } from "./tooltip-context.ts";
import type { TooltipSlotState, TooltipTriggerExpose } from "./tooltip-types.ts";

const {
  type = "button",
  disabled = false,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Remove the trigger from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Trigger contents. Receives the current Tooltip state and trigger availability. */
  default(props: TooltipSlotState): unknown;
}>();

const context = tooltipContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabledState = computed(() => disabled || context.disabled.value);
const hover = useHover({
  isDisabled: disabledState,
  onHoverStart: (event) => context.scheduleOpen(event.originalEvent),
  onHoverEnd: (event) => context.close(event.originalEvent),
});

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

function onFocus(event: FocusEvent): void {
  if (!disabledState.value) context.scheduleOpen(event);
}

function onBlur(event: FocusEvent): void {
  context.close(event);
}

function onPointerDown(event: PointerEvent): void {
  context.close(event);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== "Escape" || event.defaultPrevented || event.isComposing) return;
  if (context.close(event)) event.preventDefault();
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type TooltipTriggerSetupExpose = Omit<TooltipTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies TooltipTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.triggerId.value"
    ref="element"
    v-bind="hover.hoverProps"
    :type
    :disabled="disabledState"
    :aria-label="ariaLabel"
    :aria-describedby="context.open.value ? context.contentId.value : undefined"
    data-vize-ui="tooltip-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
    @focus="onFocus"
    @blur="onBlur"
    @pointerdown="onPointerDown"
    @keydown="onKeydown"
  >
    <slot :disabled="disabledState" :open="context.open.value" :state="context.state.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
