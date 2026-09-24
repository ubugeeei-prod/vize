<script setup lang="ts">
import type { TimelineStep } from "./ladder";
import { formatNanos } from "./format";

defineProps<{
  step: TimelineStep;
  current: boolean;
  /** What the step did, for the tooltip. */
  title: string;
}>();

const emit = defineEmits<{
  select: [];
}>();
</script>

<template>
  <button
    type="button"
    :class="[
      'davinci-step',
      `rung-${step.rung}`,
      { producer: step.producer, changed: step.changed, current },
    ]"
    :title="title"
    @click="() => emit('select')"
  >
    <span class="davinci-step-mark" aria-hidden="true"></span>
    <span class="davinci-step-rung">{{ step.rung.toUpperCase() }}</span>
    <span class="davinci-step-pass">{{ step.pass }}</span>
    <span v-if="step.nanos !== null" class="davinci-step-time">{{ formatNanos(step.nanos) }}</span>
    <span
      v-if="step.remarks > 0"
      class="davinci-step-remarks"
      :title="`${step.remarks} optimization ${step.remarks === 1 ? 'remark' : 'remarks'}`"
      >{{ step.remarks }}</span
    >
  </button>
</template>
