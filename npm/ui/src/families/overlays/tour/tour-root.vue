<script setup lang="ts" generic="TStep extends TourStepDefinition">
import {
  computed,
  onBeforeUnmount,
  onMounted,
  shallowReadonly,
  shallowRef,
  toValue,
  watch,
} from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import type { Placement } from "../positioner/positioner.ts";
import { tourContext } from "./tour-context.ts";
import type { TourContextValue } from "./tour-context.ts";
import {
  enabledTourSteps,
  findTourStep,
  resolveTourMessages,
  resolveTourTarget,
  runTourHooks,
} from "./tour-state.ts";
import type {
  TourAfterLeave,
  TourBeforeEnter,
  TourBeforeEnterContext,
  TourDirection,
  TourDismissReason,
  TourMessages,
  TourMissingTargetBehavior,
  TourNavigationDirection,
  TourRootExpose,
  TourSlotState,
  TourState,
  TourStepDefinition,
  TourTargetState,
} from "./tour-types.ts";

type StepValue = TStep["value"];

const TARGET_ATTRIBUTE = "data-vize-tour-target";
const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

const {
  steps,
  id = undefined,
  open = undefined,
  defaultOpen = false,
  step = undefined,
  defaultStep = undefined,
  placement = "bottom",
  missingTarget = "center",
  scrollIntoView = true,
  markTarget = true,
  keyboardNavigation = true,
  dir = "ltr",
  beforeEnter = undefined,
  afterLeave = undefined,
  messages = undefined,
} = defineProps<{
  /**
   * Ordered step definitions. The step type, including consumer-owned fields, is inferred and
   * returned through slot state and `v-model:step`.
   *
   * @default required
   */
  readonly steps: readonly TStep[];

  /**
   * Consumer-owned Tour base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled open state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Controlled current step value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly step?: StepValue | null;

  /**
   * Initial step for uncontrolled use. `undefined` selects the first enabled step.
   *
   * @default undefined
   */
  readonly defaultStep?: StepValue;

  /**
   * Placement used by steps that do not declare their own.
   *
   * @default "bottom"
   */
  readonly placement?: Placement;

  /**
   * Behavior when a declared step target is not found on the client. `skip` continues in the
   * navigation direction; `center` keeps the step and centers the content.
   *
   * @default "center"
   */
  readonly missingTarget?: TourMissingTargetBehavior;

  /**
   * Scroll each resolved target into view. Motion is instant under `prefers-reduced-motion`.
   *
   * @default true
   */
  readonly scrollIntoView?: boolean;

  /**
   * Mark the current target with `data-vize-tour-target="active"` for consumer highlighting.
   *
   * @default true
   */
  readonly markTarget?: boolean;

  /**
   * Let ArrowLeft and ArrowRight inside TourContent move between steps.
   *
   * @default true
   */
  readonly keyboardNavigation?: boolean;

  /**
   * Reading direction used to map horizontal arrow keys.
   *
   * @default "ltr"
   */
  readonly dir?: TourDirection;

  /**
   * Hook run before any step becomes current, before the step's own `beforeEnter`. Receives the
   * inferred step type. Resolving `false` cancels; rejecting emits `navigation-error`. While it is
   * pending the root publishes `data-pending` and TourPrev/TourNext are disabled; a newer request
   * aborts the pending one through `signal`.
   *
   * @default undefined
   */
  readonly beforeEnter?: TourBeforeEnter<TStep>;

  /**
   * Hook run after a step stops being current, including when the tour closes.
   *
   * @default undefined
   */
  readonly afterLeave?: TourAfterLeave<TStep>;

  /**
   * Overrides for default strings rendered by Tour parts.
   *
   * @default undefined
   */
  readonly messages?: Partial<TourMessages>;
}>();

const emit = defineEmits<{
  /** Fired when the tour requests a controlled open value. */
  "update:open": [value: boolean];

  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];

  /** Fired when the tour requests a controlled step value. */
  "update:step": [value: StepValue];

  /** Fired after any distinct step request. */
  "step-change": [value: StepValue, previous: StepValue | null, nativeEvent: Event | null];

  /** Fired when TourNext is activated on the last step or `complete()` is called. */
  complete: [nativeEvent: Event | null];

  /** Fired when an open tour closes without completing. */
  dismiss: [reason: TourDismissReason, nativeEvent: Event | null];

  /** Fired when a `beforeEnter` hook resolves `false`; the current step is kept. */
  "navigation-cancel": [value: StepValue, from: StepValue | null];

  /** Fired when a `beforeEnter` hook throws or rejects; the current step is kept. */
  "navigation-error": [error: unknown, value: StepValue, from: StepValue | null];
}>();

