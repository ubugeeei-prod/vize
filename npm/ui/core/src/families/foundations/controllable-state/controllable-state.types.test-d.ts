/** Compile-only assertions for the public controllable-state contract. */

import type { ComputedRef } from "vue";

import { useControllableState, type ControllableState } from "./controllable-state.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface SelectionValue {
  readonly key: string;
}

export const state = useControllableState<SelectionValue>({
  defaultValue: { key: "alpha" },
  onChange(value, previous) {
    const nextKey: string = value.key;
    const previousKey: string = previous.key;
    void nextKey;
    void previousKey;
  },
});

type _StatePreservesGeneric = Expect<Equal<typeof state, ControllableState<SelectionValue>>>;
type _ValueIsReadonlyComputed = Expect<Equal<typeof state.value, ComputedRef<SelectionValue>>>;
type _ControlledIsReadonlyComputed = Expect<Equal<typeof state.controlled, ComputedRef<boolean>>>;
type _SetterReportsWhetherValueChanged = Expect<Equal<ReturnType<typeof state.set>, boolean>>;
type _ResetReportsWhetherValueChanged = Expect<Equal<ReturnType<typeof state.reset>, boolean>>;

state.set({ key: "bravo" });
state.set((previous) => ({ key: previous.key.toUpperCase() }));

// @ts-expect-error setters preserve the declared value shape.
state.set({ key: 1 });

// @ts-expect-error updater returns must preserve the declared value shape.
state.set(() => ({ key: 1 }));

// @ts-expect-error the public value ref is readonly.
state.value.value = { key: "charlie" };

useControllableState({
  value: () => "controlled",
  defaultValue: "fallback",
  equals: (left, right) => left.length === right.length,
});

// @ts-expect-error controlled and default values must share one value type.
useControllableState({ value: () => "controlled", defaultValue: 1 });
