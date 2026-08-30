<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { METER_DEFAULT_MAX, METER_DEFAULT_MIN, getMeterState } from "./meter-state.ts";
import type { MeterExpose, MeterSlotState } from "./meter-types.ts";

const {
  id = undefined,
  value = METER_DEFAULT_MIN,
  min = METER_DEFAULT_MIN,
  max = METER_DEFAULT_MAX,
  low = null,
  high = null,
  optimum = null,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned meter id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Current value. Non-finite values are repaired to the normalized minimum.
   *
   * @default 0
   */
  readonly value?: number | null;

  /**
   * Lower bound.
   *
   * @default 0
   */
  readonly min?: number;

  /**
   * Upper bound. Values less than or equal to `min` are repaired to `min + 1`.
   *
   * @default 1
   */
  readonly max?: number;

  /**
   * Optional low threshold.
   *
   * @default null
   */
  readonly low?: number | null;

  /**
   * Optional high threshold.
   *
   * @default null
   */
  readonly high?: number | null;

  /**
   * Optional optimum threshold.
   *
   * @default null
   */
  readonly optimum?: number | null;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the meter.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the meter.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Optional fallback contents. Receives normalized Meter state for composition. */
  default(props: MeterSlotState): unknown;
}>();

const element = useTemplateRef<HTMLMeterElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "meter" });
const meter = computed(() => getMeterState({ value, min, max, low, high, optimum }));
const currentValue = computed(() => meter.value.value);
const currentMin = computed(() => meter.value.min);
const currentMax = computed(() => meter.value.max);
const currentLow = computed(() => meter.value.low);
const currentHigh = computed(() => meter.value.high);
const currentOptimum = computed(() => meter.value.optimum);
const percent = computed(() => meter.value.percent);
const range = computed(() => meter.value.range);
const optimal = computed(() => meter.value.optimal);
const invalid = computed(() => meter.value.invalid);
const state = computed(() => meter.value.state);

type MeterSetupExpose = {
  readonly element: typeof element;
  readonly value: ComputedRef<MeterExpose["value"]>;
  readonly min: ComputedRef<MeterExpose["min"]>;
  readonly max: ComputedRef<MeterExpose["max"]>;
  readonly low: ComputedRef<MeterExpose["low"]>;
  readonly high: ComputedRef<MeterExpose["high"]>;
  readonly optimum: ComputedRef<MeterExpose["optimum"]>;
  readonly percent: ComputedRef<MeterExpose["percent"]>;
  readonly range: ComputedRef<MeterExpose["range"]>;
  readonly optimal: ComputedRef<MeterExpose["optimal"]>;
  readonly invalid: ComputedRef<MeterExpose["invalid"]>;
  readonly state: ComputedRef<MeterExpose["state"]>;
};

const exposed = {
  element,
  value: currentValue,
  min: currentMin,
  max: currentMax,
  low: currentLow,
  high: currentHigh,
  optimum: currentOptimum,
  percent,
  range,
  optimal,
  invalid,
  state,
} satisfies MeterSetupExpose;

defineExpose(exposed);
</script>

<template>
  <meter
    :id="controlId"
    ref="element"
    :min="currentMin"
    :max="currentMax"
    :value="currentValue"
    :low="currentLow ?? undefined"
    :high="currentHigh ?? undefined"
    :optimum="currentOptimum ?? undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="meter"
    part="root"
    :data-state="state"
    :data-range="range"
    :data-optimal="optimal ? 'true' : 'false'"
    :data-invalid="invalid ? 'true' : undefined"
    :data-value="currentValue"
    :data-min="currentMin"
    :data-max="currentMax"
    :data-low="currentLow ?? undefined"
    :data-high="currentHigh ?? undefined"
    :data-optimum="currentOptimum ?? undefined"
    :data-percent="percent"
  >
    <slot
      :value="currentValue"
      :min="currentMin"
      :max="currentMax"
      :low="currentLow"
      :high="currentHigh"
      :optimum="currentOptimum"
      :percent="percent"
      :range="range"
      :optimal="optimal"
      :invalid="invalid"
      :state="state"
    />
  </meter>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