defineSlots<{
  /** Compound Tour children. Receives the inferred current step and progress. */
  default?(props: TourSlotState<TStep>): unknown;
}>();

const baseId = useDeterministicId({ id: () => id, hint: "tour" });
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const titleId = computed(() => deriveDeterministicId(baseId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(baseId.value, "description"));
const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const stepState = useControllableState<StepValue | null | undefined>({
  value: () => step,
  defaultValue: () => defaultStep,
  equals: Object.is,
});
const enabledSteps = computed<readonly TStep[]>(() => enabledTourSteps(steps));
const currentStep = computed<TStep | null>(() => {
  const configured = stepState.value.value;
  const match =
    configured === undefined || configured === null
      ? undefined
      : enabledSteps.value.find((candidate) => candidate.value === configured);
  if (match !== undefined) return match;
  if (stepState.controlled.value) return null;
  return enabledSteps.value[0] ?? null;
});
const isOpen = computed(() => openState.value.value);
const state = computed<TourState>(() => (isOpen.value ? "open" : "closed"));
const index = computed(() => {
  const current = readCurrentStep();
  return current === null ? -1 : enabledSteps.value.indexOf(current);
});
const total = computed(() => enabledSteps.value.length);
const first = computed(() => index.value === 0);
const last = computed(() => index.value >= 0 && index.value === total.value - 1);
const placementState = computed<Placement>(() => currentStep.value?.placement ?? placement);
const dirState = computed<TourDirection>(() => dir);
const keyboardNavigationState = computed(() => keyboardNavigation);
const messagesState = computed<TourMessages>(() => resolveTourMessages(messages));
const pending = shallowRef(false);
const targetElement = shallowRef<Element | null>(null);
const resolution = shallowRef<{ readonly value: string; readonly found: boolean } | null>(null);
const targetState = computed<TourTargetState>(() => {
  const current = readCurrentStep();
  if (current === null || current.target === undefined) return "none";
  const resolved = readResolution();
  if (resolved === null || resolved.value !== current.value) return "pending";
  return resolved.found ? "resolved" : "missing";
});
const slotState = computed<TourSlotState<TStep>>(() => ({
  first: first.value,
  index: index.value,
  last: last.value,
  open: isOpen.value,
  state: state.value,
  step: currentStep.value,
  pending: pending.value,
  targetState: targetState.value,
  total: total.value,
  value: currentStep.value?.value ?? null,
}));
let mounted = false;
let direction: 1 | -1 = 1;
let navigation: AbortController | null = null;

function readCurrentStep(): TStep | null {
  return currentStep.value;
}

function readResolution(): { readonly value: string; readonly found: boolean } | null {
  return resolution.value;
}

function readOpen(): boolean {
  return isOpen.value;
}

function readTarget(): Element | null {
  return targetElement.value;
}

function policyOf(candidate: TStep): TourMissingTargetBehavior {
  return candidate.missingTarget ?? missingTarget;
}

function resolve(candidate: TStep): Element | null {
  return resolveTourTarget(candidate.target, typeof document === "undefined" ? null : document);
}

function hooksFor(candidate: TStep): readonly TourBeforeEnter<TStep>[] {
  const hooks: TourBeforeEnter<TStep>[] = [];
  if (beforeEnter !== undefined) hooks.push(beforeEnter);
  if (candidate.beforeEnter !== undefined) hooks.push(candidate.beforeEnter);
  return hooks;
}

function canEnter(candidate: TStep): boolean {
  if (candidate.target === undefined || policyOf(candidate) !== "skip") return true;
  // A hook may render the target, so the missing-target check waits until it settles.
  if (hooksFor(candidate).length > 0) return true;
  if (typeof document === "undefined") return true;
  return resolve(candidate) !== null;
}

function indexOf(value: string): number {
  return enabledSteps.value.findIndex((candidate) => candidate.value === value);
}

function commitOpen(value: boolean, nativeEvent: Event | null): boolean {
  const previous = readOpen();
  const changed = openState.set(value);
  if (changed || previous !== value) {
    emit("update:open", value);
    emit("open-change", value, previous, nativeEvent);
    return true;
  }
  return false;
}

function leave(left: TStep | null, entered: TStep | null): void {
  if (left === null || left === entered || afterLeave === undefined) return;
  afterLeave({ step: left, to: entered });
}

function commitStep(value: StepValue, nativeEvent: Event | null): boolean {
  const left = readCurrentStep();
  const previous = left?.value ?? null;
  if (previous === value) return false;
  stepState.set(value);
  emit("update:step", value);
  emit("step-change", value, previous, nativeEvent);
  if (readOpen()) leave(left, readCurrentStep());
  return true;
}

function abortNavigation(): void {
  navigation?.abort();
  navigation = null;
  pending.value = false;
}

function evaluateHooks(
  hooks: readonly TourBeforeEnter<TStep>[],
  context: TourBeforeEnterContext<TStep>,
): boolean | Promise<boolean> | { readonly error: unknown } {
  try {
    return runTourHooks(hooks, context);
  } catch (error) {
    return { error };
  }
}

/**
 * Enter `candidate`, running `beforeEnter` hooks first. Synchronous hooks settle in the same
 * call; asynchronous ones leave the request pending until they resolve, and the latest request
 * wins. Reports whether the request was accepted (committed or pending).
 */
function requestStep(candidate: TStep, opening: boolean, nativeEvent: Event | null): boolean {
  abortNavigation();
  const from = opening && !readOpen() ? null : readCurrentStep();
  const commit = (): boolean => {
    const stepChanged = commitStep(candidate.value, nativeEvent);
    const openChanged = opening ? commitOpen(true, nativeEvent) : false;
    // Hooks may have rendered the target after the step value was already current.
    if (!stepChanged && !openChanged) syncTarget();
    return stepChanged || openChanged || opening;
  };
  const hooks = hooksFor(candidate);
  if (hooks.length === 0) return commit();
  const controller = new AbortController();
  const navigationDirection: TourNavigationDirection = direction === 1 ? "forward" : "backward";
  const fromValue = from?.value ?? null;
  const settle = (outcome: "cancel" | "enter" | { readonly error: unknown }): boolean => {
    if (controller.signal.aborted) return false;
    if (navigation === controller) {
      navigation = null;
      pending.value = false;
    }
    if (outcome === "enter") return commit();
    if (outcome === "cancel") emit("navigation-cancel", candidate.value, fromValue);
    else emit("navigation-error", outcome.error, candidate.value, fromValue);
    return false;
  };
  const result = evaluateHooks(hooks, {
    direction: navigationDirection,
    from,
    signal: controller.signal,
    step: candidate,
  });
  if (typeof result === "boolean") return settle(result ? "enter" : "cancel");
  if (!(result instanceof Promise)) return settle(result);
  navigation = controller;
  pending.value = true;
  result.then(
    (entered) => settle(entered ? "enter" : "cancel"),
    (error: unknown) => settle({ error }),
  );
  return true;
}

function openFrom(start: number, nativeEvent: Event | null): boolean {
  const candidate = findTourStep(enabledSteps.value, start, 1, canEnter);
  if (candidate === null) return false;
  direction = 1;
  return requestStep(candidate, true, nativeEvent);
}

function start(nativeEvent: Event | null = null): boolean {
  return openFrom(0, nativeEvent);
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  if (!value) {
    if (!isOpen.value) return commitOpen(false, nativeEvent);
    close(nativeEvent);
    return true;
  }
  if (isOpen.value) return false;
  return openFrom(Math.max(index.value, 0), nativeEvent);
}

function close(nativeEvent: Event | null): void {
  abortNavigation();
  const left = readCurrentStep();
  commitOpen(false, nativeEvent);
  leave(left, null);
}

function complete(nativeEvent: Event | null = null): boolean {
  if (!isOpen.value) return false;
  close(nativeEvent);
  emit("complete", nativeEvent);
  return true;
}

function dismiss(reason: TourDismissReason = "close", nativeEvent: Event | null = null): boolean {
  if (!isOpen.value) return false;
  close(nativeEvent);
  emit("dismiss", reason, nativeEvent);
  return true;
}

function next(nativeEvent: Event | null = null): boolean {
  if (!isOpen.value) return false;
  direction = 1;
  const candidate = findTourStep(enabledSteps.value, index.value + 1, 1, canEnter);
  if (candidate === null) return complete(nativeEvent);
  return requestStep(candidate, false, nativeEvent);
}

function previous(nativeEvent: Event | null = null): boolean {
  if (!isOpen.value) return false;
  direction = -1;
  const candidate = findTourStep(enabledSteps.value, index.value - 1, -1, canEnter);
  return candidate === null ? false : requestStep(candidate, false, nativeEvent);
}

function goTo(value: StepValue, nativeEvent: Event | null = null): boolean {
  const target = indexOf(value);
  const candidate = enabledSteps.value[target];
  if (candidate === undefined) return false;
  direction = target >= index.value ? 1 : -1;
  if (candidate === readCurrentStep()) return false;
  return requestStep(candidate, false, nativeEvent);
}

function setTarget(element: Element | null): void {
  const previousElement = readTarget();
  if (previousElement === element) return;
  previousElement?.removeAttribute(TARGET_ATTRIBUTE);
  if (element !== null && markTarget) element.setAttribute(TARGET_ATTRIBUTE, "active");
  targetElement.value = element;
}

function reveal(element: Element): void {
  if (!scrollIntoView || typeof element.scrollIntoView !== "function") return;
  const reduced =
    typeof globalThis.matchMedia === "function" &&
    globalThis.matchMedia(REDUCED_MOTION_QUERY).matches;
  element.scrollIntoView({
    block: "center",
    inline: "nearest",
    behavior: reduced ? "auto" : "smooth",
  });
}

function skipMissing(current: TStep): boolean {
  const position = indexOf(current.value);
  const forward = findTourStep(enabledSteps.value, position + direction, direction, canEnter);
  const candidate =
    forward ??
    (direction === -1 ? findTourStep(enabledSteps.value, position + 1, 1, canEnter) : null);
  if (candidate !== null) return requestStep(candidate, false, null);
  return complete(null);
}

function syncTarget(): void {
  if (!mounted) return;
  const current = readCurrentStep();
  if (!isOpen.value || current === null) {
    setTarget(null);
    resolution.value = null;
    return;
  }
  const element = resolve(current);
  if (element === null && current.target !== undefined && policyOf(current) === "skip") {
    if (skipMissing(current)) return;
  }
  setTarget(element);
  resolution.value = { value: current.value, found: element !== null };
  if (element !== null) reveal(element);
}

function readTargetSource(): unknown {
  const target = currentStep.value?.target;
  return typeof target === "string" || target === undefined ? target : toValue(target);
}

watch([isOpen, () => currentStep.value?.value, readTargetSource], syncTarget, { flush: "post" });

watch(
  () => markTarget,
  (value) => {
    const element = readTarget();
    if (element === null) return;
    if (value) element.setAttribute(TARGET_ATTRIBUTE, "active");
    else element.removeAttribute(TARGET_ATTRIBUTE);
  },
);

onMounted(() => {
  mounted = true;
  syncTarget();
});

onBeforeUnmount(() => {
  mounted = false;
  abortNavigation();
  setTarget(null);
});

tourContext.provide({
  contentId,
  descriptionId,
  dir: dirState,
  dismiss,
  first,
  id: baseId,
  index,
  indexOf,
  keyboardNavigation: keyboardNavigationState,
  last,
  messages: messagesState,
  next,
  open: isOpen,
  pending: shallowReadonly(pending),
  placement: placementState,
  previous,
  slotState,
  state,
  step: currentStep,
  steps: enabledSteps,
  target: shallowReadonly(targetElement),
  targetState,
  titleId,
  total,
} satisfies TourContextValue);

type TourRootSetupExpose = Omit<
  TourRootExpose<TStep>,
  keyof TourSlotState<TStep> | "id" | "target"
> & {
  readonly first: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
  readonly index: ComputedRef<number>;
  readonly last: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly pending: Readonly<ShallowRef<boolean>>;
  readonly state: ComputedRef<TourState>;
  readonly step: ComputedRef<TStep | null>;
  readonly target: Readonly<ShallowRef<Element | null>>;
  readonly targetState: ComputedRef<TourTargetState>;
  readonly total: ComputedRef<number>;
  readonly value: ComputedRef<StepValue | null>;
};

const exposed = {
  complete,
  dismiss,
  first,
  goTo,
  id: baseId,
  index,
  last,
  next,
  open: isOpen,
  pending: shallowReadonly(pending),
  previous,
  refresh: syncTarget,
  setOpen,
  start,
  state,
  step: currentStep,
  target: shallowReadonly(targetElement),
  targetState,
  total,
  value: computed<StepValue | null>(() => currentStep.value?.value ?? null),
} satisfies TourRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="tour-root"
    part="root"
    :data-state="state"
    :data-step="currentStep?.value"
    :data-target="targetState"
    :data-pending="pending ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
