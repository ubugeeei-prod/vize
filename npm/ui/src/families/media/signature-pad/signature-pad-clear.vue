<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { signaturePadContext } from "./signature-pad-context.ts";
import type { SignaturePadButtonExpose, SignaturePadSlotState } from "./signature-pad-types.ts";

const emit = defineEmits<{
  /** Fired before clearing. Call `preventDefault()` to keep the strokes. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button label. Receives the pad state. */
  default(props: SignaturePadSlotState): unknown;
}>();

const context = signaturePadContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => !context.interactive.value || context.slotState.value.empty);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.clear();
}

type SignaturePadClearSetupExpose = Omit<SignaturePadButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies SignaturePadClearSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-controls="context.id.value"
    data-vize-ui="signature-pad-clear"
    part="clear"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
