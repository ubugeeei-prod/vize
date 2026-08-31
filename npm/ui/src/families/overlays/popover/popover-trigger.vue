<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { popoverContext } from "./popover-context.ts";
import type { PopoverSlotState, PopoverTriggerExpose } from "./popover-types.ts";

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

const emit = defineEmits<{
  /** Fired before the trigger toggles the popover. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Trigger contents. Receives the current Popover state and trigger availability. */
  default(props: PopoverSlotState): unknown;
}>();

const context = popoverContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabledState = computed(() => disabled || context.disabled.value);

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

function onClick(event: MouseEvent): void {
  if (disabledState.value) {
    event.preventDefault();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) context.toggle(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type PopoverTriggerSetupExpose = Omit<PopoverTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies PopoverTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.triggerId.value"
    ref="element"
    :type
    :disabled="disabledState"
    :aria-label="ariaLabel"
    aria-haspopup="dialog"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.contentId.value"
    data-vize-ui="popover-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
    @click="onClick"
  >
    <slot
      :disabled="disabledState"
      :modal="context.modal.value"
      :open="context.open.value"
      :state="context.state.value"
    />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
