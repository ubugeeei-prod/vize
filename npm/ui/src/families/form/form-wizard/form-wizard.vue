<script setup lang="ts" generic="StepId extends string">
import { computed, nextTick, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { formWizardContext } from "./form-wizard-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type {
  FormWizardDirection,
  FormWizardDraftStore,
  FormWizardExpose,
  FormWizardSlotState,
  FormWizardState,
  FormWizardValidate,
} from "./form-wizard-types.ts";

const {
  steps,
  modelValue = undefined,
  defaultValue = undefined,
  validate = undefined,
  linear = true,
  draft = undefined,
  focusOnChange = true,
  id = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Step ids in order; their literal union types every step-related API.
   *
   * @default required
   */
  readonly steps: readonly StepId[];

  /**
   * Controlled current step. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: StepId;

  /**
   * Initial uncontrolled step.
   *
   * @default steps[0]
   */
  readonly defaultValue?: StepId;

  /**
   * Validation gate run before leaving a step forward and before completing.
   *
   * @default undefined
   */
  readonly validate?: FormWizardValidate<StepId>;

  /**
   * Only allow jumping forward to visited steps (or the next one).
   *
   * @default true
   */
  readonly linear?: boolean;

  /**
   * Persisted draft store for the current and visited steps.
   *
   * @default undefined
   */
  readonly draft?: FormWizardDraftStore<StepId>;

  /**
   * Move focus to the newly current step panel after a user transition.
   *
   * @default true
   */
  readonly focusOnChange?: boolean;

  /**
   * Base id for panels (`<id>-<step>`). `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Accessible name of the wizard region.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the wizard region.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired when the current step requests a change. */
  "update:modelValue": [step: StepId];

  /** Fired after the current step changes, with the previous step and direction. */
  change: [step: StepId, previous: StepId, direction: FormWizardDirection];

  /** Fired when a validation gate keeps the wizard on a step. */
  blocked: [step: StepId, target: StepId];

  /** Fired when the last step passes its gate. */
  complete: [];
}>();

