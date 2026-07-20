import { computed, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Direct value or updater accepted by {@link ControllableState.set}. */
export type StateUpdate<Value> = Value | ((previous: Value) => Value);

/** Options for {@link useControllableState}. */
export interface ControllableStateOptions<Value> {
  /**
   * Reactive controlled value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly value?: MaybeRefOrGetter<Value | undefined>;

  /** Initial value and the value restored by {@link ControllableState.reset}. */
  readonly defaultValue: MaybeRefOrGetter<Value>;

  /**
   * Equality comparison used to suppress redundant updates.
   *
   * @default Object.is
   */
  readonly equals?: (left: Value, right: Value) => boolean;

  /**
   * Called after a distinct state change is requested.
   *
   * @default undefined
   */
  readonly onChange?: (value: Value, previous: Value) => void;
}

/** Reactive state that can move safely between controlled and uncontrolled use. */
export interface ControllableState<Value> {
  /** Current controlled or internal value. */
  readonly value: ComputedRef<Value>;

  /** Whether the reactive source currently controls the value. */
  readonly controlled: ComputedRef<boolean>;

  /** Request an update and report whether it differs from the current value. */
  readonly set: (update: StateUpdate<Value>) => boolean;

  /** Request the current default value and report whether it changed. */
  readonly reset: () => boolean;
}

/**
 * Create one state contract for controlled props and internal state.
 *
 * The last controlled value is retained if the source becomes uncontrolled,
 * preventing an abrupt jump back to the initial default.
 */
export function useControllableState<Value>(
  options: ControllableStateOptions<Value>,
): ControllableState<Value> {
  const internal = shallowRef(toValue(options.defaultValue));
  const controlledValue = () => (options.value === undefined ? undefined : toValue(options.value));
  const controlled = computed(() => controlledValue() !== undefined);
  const value = computed<Value>(() => controlledValue() ?? internal.value);

  watch(
    controlledValue,
    (next) => {
      if (next !== undefined) internal.value = next;
    },
    { flush: "sync", immediate: true },
  );

  const set = (update: StateUpdate<Value>) => {
    const previous = value.value;
    const next =
      typeof update === "function" ? (update as (current: Value) => Value)(previous) : update;
    if ((options.equals ?? Object.is)(previous, next)) return false;
    if (!controlled.value) internal.value = next;
    options.onChange?.(next, previous);
    return true;
  };

  return {
    value,
    controlled,
    set,
    reset: () => set(toValue(options.defaultValue)),
  };
}
