<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import {
  PROGRESS_BAR_DEFAULT_MAX,
  PROGRESS_BAR_DEFAULT_MIN,
  getProgressBarState,
  getProgressBarStyle,
} from "./progress-bar-state.ts";
import type {
  ProgressBarDirection,
  ProgressBarExpose,
  ProgressBarSlotState,
  ProgressBarState,
} from "./progress-bar-types.ts";

const {
  as = "div",
  id = undefined,
  value = null,
  min = PROGRESS_BAR_DEFAULT_MIN,
  max = PROGRESS_BAR_DEFAULT_MAX,
  dir = "ltr",
  label = undefined,
  valueLabel = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaValueText = undefined,
} = defineProps<{
  /** Native element, custom element, or component rendered as the root. @default "div" */
  readonly as?: PrimitiveAs;

  /** Consumer-owned progressbar id. @default undefined */
  readonly id?: string | null;

  /** Current determinate value. `null` and `undefined` render indeterminate. @default null */
  readonly value?: number | null;

  /** Lower bound for the progress range. @default 0 */
  readonly min?: number | null;

  /** Upper bound for the progress range. @default 100 */
  readonly max?: number | null;

  /** Reading direction for inline-start fill and indeterminate motion. @default "ltr" */
  readonly dir?: ProgressBarDirection;

  /** Optional visible label rendered in the label part. @default undefined */
  readonly label?: string;

  /** Optional visible value text reused as `aria-valuetext`. @default undefined */
  readonly valueLabel?: string;

  /** Accessible name when no visible label or `aria-labelledby` supplies one. @default undefined */
  readonly ariaLabel?: string;

  /** Space-separated ids that label the progressbar. @default undefined */
  readonly ariaLabelledby?: string;

  /** Space-separated ids that describe the progressbar. @default undefined */
  readonly ariaDescribedby?: string;

  /** Human-readable assistive value text, overriding `valueLabel`. @default undefined */
  readonly ariaValueText?: string;
}>();

const slots = defineSlots<{
  /** Render consumer-owned content inside the root with normalized progress state. */
  default?(props: ProgressBarSlotState): unknown;

  /** Render a visible label inside the deterministic label node. */
  label?(props: ProgressBarSlotState): unknown;

  /** Render visible value text inside the deterministic value node. */
  value?(props: ProgressBarSlotState): unknown;

  /** Render optional content inside the indicator part. */
  indicator?(props: ProgressBarSlotState): unknown;
}>();

const root = useTemplateRef<ProgressBarExpose["root"]>("root");
const track = useTemplateRef<HTMLSpanElement>("track");
const indicator = useTemplateRef<HTMLSpanElement>("indicator");
const controlId = useDeterministicId({ id: () => id, hint: "progress" });
const labelId = computed(() => deriveDeterministicId(controlId.value, "label"));
const valueId = computed(() => deriveDeterministicId(controlId.value, "value"));
const progress = computed(() => getProgressBarState({ value, min, max, dir }));
const currentValue = computed(() => progress.value.value);
const currentMin = computed(() => progress.value.min);
const currentMax = computed(() => progress.value.max);
const percent = computed(() => progress.value.percent);
const ratio = computed(() => progress.value.ratio);
const direction = computed<ProgressBarDirection>(() => progress.value.dir);
const indeterminate = computed(() => progress.value.indeterminate);
const complete = computed(() => progress.value.complete);
const invalid = computed(() => progress.value.invalid);
const state = computed<ProgressBarState>(() => progress.value.state);
const progressStyle = computed(() => getProgressBarStyle(progress.value));
const rootIntrinsicProps = computed(() => ({ style: progressStyle.value }));
const hasLabel = computed(() => hasText(label) || slots.label !== undefined);
const hasValue = computed(() => hasText(valueLabel) || slots.value !== undefined);
const labelledby = computed(() => {
  if (hasText(ariaLabelledby)) return ariaLabelledby;
  return hasLabel.value ? labelId.value : undefined;
});
const accessibleLabel = computed(() => (labelledby.value === undefined ? ariaLabel : undefined));
const labelled = computed(() => hasText(accessibleLabel.value) || labelledby.value !== undefined);
const valueText = computed(() => normalizeText(ariaValueText) ?? normalizeText(valueLabel));
const slotState = computed<ProgressBarSlotState>(() => ({
  complete: complete.value,
  dir: direction.value,
  id: controlId.value,
  indeterminate: indeterminate.value,
  invalid: invalid.value,
  labelId: labelId.value,
  max: currentMax.value,
  min: currentMin.value,
  percent: percent.value,
  ratio: ratio.value,
  state: state.value,
  style: progressStyle.value,
  value: currentValue.value,
  valueId: valueId.value,
}));

