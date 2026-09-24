<script setup lang="ts">
import { useTemplateRef } from "vue";

import { signaturePadContext } from "./signature-pad-context.ts";
import type { SignaturePadGuideExpose, SignaturePadSlotState } from "./signature-pad-types.ts";

defineSlots<{
  /** Decorative guide content such as a baseline or "Sign here" hint. */
  default(props: SignaturePadSlotState): unknown;
}>();

const context = signaturePadContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

type SignaturePadGuideSetupExpose = Omit<SignaturePadGuideExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SignaturePadGuideSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    data-vize-ui="signature-pad-guide"
    part="guide"
    :data-state="context.slotState.value.state"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
