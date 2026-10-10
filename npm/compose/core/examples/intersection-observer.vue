<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { useIntersectionObserver } from "@vizejs/composable/intersection-observer";

const viewport = useTemplateRef<HTMLElement>("viewport");
const featured = useTemplateRef<HTMLElement>("featured");
const ratio = ref(0);
const visible = ref(false);
const stopped = ref(false);
const { isSupported, isActive, pause, resume, stop } = useIntersectionObserver(
  featured,
  (entries) => {
    if (!isActive.value) return;
    const entry = entries.at(-1);
    if (!entry) return;
    ratio.value = entry.intersectionRatio;
    visible.value = entry.isIntersecting && entry.intersectionRatio >= 0.6;
  },
  { root: viewport, threshold: 0.6 },
);

function showFeatured(): void {
  if (!viewport.value || !featured.value) return;
  viewport.value.scrollTo({ top: featured.value.offsetTop - 12, behavior: "auto" });
}

function stopObservation(): void {
  stop();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <section ref="viewport" class="guide-viewport" aria-label="Scrollable guide cards">
      <div class="guide-space">
        <strong>Preparation</strong>
        <p>Move to the featured guide below.</p>
      </div>
      <article ref="featured" class="featured-guide" :class="{ visible }">
        <strong>Featured guide: keyboard interaction</strong>
        <p>Use native controls before adding shortcuts.</p>
      </article>
      <div class="guide-space">
        <strong>Next steps</strong>
        <p>Review the deployed result.</p>
      </div>
    </section>
    <div class="example-actions">
      <button type="button" @click="showFeatured">Show featured guide</button>
      <button type="button" @click="() => viewport?.scrollTo({ top: 0 })">
        Return to preparation
      </button>
    </div>
    <output aria-live="polite">
      Last observed featured visibility: {{ visible ? "at least 60%" : "below 60%" }} · Visible
      ratio: {{ Math.round(ratio * 100) }}%
    </output>
    <div class="example-actions">
      <button type="button" :disabled="!isActive || stopped" @click="() => pause()">
        Pause visibility observation
      </button>
      <button type="button" :disabled="isActive || stopped" @click="() => resume()">
        Resume visibility observation
      </button>
      <button type="button" :disabled="stopped" @click="stopObservation">
        Stop visibility observation
      </button>
    </div>
    <p>Observation: {{ stopped ? "stopped" : isActive ? "requested" : "paused" }}.</p>
    <p>
      The native observer measures this viewport. The example records entries only while observation
      is active, retaining its last readout while paused. Resume observes the current position; Stop
      permanently disables observation. The native scrolling buttons remain usable.
    </p>
    <p v-if="!isSupported">Intersection observation is not available in this context.</p>
  </div>
</template>

<style scoped>
.guide-viewport {
  position: relative;
  block-size: 160px;
  overflow: auto;
  border: 1px solid currentColor;
  border-radius: 8px;
}
.guide-space {
  box-sizing: border-box;
  block-size: 190px;
  padding: 12px;
}
.featured-guide {
  box-sizing: border-box;
  min-block-size: 90px;
  padding: 12px;
  border: 2px solid currentColor;
}
.featured-guide.visible {
  background: #e8f5e9;
}
</style>
