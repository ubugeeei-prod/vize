<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { useMouseInElement } from "@vizejs/composable/mouse-in-element";

const pad = useTemplateRef<HTMLElement>("pad");
const shifted = ref(false);
const stopped = ref(false);
const { x, y, elementX, elementY, elementWidth, elementHeight, isOutside, stop } =
  useMouseInElement(pad, { handleOutside: false });

function stopRelativeReadings(): void {
  stop();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <label class="coordinate-toggle"
      ><input v-model="shifted" type="checkbox" /> Shift coordinate pad</label
    >
    <section ref="pad" class="coordinate-pad" :class="{ shifted }" aria-label="Coordinate pad">
      <strong>Locate the pointer inside a guide</strong>
      <p>Move here, then outside the border.</p>
    </section>
    <output aria-live="polite">
      Relative: {{ Math.round(elementX) }}, {{ Math.round(elementY) }} · Outside:
      {{ isOutside ? "yes" : "no" }} · {{ stopped ? "Stopped" : "Observing" }}
    </output>
    <p class="mouse-client">Client: {{ Math.round(x) }}, {{ Math.round(y) }} CSS pixels</p>
    <p class="mouse-size">
      Recorded pad: {{ Math.round(elementWidth) }} × {{ Math.round(elementHeight) }} CSS pixels
    </p>
    <button type="button" :disabled="stopped" @click="stopRelativeReadings">
      Stop relative readings
    </button>
    <p>
      Each mouse move reads the current native rectangle, including transforms and scrolling. Shift
      the pad and move again to compare the new local coordinates. With handleOutside: false,
      leaving updates Outside but retains the last inside coordinates.
    </p>
    <p>
      Stop freezes relative coordinates, size and Outside. The underlying client mouse coordinates
      still update until this component's scope is disposed. Server rendering starts with zero
      geometry and Outside: yes; it has not measured a browser element.
    </p>
  </div>
</template>

<style scoped>
.coordinate-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;

  input {
    inline-size: auto;
  }
}
.coordinate-pad {
  box-sizing: border-box;
  inline-size: 180px;
  max-inline-size: calc(100% - 24px);
  min-block-size: 120px;
  padding: 12px;
  border: 1px solid currentColor;
  border-radius: 8px;

  &.shifted {
    transform: translateX(24px);
  }
}
</style>
