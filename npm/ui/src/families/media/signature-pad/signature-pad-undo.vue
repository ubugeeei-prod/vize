<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { signaturePadContext } from "./signature-pad-context.ts";
import type { SignaturePadButtonExpose, SignaturePadSlotState } from "./signature-pad-types.ts";

const emit = defineEmits<{
  /** Fired before undoing. Call `preventDefault()` to keep the strokes. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button label. Receives the pad state. */
  default(props: SignaturePadSlotState): unknown;
}>();

const context = signaturePadContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => !context.slotState.value.canUndo);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.undo();
}

type SignaturePadUndoSetupExpose = Omit<SignaturePadButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies SignaturePadUndoSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-controls="context.id.value"
    data-vize-ui="signature-pad-undo"
    part="undo"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
