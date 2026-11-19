<script setup lang="ts">
import type { TimelineStep } from "./ladder";
import { formatNanos } from "./format";

const { step, current, title } = defineProps<{
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
      'level-step',
      `rung-${step.rung}`,
      { producer: step.producer, changed: step.changed, current },
    ]"
    :title
    @click="() => emit('select')"
  >
    <span class="level-step-mark" aria-hidden="true"></span>
    <span class="level-step-rung">{{ step.rung.toUpperCase() }}</span>
    <span class="level-step-pass">{{ step.pass }}</span>
    <span v-if="step.nanos !== null" class="level-step-time">{{ formatNanos(step.nanos) }}</span>
    <span
      v-if="step.remarks > 0"
      class="level-step-remarks"
      :title="`${step.remarks} optimization ${step.remarks === 1 ? 'remark' : 'remarks'}`"
      >{{ step.remarks }}</span
    >
  </button>
</template>