defineSlots<{
  /** Renders step panels, navigation, and progress with the typed wizard state. */
  default(props: FormWizardSlotState<StepId>): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const baseId = useDeterministicId({ id: () => id, hint: "wizard" });
const panels = new Map<string, HTMLElement>();
const firstStep = computed<StepId>(() => {
  const step = steps[0];
  if (step === undefined) {
    throw new TypeError("VIZE_UI_FORM_WIZARD_STEPS: steps must contain at least one step id");
  }
  return step;
});

function toStep(candidate: string | undefined): StepId | undefined {
  return steps.find((step) => step === candidate);
}

const state = useControllableState<StepId>({
  value: () => (modelValue === undefined ? undefined : (toStep(modelValue) ?? firstStep.value)),
  defaultValue: () => toStep(defaultValue) ?? firstStep.value,
  onChange: (step) => emit("update:modelValue", step),
});
const visitedSteps = shallowRef<ReadonlySet<string>>(new Set());
const validating = shallowRef(false);
const completed = shallowRef(false);
let controller: AbortController | undefined;

const current = computed<StepId>(() => toStep(state.value.value) ?? firstStep.value);
const index = computed(() => Math.max(steps.indexOf(current.value), 0));
const count = computed(() => steps.length);
const progress = computed(() => (count.value <= 1 ? 1 : index.value / (count.value - 1)));
const visited = computed<readonly StepId[]>(() =>
  steps.filter((step, position) => position <= index.value || visitedSteps.value.has(step)),
);
const dataState = computed<FormWizardState>(() => {
  if (validating.value) return "validating";
  return completed.value ? "complete" : "in-progress";
});

function snapshot(): void {
  if (draft === undefined) return;
  draft.save({ step: current.value, visited: visited.value });
}

function markVisited(step: StepId): void {
  if (visitedSteps.value.has(step)) return;
  visitedSteps.value = new Set([...visitedSteps.value, step]);
}

function focusPanel(step: StepId): void {
  if (!focusOnChange) return;
  void nextTick(() => panels.get(step)?.focus({ preventScroll: false }));
}

function commit(target: StepId, direction: FormWizardDirection): boolean {
  if (current.value === target) return false;
  const previous = steps[index.value] ?? firstStep.value;
  markVisited(target);
  state.set(target);
  completed.value = false;
  emit("change", target, previous, direction);
  snapshot();
  focusPanel(target);
  return true;
}

async function passes(step: StepId, target: StepId): Promise<boolean> {
  if (validate === undefined) return true;
  controller?.abort();
  const active = new AbortController();
  controller = active;
  validating.value = true;
  try {
    const result = await validate({ step, target, signal: active.signal });
    if (active.signal.aborted) return false;
    if (!result) emit("blocked", step, target);
    return result;
  } catch {
    if (!active.signal.aborted) emit("blocked", step, target);
    return false;
  } finally {
    if (controller === active) {
      validating.value = false;
      controller = undefined;
    }
  }
}

async function next(): Promise<boolean> {
  if (validating.value) return false;
  const step = steps[index.value] ?? firstStep.value;
  const target = steps[index.value + 1];
  if (!(await passes(step, target ?? step))) return false;
  if (target === undefined) {
    completed.value = true;
    draft?.clear?.();
    emit("complete");
    return true;
  }
  return commit(target, "forward");
}

function back(): boolean {
  const target = steps[index.value - 1];
  if (target === undefined || validating.value) return false;
  controller?.abort();
  return commit(target, "back");
}

async function goTo(target: StepId): Promise<boolean> {
  const targetIndex = steps.indexOf(target);
  if (targetIndex === -1 || targetIndex === index.value || validating.value) return false;
  if (targetIndex < index.value) return commit(target, "back");
  if (linear && targetIndex > index.value + 1 && !visitedSteps.value.has(target)) return false;
  // Moving forward validates the current step and every step in between.
  for (let position = index.value; position < targetIndex; position += 1) {
    const step = steps[position];
    const following = steps[position + 1];
    if (step === undefined || following === undefined) return false;
    if (!(await passes(step, following))) {
      if (step !== current.value) commit(step, "forward");
      return false;
    }
  }
  return commit(target, "forward");
}

function reset(): void {
  controller?.abort();
  visitedSteps.value = new Set();
  completed.value = false;
  state.set(firstStep.value);
  draft?.clear?.();
}

// Drafts load after mount so the server and hydration render the default step.
watch(
  root,
  (element) => {
    if (element === null || draft === undefined) return;
    const saved = draft.load();
    const savedStep = toStep(saved?.step);
    if (saved === null || saved === undefined || savedStep === undefined) return;
    visitedSteps.value = new Set(saved.visited.flatMap((step) => toStep(step) ?? []));
    markVisited(savedStep);
    state.set(savedStep);
  },
  { flush: "post", once: true },
);

onScopeDispose(() => controller?.abort());

formWizardContext.provide({
  baseId,
  current: computed(() => current.value),
  index,
  count,
  progress,
  isFirst: computed(() => index.value === 0),
  isLast: computed(() => index.value === count.value - 1),
  validating: computed(() => validating.value),
  indexOf: (step: string) => steps.findIndex((candidate) => candidate === step),
  isVisited: (step: string) => visited.value.some((candidate) => candidate === step),
  registerPanel: (step: string, element: HTMLElement | null) => {
    if (element === null) panels.delete(step);
    else panels.set(step, element);
  },
  next,
  back,
  goToStep: (step: string) => {
    const target = toStep(step);
    return target === undefined ? Promise.resolve(false) : goTo(target);
  },
});

const slotState = computed<FormWizardSlotState<StepId>>(() => ({
  steps,
  current: current.value,
  index: index.value,
  count: count.value,
  progress: progress.value,
  visited: visited.value,
  isFirst: index.value === 0,
  isLast: index.value === count.value - 1,
  validating: validating.value,
  completed: completed.value,
  state: dataState.value,
}));

type FormWizardSetupExpose = Omit<FormWizardExpose<StepId>, keyof FormWizardSlotState<StepId>> & {
  readonly [Key in keyof FormWizardSlotState<StepId>]: ComputedRef<
    FormWizardSlotState<StepId>[Key]
  >;
} & { readonly root: typeof root };

function field<Key extends keyof FormWizardSlotState<StepId>>(
  key: Key,
): ComputedRef<FormWizardSlotState<StepId>[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  steps: field("steps"),
  current: field("current"),
  index: field("index"),
  count: field("count"),
  progress: field("progress"),
  visited: field("visited"),
  isFirst: field("isFirst"),
  isLast: field("isLast"),
  validating: field("validating"),
  completed: field("completed"),
  state: field("state"),
  root,
  next,
  back,
  goTo,
  reset,
} satisfies FormWizardSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="root"
    role="group"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-busy="validating ? 'true' : undefined"
    part="root"
    data-vize-ui="form-wizard"
    :data-state="dataState"
    :data-step="current"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
