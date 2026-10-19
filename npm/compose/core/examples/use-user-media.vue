<script setup lang="ts">
import { computed, watchEffect, useTemplateRef } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useUserMedia } from "@vizejs/composable/use-user-media";

const mounted = useMounted();
const camera = useTemplateRef<HTMLVideoElement>("camera");
const { supported, stream, status, start, stop } = useUserMedia({ constraints: { video: true } });
const live = computed(() => stream.value !== undefined);
watchEffect(() => {
  if (camera.value) camera.value.srcObject = (stream.value as MediaStream | undefined) ?? null;
});
</script>

<template>
  <div class="composable-example">
    <p>Show a camera preview before a video call starts.</p>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported || live" @click="() => start()">
        Turn on camera
      </button>
      <button type="button" :disabled="!live" @click="() => stop()">Turn off camera</button>
    </div>
    <video ref="camera" autoplay muted playsinline aria-label="Camera preview"></video>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `Camera: ${status}`
          : "Camera access is not available in this browser."
    }}</output>
  </div>
</template>
