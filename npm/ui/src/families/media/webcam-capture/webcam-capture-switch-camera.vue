<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { webcamCaptureContext } from "./webcam-capture-context.ts";
import type { WebcamCaptureButtonExpose, WebcamCaptureSlotState } from "./webcam-capture-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name for icon-only buttons. The default slot renders `messages.switchCamera`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before the action. Call `preventDefault()` to cancel it. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content. Receives the camera state. */
  default?(props: WebcamCaptureSlotState): unknown;
}>();

const context = webcamCaptureContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const status = computed(() => context.slotState.value.status);
const disabled = computed(() => context.slotState.value.external);
const text = computed(() => context.messages.value.switchCamera);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) void context.switchCamera();
}

type SetupExpose = Omit<WebcamCaptureButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies SetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    :aria-controls="context.id.value"
    data-vize-ui="webcam-capture-switch-camera"
    part="switch-camera"
    :data-status="status"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value">{{ text }}</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
