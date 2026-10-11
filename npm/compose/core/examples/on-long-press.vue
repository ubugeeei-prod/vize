<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { onLongPress } from "@vizejs/composable/on-long-press";

const reveal = useTemplateRef<HTMLButtonElement>("reveal");
const details = ref(false);
const deliveries = ref(0);
const pointer = ref("none");
const release = ref("No pointer release recorded.");
const stopped = ref(false);
const stop = onLongPress(
  reveal,
  (event) => {
    deliveries.value += 1;
    pointer.value = event.pointerType;
    details.value = true;
  },
  {
    delay: 500,
    distanceThreshold: 10,
    onRelease: (result) => {
      release.value = `Last pointer release: long press ${result.isLongPress ? "yes" : "no"} · ${Math.round(result.duration)} ms · ${Math.round(result.distance)} pixels.`;
    },
  },
);

function stopObservation(): void {
  stop();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <button ref="reveal" type="button" :aria-expanded="details" @click="() => (details = true)">
      Reveal review note
    </button>
    <p v-if="details">Check the deployed keyboard path before calling a fix delivered.</p>
    <div class="example-actions">
      <button type="button" @click="() => (details = false)">Hide review note</button>
      <button type="button" :disabled="stopped" @click="stopObservation">
        Stop long-press observation
      </button>
    </div>
    <output aria-live="polite"
      >Long-press deliveries: {{ deliveries }} · Last pointer: {{ pointer }}</output
    >
    <p>{{ release }}</p>
    <p>
      Hold Reveal review note for 500 ms to reveal it before release. Releasing early or moving more
      than 10 pixels cancels long-press delivery. An ordinary click, tap, or Enter on the same
      button independently reveals the note; the long-press count records only the held action.
    </p>
    <p v-if="stopped">
      Long-press observation stopped. The native reveal and hide controls still work.
    </p>
  </div>
</template>
