<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useServiceWorker } from "@vizejs/composable/use-service-worker";

const mounted = useMounted();
// Serve `/sw.js` from your app's public directory.
const { supported, state, updateAvailable, register, skipWaiting } = useServiceWorker("/sw.js", {
  immediate: false,
});
</script>

<template>
  <div class="composable-example">
    <p>Install offline support on request and offer a reload when an update is waiting.</p>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported" @click="() => register()">
        Enable offline mode
      </button>
      <button type="button" :disabled="!updateAvailable" @click="() => skipWaiting()">
        Reload to update
      </button>
    </div>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `Service worker: ${state}`
          : "Service workers are not available in this browser."
    }}</output>
  </div>
</template>
