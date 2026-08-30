<script setup lang="ts">
import { useTemplateRef } from "vue";

import { dialogContext } from "./dialog-context.ts";
import type { DialogCloseExpose, DialogSlotState } from "./dialog-types.ts";

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
   * Remove the close button from activation and sequential keyboard focus.
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
  /** Fired before the button requests closing. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Close button contents. Receives the current Dialog state and button availability. */
  default(props: DialogSlotState & { readonly disabled: boolean }): unknown;
}>();

const context = dialogContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");

function onClick(event: MouseEvent): void {
  if (disabled) {
    event.preventDefault();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) context.close(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type DialogCloseSetupExpose = Omit<DialogCloseExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies DialogCloseSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    :type
    :disabled
    :aria-label="ariaLabel"
    data-vize-ui="dialog-close"
    part="close"
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
