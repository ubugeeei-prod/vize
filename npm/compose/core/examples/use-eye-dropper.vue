<script setup lang="ts">
import { useEyeDropper } from "@vizejs/composable/use-eye-dropper";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const { supported, sRGBHex, open } = useEyeDropper({ initialValue: "#42b883" });
</script>

<template>
  <div class="composable-example">
    <p>Pick a brand color from anywhere on the screen.</p>
    <button type="button" :disabled="!mounted || !supported" @click="() => open()">
      Pick a color
    </button>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `Brand color: ${sRGBHex}`
          : "The EyeDropper API is not available; type a hex value instead."
    }}</output>
  </div>
</template>
