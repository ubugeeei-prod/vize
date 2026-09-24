<script setup lang="ts">
import { computed } from "vue";

import { formWizardContext } from "./form-wizard-context.ts";

const { formatValueText = undefined, ariaLabel = "Progress" } = defineProps<{
  /**
   * Builds the announced value text from the 1-based step number and total.
   *
   * @default (step, count) => `Step ${step} of ${count}`
   */
  readonly formatValueText?: (step: number, count: number) => string;

  /**
   * Accessible name of the progress bar.
   *
   * @default "Progress"
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Optional fallback contents rendered inside the native progress element. */
  default?(props: {
    readonly step: number;
    readonly count: number;
    readonly progress: number;
  }): unknown;
}>();

const context = formWizardContext.use();
const step = computed(() => context.index.value + 1);
const valueText = computed(() =>
  (formatValueText ?? ((current: number, total: number) => `Step ${current} of ${total}`))(
    step.value,
    context.count.value,
  ),
);
</script>

<template>
  <progress
    :max="context.count.value"
    :value="step"
    :aria-label="ariaLabel"
    :aria-valuetext="valueText"
    part="progress"
    data-vize-ui="form-wizard-progress"
    :style="{ '--vize-form-wizard-progress': `${context.progress.value * 100}%` }"
  >
    <slot :step="step" :count="context.count.value" :progress="context.progress.value">{{
      valueText
    }}</slot>
  </progress>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
