<script setup lang="ts">
import { ref } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useVibrate } from "@vizejs/composable/use-vibrate";

const mounted = useMounted();
const { supported, vibrate } = useVibrate({ pattern: [200, 100, 200] });
const result = ref("");

function buzz() {
  result.value = vibrate() ? "Vibration requested." : "The device declined to vibrate.";
}
</script>

<template>
  <div class="composable-example">
    <p>Give haptic feedback when a timer finishes, on devices that can vibrate.</p>
    <button type="button" :disabled="!mounted || !supported" @click="buzz">Buzz twice</button>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? result || "Vibration is available."
          : "Vibration is not available in this browser."
    }}</output>
  </div>
</template>
