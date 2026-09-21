<script setup lang="ts">
import { computed } from "vue";
import type { TimelineStep } from "./ladder";

const props = defineProps<{
  steps: TimelineStep[];
  /** Key of the page on screen, to mark its step. */
  current: string | null;
}>();

const emit = defineEmits<{
  select: [TimelineStep];
}>();

const passCount = computed(() => props.steps.filter((step) => !step.producer).length);
const changedPasses = computed(
  () => props.steps.filter((step) => !step.producer && step.changed).length,
);

function describe(step: TimelineStep): string {
  if (step.producer) return `${step.rung.toUpperCase()} ${step.pass}: produces a new artifact`;
  return step.changed
    ? `${step.pass} changed the ${step.rung.toUpperCase()} folio`
    : `${step.pass} left the ${step.rung.toUpperCase()} folio unchanged (its product is facts)`;
}
</script>

<template>
  <div class="davinci-timeline">
    <ol class="davinci-steps" aria-label="Pipeline steps in run order">
      <li v-for="step in steps" :key="step.key">
        <button
          type="button"
          :class="[
            'davinci-step',
            `rung-${step.rung}`,
            {
              producer: step.producer,
              changed: step.changed,
              current: current === step.key,
            },
          ]"
          :title="describe(step)"
          @click="emit('select', step)"
        >
          <span class="davinci-step-mark" aria-hidden="true"></span>
          <span class="davinci-step-rung">{{ step.rung.toUpperCase() }}</span>
          <span class="davinci-step-pass">{{ step.pass }}</span>
        </button>
      </li>
    </ol>
    <p class="davinci-timeline-summary">
      {{ changedPasses }} of {{ passCount }} {{ passCount === 1 ? "pass" : "passes" }} changed the
      folio
    </p>
  </div>
</template>
