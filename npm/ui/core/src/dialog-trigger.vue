<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from "vue";

import { dialogContext } from "./dialog-context.ts";
import type { DialogSlotState, DialogTriggerExpose } from "./dialog-types.ts";

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
  /** Fired before the trigger requests opening. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Trigger contents. Receives the current Dialog state and trigger availability. */
  default(props: DialogSlotState & { readonly disabled: boolean }): unknown;
}>();

const context = dialogContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

function onClick(event: MouseEvent): void {
  if (disabled) {
    event.preventDefault();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) context.openDialog(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type DialogTriggerSetupExpose = Omit<DialogTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies DialogTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="`${context.id.value}-trigger`"
    ref="element"
    :type
    :disabled
    :aria-label="ariaLabel"
    aria-haspopup="dialog"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.contentId.value"
    data-vize-ui="dialog-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-disabled="disabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot
      :disabled
      :modal="context.modal.value"
      :open="context.open.value"
      :state="context.state.value"
    />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
