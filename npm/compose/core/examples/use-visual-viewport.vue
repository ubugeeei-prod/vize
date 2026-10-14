<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useVisualViewport } from "@vizejs/composable/use-visual-viewport";

const { width, height, scale, keyboardOpen } = useVisualViewport();
const mounted = useMounted();
</script>

<template>
  <div class="composable-example">
    <p>Keep a floating action bar above the on-screen keyboard on mobile.</p>
    <output v-if="mounted" aria-live="polite"
      >Visible area {{ Math.round(width) }} × {{ Math.round(height) }} at {{ scale }}× zoom ·
      {{ keyboardOpen ? "keyboard open" : "keyboard closed" }}</output
    >
    <output v-else aria-live="polite">Measuring the visible area…</output>
  </div>
</template>
