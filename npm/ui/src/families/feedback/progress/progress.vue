<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { PROGRESS_DEFAULT_MAX, getProgressState } from "./progress-state.ts";
import type { ProgressExpose, ProgressSlotState } from "./progress-types.ts";

const {
  id = undefined,
  value = null,
  max = PROGRESS_DEFAULT_MAX,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaValueText = undefined,
} = defineProps<{
  /** Consumer-owned progressbar id. @default undefined */
  readonly id?: string | null;

  /** Current determinate value. `null`, `undefined`, and non-finite numbers render indeterminate. @default null */
  readonly value?: number | null;

  /** Positive maximum value. Non-positive and non-finite numbers fall back to 100. @default 100 */
  readonly max?: number | null;

  /** Accessible name when no visible label or `aria-labelledby` supplies one. @default undefined */
  readonly ariaLabel?: string;

  /** Space-separated ids that label the progressbar. @default undefined */
  readonly ariaLabelledby?: string;

  /** Space-separated ids that describe the progressbar. @default undefined */
  readonly ariaDescribedby?: string;

  /** Human-readable value text for assistive technology. @default undefined */
  readonly ariaValueText?: string;
}>();

defineSlots<{
  /** Optional fallback contents. Receives normalized Progress state for composition. */
  default?(props: ProgressSlotState): unknown;
}>();

const element = useTemplateRef<HTMLProgressElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "progress" });
const progress = computed(() => getProgressState({ value, max }));
const currentValue = computed(() => progress.value.value);
const currentMax = computed(() => progress.value.max);
const percent = computed(() => progress.value.percent);
const indeterminate = computed(() => progress.value.indeterminate);
const complete = computed(() => progress.value.complete);
const state = computed(() => progress.value.state);

type ProgressSetupExpose = {
  readonly element: typeof element;
  readonly value: ComputedRef<ProgressExpose["value"]>;
  readonly max: ComputedRef<ProgressExpose["max"]>;
  readonly percent: ComputedRef<ProgressExpose["percent"]>;
  readonly indeterminate: ComputedRef<ProgressExpose["indeterminate"]>;
  readonly complete: ComputedRef<ProgressExpose["complete"]>;
  readonly state: ComputedRef<ProgressExpose["state"]>;
};

const exposed = {
  element,
  value: currentValue,
  max: currentMax,
  percent,
  indeterminate,
  complete,
  state,
} satisfies ProgressSetupExpose;

defineExpose(exposed);
</script>

<template>
  <progress
    :id="controlId"
    ref="element"
    :value="currentValue ?? undefined"
    :max="currentMax"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-valuetext="ariaValueText"
    data-vize-ui="progress"
    part="root"
    :data-state="state"
    :data-indeterminate="indeterminate ? 'true' : 'false'"
    :data-complete="complete ? 'true' : 'false'"
    :data-value="currentValue ?? undefined"
    :data-max="currentMax"
    :data-percent="percent ?? undefined"
  >
    <slot
      :value="currentValue"
      :max="currentMax"
      :percent="percent"
      :indeterminate="indeterminate"
      :complete="complete"
      :state="state"
    />
  </progress>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
