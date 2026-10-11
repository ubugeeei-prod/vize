<script setup lang="ts">
import { computed, ref, useId, useTemplateRef } from "vue";
import { useResizeObserver } from "@vizejs/composable/resize-observer";

const panelsId = useId();
const first = useTemplateRef<HTMLElement>("first");
const second = useTemplateRef<HTMLElement>("second");
const firstWide = ref(true);
const secondWide = ref(false);
const selected = ref("first");
const target = computed(() => (selected.value === "first" ? first.value : second.value));
const observedWidth = ref(0);
const observedHeight = ref(0);
const observedName = ref("No panel observed yet");
const notifications = ref(0);
const stopped = ref(false);
const { isSupported, stop } = useResizeObserver(target, (entries) => {
  const entry = entries.find((candidate) => candidate.target === target.value);
  if (!entry) return;
  observedWidth.value = entry.contentRect.width;
  observedHeight.value = entry.contentRect.height;
  observedName.value = entry.target === first.value ? "First guide panel" : "Second guide panel";
  notifications.value += 1;
});

function stopObservation(): void {
  stop();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <label :for="`${panelsId}-target`">Observed guide panel</label>
    <select :id="`${panelsId}-target`" v-model="selected">
      <option value="first">First guide panel</option>
      <option value="second">Second guide panel</option>
    </select>
    <div class="example-actions">
      <label class="panel-size-choice"
        ><input v-model="firstWide" type="checkbox" /> Wide first panel</label
      >
      <label class="panel-size-choice"
        ><input v-model="secondWide" type="checkbox" /> Wide second panel</label
      >
    </div>
    <div class="guide-panels">
      <section
        ref="first"
        class="guide-panel"
        :class="{ wide: firstWide }"
        aria-label="First guide panel"
      >
        <strong>First guide panel</strong>
        <p>Compile a representative component.</p>
      </section>
      <section
        ref="second"
        class="guide-panel"
        :class="{ wide: secondWide }"
        aria-label="Second guide panel"
      >
        <strong>Second guide panel</strong>
        <p>Review native keyboard interaction.</p>
      </section>
    </div>
    <output aria-live="polite">
      Last observed: {{ observedName }} · Content box: {{ Math.round(observedWidth) }} ×
      {{ Math.round(observedHeight) }} CSS pixels · Notifications: {{ notifications }}
    </output>
    <button type="button" :disabled="stopped || !isSupported" @click="stopObservation">
      Stop size observation
    </button>
    <p v-if="stopped">
      Observation stopped. Both panels can still resize; the last delivered measurement is retained.
    </p>
    <p v-else>
      Only the selected panel updates the readout. Changing the selected target detaches the
      previous observer.
    </p>
    <p v-if="!isSupported">Resize observation is not available in this context.</p>
  </div>
</template>

<style scoped>
.panel-size-choice {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  input {
    inline-size: auto;
  }
}
.guide-panels {
  display: grid;
  gap: 12px;
  margin-block: 12px;
}
.guide-panel {
  box-sizing: border-box;
  inline-size: 180px;
  max-inline-size: 100%;
  padding: 12px;
  border: 1px solid currentColor;
  border-radius: 8px;
}
.guide-panel.wide {
  inline-size: 280px;
}
</style>
