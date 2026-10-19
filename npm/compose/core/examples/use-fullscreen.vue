<script setup lang="ts">
import { useTemplateRef } from "vue";
import { useFullscreen } from "@vizejs/composable/use-fullscreen";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const slide = useTemplateRef<HTMLElement>("slide");
const { supported, isFullscreen, toggle } = useFullscreen(slide);
</script>

<template>
  <div class="composable-example">
    <p>Present one slide in fullscreen and leave with the same button or Escape.</p>
    <section ref="slide" aria-label="Slide">
      <h3>Vize at Vue Fes Japan</h3>
      <button type="button" :disabled="!mounted || !supported" @click="() => toggle()">
        {{ isFullscreen ? "Exit fullscreen" : "Present fullscreen" }}
      </button>
    </section>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !supported
          ? "Fullscreen is not available here."
          : isFullscreen
            ? "Presenting in fullscreen."
            : "Showing inline."
    }}</output>
  </div>
</template>
