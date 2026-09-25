<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { webcamCaptureContext } from "./webcam-capture-context.ts";
import type { WebcamCaptureButtonExpose, WebcamCaptureSlotState } from "./webcam-capture-types.ts";

const { countdown = 0, ariaLabel = undefined } = defineProps<{
  /**
   * Seconds to count down (announced through WebcamCaptureStatusMessage) before capturing.
   *
   * @default 0
   */
  readonly countdown?: number;

  /**
   * Accessible name for icon-only buttons. The default slot renders `messages.shutter`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before capturing. Call `preventDefault()` to cancel. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content. Receives the camera state, including the running countdown. */
  default?(props: WebcamCaptureSlotState): unknown;
}>();

const context = webcamCaptureContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const state = computed(() => context.slotState.value);
const disabled = computed(
  () => state.value.status !== "active" || state.value.capturing || state.value.countdown > 0,
);
const text = computed(() => context.messages.value.shutter);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) void context.shoot(countdown);
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
    data-vize-ui="webcam-capture-shutter"
    part="shutter"
    :data-countdown="state.countdown > 0 ? state.countdown : undefined"
    :data-capturing="state.capturing ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="state">{{ text }}</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
