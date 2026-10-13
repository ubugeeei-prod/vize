<script setup lang="ts">
import { computed } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const width = computed(() => (mounted.value ? window.innerWidth : 0));
</script>

<template>
  <div class="composable-example">
    <p>Render a placeholder on the server and switch to browser-only content after mounting.</p>
    <output aria-live="polite">{{
      mounted ? `Viewport width: ${width}px` : "Measuring the viewport…"
    }}</output>
  </div>
</template>