function hasText(text: string | undefined): boolean {
  return normalizeText(text) !== undefined;
}

function normalizeText(text: string | undefined): string | undefined {
  if (text === undefined) return undefined;
  const normalized = text.replaceAll(/\s+/g, " ").trim();
  return normalized.length === 0 ? undefined : normalized;
}

function focus(options?: FocusOptions): void {
  focusTarget(root.value, options);
}

function focusTarget(target: unknown, options?: FocusOptions): void {
  if (
    typeof target === "object" &&
    target !== null &&
    "focus" in target &&
    typeof target.focus === "function"
  ) {
    target.focus(options);
  }
}

type ProgressSetupExpose = {
  readonly root: typeof root;
  readonly track: typeof track;
  readonly indicator: typeof indicator;
  readonly value: ComputedRef<ProgressBarExpose["value"]>;
  readonly min: ComputedRef<ProgressBarExpose["min"]>;
  readonly max: ComputedRef<ProgressBarExpose["max"]>;
  readonly percent: ComputedRef<ProgressBarExpose["percent"]>;
  readonly ratio: ComputedRef<ProgressBarExpose["ratio"]>;
  readonly dir: ComputedRef<ProgressBarExpose["dir"]>;
  readonly indeterminate: ComputedRef<ProgressBarExpose["indeterminate"]>;
  readonly complete: ComputedRef<ProgressBarExpose["complete"]>;
  readonly invalid: ComputedRef<ProgressBarExpose["invalid"]>;
  readonly state: ComputedRef<ProgressBarExpose["state"]>;
  readonly id: ComputedRef<ProgressBarExpose["id"]>;
  readonly labelId: ComputedRef<ProgressBarExpose["labelId"]>;
  readonly valueId: ComputedRef<ProgressBarExpose["valueId"]>;
  readonly style: ComputedRef<ProgressBarExpose["style"]>;
  readonly focus: ProgressBarExpose["focus"];
};

const exposed = {
  root,
  track,
  indicator,
  value: currentValue,
  min: currentMin,
  max: currentMax,
  percent,
  ratio,
  dir: direction,
  indeterminate,
  complete,
  invalid,
  state,
  id: controlId,
  labelId,
  valueId,
  style: progressStyle,
  focus,
} satisfies ProgressSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="controlId"
    ref="root"
    v-bind="rootIntrinsicProps"
    role="progressbar"
    :dir="direction"
    :aria-label="accessibleLabel"
    :aria-labelledby="labelledby"
    :aria-describedby="ariaDescribedby"
    :aria-valuemin="currentMin"
    :aria-valuemax="currentMax"
    :aria-valuenow="currentValue ?? undefined"
    :aria-valuetext="valueText"
    data-vize-ui="progress-bar"
    part="root"
    :data-dir="direction"
    :data-state="state"
    :data-labelled="labelled ? 'true' : 'false'"
    :data-indeterminate="indeterminate ? 'true' : 'false'"
    :data-complete="complete ? 'true' : 'false'"
    :data-invalid="invalid ? 'true' : undefined"
    :data-value="currentValue ?? undefined"
    :data-min="currentMin"
    :data-max="currentMax"
    :data-percent="percent ?? undefined"
  >
    <span v-if="hasLabel" :id="labelId" data-vize-ui="progress-bar-label" part="label">
      <slot name="label" v-bind="slotState">
        {{ label }}
      </slot>
    </span>
    <span ref="track" data-vize-ui="progress-bar-track" part="track">
      <span ref="indicator" data-vize-ui="progress-bar-indicator" part="indicator">
        <slot name="indicator" v-bind="slotState" />
      </span>
    </span>
    <span v-if="hasValue" :id="valueId" data-vize-ui="progress-bar-value" part="value">
      <slot name="value" v-bind="slotState">
        {{ valueLabel }}
      </slot>
    </span>
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Structural styles live in progress-bar.css so root and subpath bundles stay byte-identical. */
</style>
