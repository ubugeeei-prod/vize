<script setup lang="ts">
import { computed } from "vue";
import type { TimelineStep } from "./ladder";
import { formatNanos } from "./format";

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
const measured = computed(() => props.steps.filter((step) => step.nanos !== null));
const totalNanos = computed(() => measured.value.reduce((sum, step) => sum + (step.nanos ?? 0), 0));

function share(step: TimelineStep): string {
  const total = totalNanos.value;
  return total > 0 ? `${((step.nanos ?? 0) / total) * 100}%` : "0%";
}

function describe(step: TimelineStep): string {
  const rung = step.rung.toUpperCase();
  const time = step.nanos === null ? "" : `, ${formatNanos(step.nanos)}`;
  if (step.producer) return `${rung} ${step.pass}: produces a new artifact${time}`;
  return step.changed
    ? `${step.pass} changed the ${rung} folio${time}`
    : `${step.pass} left the ${rung} folio unchanged (its product is facts)${time}`;
}
</script>

<template>
  <div class="davinci-timeline">
    <div class="davinci-timeline-row">
      <div
        v-if="totalNanos > 0"
        class="davinci-time-strip"
        role="img"
        :aria-label="`Measured compiler work: ${formatNanos(totalNanos)} in total`"
      >
        <span
          v-for="step in measured"
          :key="step.key"
          :class="['davinci-time-segment', `rung-${step.rung}`, { current: current === step.key }]"
          :style="{ width: share(step) }"
          :title="describe(step)"
        ></span>
      </div>
      <p class="davinci-timeline-summary">
        {{ changedPasses }} of {{ passCount }} {{ passCount === 1 ? "pass" : "passes" }} changed the
        folio<template v-if="totalNanos > 0">, {{ formatNanos(totalNanos) }} total</template>
      </p>
    </div>
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
          <span v-if="step.nanos !== null" class="davinci-step-time">{{
            formatNanos(step.nanos)
          }}</span>
        </button>
      </li>
    </ol>
  </div>
</template>
