export const webcamCaptureRendererFixtures = [
  {
    filename: "WebcamCaptureConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  WebcamCaptureDeviceSelect,
  WebcamCapturePhoto,
  WebcamCaptureRoot,
  WebcamCaptureShutter,
  WebcamCaptureStartButton,
  WebcamCaptureStatusMessage,
  WebcamCaptureStopButton,
  WebcamCaptureSwitchCamera,
  WebcamCaptureVideo,
} from "./families/media/webcam-capture/webcam-capture.ts";
import type { WebcamCapturePhotoResult, WebcamFacingMode } from "./families/media/webcam-capture/webcam-capture.ts";

const facingMode = ref<WebcamFacingMode>("user");
const last = ref<WebcamCapturePhotoResult | null>(null);

function onCapture(photo: WebcamCapturePhotoResult): void {
  last.value = photo;
}
</script>

<template>
  <WebcamCaptureRoot
    v-model:facing-mode="facingMode"
    :aspect-ratio="1"
    capture-type="image/jpeg"
    :messages="{ shutter: 'Snap' }"
    @capture="onCapture"
  >
    <template #default="{ status }">
      <WebcamCaptureVideo />
      <WebcamCaptureStartButton />
      <WebcamCaptureStopButton />
      <WebcamCaptureSwitchCamera />
      <WebcamCaptureDeviceSelect />
      <WebcamCaptureShutter :countdown="3" />
      <WebcamCapturePhoto>
        <template #empty>No photo yet ({{ status }})</template>
      </WebcamCapturePhoto>
      <WebcamCaptureStatusMessage />
    </template>
  </WebcamCaptureRoot>
</template>
`,
  },
] as const;
