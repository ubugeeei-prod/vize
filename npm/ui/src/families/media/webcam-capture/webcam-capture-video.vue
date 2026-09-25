<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from "vue";

import { webcamCaptureContext } from "./webcam-capture-context.ts";
import type { WebcamCaptureSlotState, WebcamCaptureVideoExpose } from "./webcam-capture-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible preview name. Defaults to `messages.preview`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Fallback content for browsers without video support. */
  default?(props: WebcamCaptureSlotState): unknown;
}>();

const context = webcamCaptureContext.use();
const element = useTemplateRef<HTMLVideoElement>("element");
const label = computed(() => ariaLabel ?? context.messages.value.preview);
const mirrored = computed(() => context.slotState.value.mirrored);
const status = computed(() => context.slotState.value.status);
const mirrorStyle = computed(() => `--vize-ui-webcam-capture-scale-x:${mirrored.value ? -1 : 1}`);

onMounted(() => context.setVideoElement(element.value));
onBeforeUnmount(() => context.setVideoElement(null));

type WebcamCaptureVideoSetupExpose = Omit<WebcamCaptureVideoExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies WebcamCaptureVideoSetupExpose;

defineExpose(exposed);
</script>

<template>
  <!-- A live camera preview has no captions to provide. -->
  <!-- eslint-disable a11y/media-has-caption -->
  <video
    ref="element"
    muted
    autoplay
    playsinline
    :aria-label="label"
    data-vize-ui="webcam-capture-video"
    part="video"
    :data-status="status"
    :data-mirrored="mirrored ? 'true' : 'false'"
    :style="mirrorStyle"
  >
    <slot v-bind="context.slotState.value" />
  </video>
</template>

<style scoped>
/* Headless by design. Mirror with: transform: scaleX(var(--vize-ui-webcam-capture-scale-x)); */
</style>
