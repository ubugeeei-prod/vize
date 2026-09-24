<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { webcamCaptureContext } from "./webcam-capture-context.ts";
import type { WebcamCaptureSlotState, WebcamCaptureStatusExpose } from "./webcam-capture-types.ts";

defineSlots<{
  /** Announcement content. Defaults to the localized status, countdown, or capture text. */
  default?(props: WebcamCaptureSlotState & { readonly message: string }): unknown;
}>();

const context = webcamCaptureContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const message = computed(() => context.announcement.value);
const slotState = computed(() => ({ ...context.slotState.value, message: message.value }));

type SetupExpose = Omit<WebcamCaptureStatusExpose, "element" | "message"> & {
  readonly element: typeof element;
  readonly message: ComputedRef<string>;
};

const exposed = { element, message } satisfies SetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    role="status"
    aria-live="polite"
    aria-atomic="true"
    data-vize-ui="webcam-capture-status"
    part="status"
    :data-status="context.slotState.value.status"
  >
    <slot v-bind="slotState">{{ message }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
