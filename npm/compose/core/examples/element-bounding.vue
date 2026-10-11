<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { useElementBounding } from "@vizejs/composable/element-bounding";

const panel = useTemplateRef<HTMLElement>("panel");
const visible = ref(true);
const wide = ref(false);
const shifted = ref(false);
const stopped = ref(false);
const { left, top, width, height, update, stop } = useElementBounding(panel);

function stopObservation(): void {
  stop();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <div class="measurement-controls">
      <label><input v-model="visible" type="checkbox" /> Show guide panel</label>
      <label><input v-model="wide" type="checkbox" /> Wide guide panel</label>
      <label><input v-model="shifted" type="checkbox" /> Shift guide panel</label>
    </div>
    <section
      v-if="visible"
      ref="panel"
      class="measurement-panel"
      :class="{ wide, shifted }"
      aria-label="Measured guide panel"
    >
      <strong>Review the actual panel</strong>
      <p>Geometry belongs to this viewport.</p>
    </section>
    <div class="example-actions">
      <button type="button" :disabled="stopped" @click="() => update()">
        Measure current rectangle
      </button>
      <button type="button" :disabled="stopped" @click="stopObservation">
        Stop rectangle observation
      </button>
    </div>
    <output aria-live="polite">
      {{ stopped ? "Last recorded" : "Measured" }} rectangle: left {{ Math.round(left) }}, top
      {{ Math.round(top) }}, width {{ Math.round(width) }}, height {{ Math.round(height) }} CSS
      pixels
    </output>
    <p>
      Size and scroll changes refresh the native border rectangle automatically. A CSS transform
      changes position without resizing: use Measure current rectangle to refresh it explicitly.
      Hiding the observed panel resets its measurements to zero while observation is active.
    </p>
    <p v-if="stopped">
      Observation stopped. Native panel controls still work; the last recorded rectangle stays
      fixed.
    </p>
  </div>
</template>

<style scoped>
.measurement-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;

  label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  input {
    inline-size: auto;
  }
}
.measurement-panel {
  box-sizing: border-box;
  inline-size: 160px;
  max-inline-size: calc(100% - 40px);
  padding: 12px;
  border: 1px solid currentColor;
  border-radius: 8px;

  &.wide {
    inline-size: 230px;
  }

  &.shifted {
    transform: translateX(40px);
  }
}
</style>
