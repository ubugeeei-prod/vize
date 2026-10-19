<script setup lang="ts">
import { computed, watchEffect, useTemplateRef } from "vue";
import { useDisplayMedia } from "@vizejs/composable/use-display-media";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const preview = useTemplateRef<HTMLVideoElement>("preview");
const { supported, stream, status, start, stop } = useDisplayMedia({ video: true });
const sharing = computed(() => stream.value !== undefined);
watchEffect(() => {
  if (preview.value) preview.value.srcObject = (stream.value as MediaStream | undefined) ?? null;
});
</script>

<template>
  <div class="composable-example">
    <p>Share a screen or window into a recording preview.</p>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported || sharing" @click="() => start()">
        Share a screen
      </button>
      <button type="button" :disabled="!sharing" @click="() => stop()">Stop sharing</button>
    </div>
    <video ref="preview" autoplay muted playsinline aria-label="Screen preview"></video>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `Screen capture: ${status}`
          : "Screen capture is not available in this browser."
    }}</output>
  </div>
</template>
