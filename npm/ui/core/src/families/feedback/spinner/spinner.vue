<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type { PrimitiveAs } from "../../../primitive.ts";
import type {
  SpinnerAriaState,
  SpinnerElement,
  SpinnerExpose,
  SpinnerProgressState,
  SpinnerRole,
  SpinnerSlotState,
  SpinnerState,
} from "./spinner-types.ts";

const SPINNER_DEFAULT_MIN = 0;
const SPINNER_DEFAULT_MAX = 100;

const {
  as = "span",
  id = undefined,
  loading = true,
  visible = true,
  role = "status",
  value = null,
  min = SPINNER_DEFAULT_MIN,
  max = SPINNER_DEFAULT_MAX,
  atomic = true,
  ariaHidden = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaValueText = undefined,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "span"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer-owned spinner id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Whether the spinner represents pending work.
   *
   * @default true
   */
  readonly loading?: boolean;

  /**
   * Whether the spinner remains rendered and visible in layout.
   *
   * @default true
   */
  readonly visible?: boolean;

  /**
   * Accessibility semantics for the host.
   *
   * @default "status"
   */
  readonly role?: SpinnerRole;

  /**
   * Optional determinate progress value for `role="progressbar"`.
   *
   * @default null
   */
  readonly value?: number | null;

  /**
   * Lower progress bound for `role="progressbar"`.
   *
   * @default 0
   */
  readonly min?: number;

  /**
   * Upper progress bound for `role="progressbar"`.
   *
   * @default 100
   */
  readonly max?: number;

  /**
   * Whether status announcements should be atomic.
   *
   * @default true
   */
  readonly atomic?: boolean;

  /**
   * Hide the spinner from assistive technology and suppress status/progress semantics.
   *
   * @default undefined
   */
  readonly ariaHidden?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the spinner.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the spinner.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Human-readable progress value text for assistive technology.
   *
   * @default undefined
   */
  readonly ariaValueText?: string;
}>();

defineSlots<{
  /** Optional spinner contents. Receives current state for composition. */
  default(props: SpinnerSlotState): unknown;
}>();

const element = useTemplateRef<SpinnerElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "spinner" });
const loadingState = computed(() => loading);
const visibleState = computed(() => visible);
const ariaState = computed<SpinnerAriaState>(() => (ariaHidden === true ? "decorative" : role));
const currentMin = computed(() => (Number.isFinite(min) ? min : SPINNER_DEFAULT_MIN));
const currentMax = computed(() => normalizeMax(max, currentMin.value));
const currentValue = computed(() => normalizeValue(value, currentMin.value, currentMax.value));
const percent = computed(() =>
  currentValue.value === null
    ? null
    : ((currentValue.value - currentMin.value) / (currentMax.value - currentMin.value)) * 100,
);
const progressState = computed<SpinnerProgressState>(() => {
  if (ariaState.value !== "progressbar") return "none";
  return currentValue.value === null ? "indeterminate" : "determinate";
});
const complete = computed(
  () => progressState.value === "determinate" && currentValue.value === currentMax.value,
);
const state = computed<SpinnerState>(() => {
  if (!visibleState.value) return "hidden";
  if (!loadingState.value) return "idle";
  return complete.value && progressState.value === "determinate" ? "complete" : "loading";
});
const slotState = computed<SpinnerSlotState>(() => ({
  ariaState: ariaState.value,
  complete: complete.value,
  loading: loadingState.value,
  max: currentMax.value,
  min: currentMin.value,
  percent: percent.value,
  progressState: progressState.value,
  state: state.value,
  value: currentValue.value,
  visible: visibleState.value,
}));

type SpinnerSetupExpose = {
  readonly ariaState: ComputedRef<SpinnerExpose["ariaState"]>;
  readonly complete: ComputedRef<SpinnerExpose["complete"]>;
  readonly element: typeof element;
  readonly loading: ComputedRef<SpinnerExpose["loading"]>;
  readonly max: ComputedRef<SpinnerExpose["max"]>;
  readonly min: ComputedRef<SpinnerExpose["min"]>;
  readonly percent: ComputedRef<SpinnerExpose["percent"]>;
  readonly progressState: ComputedRef<SpinnerExpose["progressState"]>;
  readonly state: ComputedRef<SpinnerExpose["state"]>;
  readonly value: ComputedRef<SpinnerExpose["value"]>;
  readonly visible: ComputedRef<SpinnerExpose["visible"]>;
};

const exposed = {
  ariaState,
  complete,
  element,
  loading: loadingState,
  max: currentMax,
  min: currentMin,
  percent,
  progressState,
  state,
  value: currentValue,
  visible: visibleState,
} satisfies SpinnerSetupExpose;

defineExpose(exposed);

function normalizeMax(rawMax: number, normalizedMin: number): number {
  const candidate = Number.isFinite(rawMax) ? rawMax : SPINNER_DEFAULT_MAX;
  return candidate > normalizedMin ? candidate : normalizedMin + SPINNER_DEFAULT_MAX;
}

function normalizeValue(
  rawValue: number | null | undefined,
  normalizedMin: number,
  normalizedMax: number,
): number | null {
  if (rawValue == null || !Number.isFinite(rawValue)) return null;
  return Math.min(Math.max(rawValue, normalizedMin), normalizedMax);
}
</script>

<template>
  <component
    :is="as"
    :id="controlId"
    ref="element"
    :hidden="visible ? undefined : true"
    :role="ariaState === 'decorative' ? undefined : role"
    :aria-hidden="ariaState === 'decorative' ? 'true' : undefined"
    :aria-label="ariaState === 'decorative' ? undefined : ariaLabel"
    :aria-labelledby="ariaState === 'decorative' ? undefined : ariaLabelledby"
    :aria-describedby="ariaState === 'decorative' ? undefined : ariaDescribedby"
    :aria-live="ariaState === 'status' ? 'polite' : undefined"
    :aria-atomic="ariaState === 'status' ? (atomic ? 'true' : 'false') : undefined"
    :aria-valuemin="ariaState === 'progressbar' ? currentMin : undefined"
    :aria-valuemax="ariaState === 'progressbar' ? currentMax : undefined"
    :aria-valuenow="ariaState === 'progressbar' ? (currentValue ?? undefined) : undefined"
    :aria-valuetext="ariaState === 'progressbar' ? ariaValueText : undefined"
    data-vize-ui="spinner"
    part="root"
    :data-state="state"
    :data-loading="loading ? 'true' : 'false'"
    :data-visible="visible ? 'true' : 'false'"
    :data-aria-state="ariaState"
    :data-progress-state="progressState"
    :data-complete="complete ? 'true' : 'false'"
    :data-value="progressState === 'none' ? undefined : (currentValue ?? undefined)"
    :data-min="progressState === 'none' ? undefined : currentMin"
    :data-max="progressState === 'none' ? undefined : currentMax"
    :data-percent="progressState === 'determinate' ? percent : undefined"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
