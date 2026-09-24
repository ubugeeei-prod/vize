import { computed, shallowRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

/** Step values keyed by step name. */
export type StepValues<Name extends string> = { readonly [Step in Name]: unknown };

/** Stepper state and navigation returned by {@link useStepper}. */
export interface Stepper<Name extends string, Values extends StepValues<Name>> {
  /** Step names in order. */
  readonly stepNames: readonly Name[];

  /** Zero-based index of the current step. */
  readonly index: Readonly<ShallowRef<number>>;

  /** Name of the current step. */
  readonly current: ComputedRef<Name>;

  /** Value of the current step (the name itself for array steps). */
  readonly currentValue: ComputedRef<Values[Name]>;

  /** Name of the next step, or `undefined` on the last. */
  readonly next: ComputedRef<Name | undefined>;

  /** Name of the previous step, or `undefined` on the first. */
  readonly previous: ComputedRef<Name | undefined>;

  /** Whether the current step is the first. */
  readonly isFirst: ComputedRef<boolean>;

  /** Whether the current step is the last. */
  readonly isLast: ComputedRef<boolean>;

  /** Step name at `index`, or `undefined`. */
  readonly at: (index: number) => Name | undefined;

  /** Value of a step. */
  readonly get: <Step extends Name>(step: Step) => Values[Step];

  /** Jump to a step. */
  readonly goTo: (step: Name) => void;

  /** Advance one step (no-op on the last). */
  readonly goToNext: () => void;

  /** Go back one step (no-op on the first). */
  readonly goToPrevious: () => void;

  /**
   * Go back to an earlier step; later or current steps are ignored.
   *
   * @returns Whether the cursor moved.
   */
  readonly goBackTo: (step: Name) => boolean;

  /** Whether `step` is the current step. */
  readonly isCurrent: (step: Name) => boolean;

  /** Whether `step` is the step right after the current one. */
  readonly isNext: (step: Name) => boolean;

  /** Whether `step` is the step right before the current one. */
  readonly isPrevious: (step: Name) => boolean;

  /** Whether `step` comes before the current step. */
  readonly isBefore: (step: Name) => boolean;

  /** Whether `step` comes after the current step. */
  readonly isAfter: (step: Name) => boolean;
}

/**
 * Linear multi-step flow (wizards, onboarding, checkout) with typed step
 * names.
 *
 * Pass a tuple of names (`["account", "billing", "review"] as const`, or a
 * plain array literal thanks to `const` inference) or an object whose keys
 * are the names and whose values carry per-step data; every navigation
 * method only accepts declared names. Synchronous state: SSR-safe and
 * nothing to dispose.
 *
 * @example
 * ```ts
 * const wizard = useStepper(["account", "billing", "review"]);
 * wizard.goTo("billing");
 * wizard.goTo("shipping"); // type error
 * ```
 *
 * @param steps Step names, or step values keyed by name.
 * @param initial Step to start on.
 * @default initial the first step
 * @throws `RangeError` tagged `VIZE_COMPOSE_STEPPER_EMPTY` when there are no
 * steps, and `RangeError` tagged `VIZE_COMPOSE_STEPPER_UNKNOWN_STEP` for an
 * unknown step name at runtime.
 * @returns Stepper state and navigation.
 */
export function useStepper<const Steps extends readonly string[]>(
  steps: Steps,
  initial?: NoInfer<Steps[number]>,
): Stepper<Steps[number], { readonly [Step in Steps[number]]: Step }>;
export function useStepper<Steps extends Readonly<Record<string, unknown>>>(
  steps: Steps,
  initial?: NoInfer<keyof Steps & string>,
): Stepper<keyof Steps & string, Steps>;
export function useStepper(
  steps: readonly string[] | Readonly<Record<string, unknown>>,
  initial?: string,
): Stepper<string, Readonly<Record<string, unknown>>> {
  const names: readonly string[] = isNameList(steps) ? [...steps] : Object.keys(steps);
  const values: Readonly<Record<string, unknown>> = isNameList(steps)
    ? Object.fromEntries(steps.map((name) => [name, name]))
    : steps;
  if (names.length === 0) {
    throw new RangeError("[VIZE_COMPOSE_STEPPER_EMPTY] useStepper needs at least one step");
  }

  const indexOf = (step: string): number => {
    const found = names.indexOf(step);
    if (found === -1) {
      throw new RangeError(`[VIZE_COMPOSE_STEPPER_UNKNOWN_STEP] unknown step "${step}"`);
    }
    return found;
  };

  const index = shallowRef(initial === undefined ? 0 : indexOf(initial));
  const at = (position: number): string | undefined => names[position];
  const current = computed(() => names[index.value] ?? "");

  return {
    stepNames: names,
    index,
    current,
    currentValue: computed(() => values[current.value]),
    next: computed(() => at(index.value + 1)),
    previous: computed(() => at(index.value - 1)),
    isFirst: computed(() => index.value === 0),
    isLast: computed(() => index.value === names.length - 1),
    at,
    get: (step) => values[step],
    goTo: (step) => {
      index.value = indexOf(step);
    },
    goToNext: () => {
      if (index.value < names.length - 1) index.value += 1;
    },
    goToPrevious: () => {
      if (index.value > 0) index.value -= 1;
    },
    goBackTo: (step) => {
      const target = indexOf(step);
      if (target >= index.value) return false;
      index.value = target;
      return true;
    },
    isCurrent: (step) => indexOf(step) === index.value,
    isNext: (step) => indexOf(step) === index.value + 1,
    isPrevious: (step) => indexOf(step) === index.value - 1,
    isBefore: (step) => indexOf(step) < index.value,
    isAfter: (step) => indexOf(step) > index.value,
  };
}

function isNameList(
  steps: readonly string[] | Readonly<Record<string, unknown>>,
): steps is readonly string[] {
  return Array.isArray(steps);
}
