/** Compile-only assertions for the public measurement observer contract. */

import type { ShallowRef } from "vue";

import { createSizeObserver, createVisibilityObserver, type SizeObserverEntry } from "./measure.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const sizes = createSizeObserver({
  box: "content-box",
  onResize(entries) {
    const [entry] = entries;
    void entry?.width;
  },
});

export const visibility = createVisibilityObserver({
  rootMargin: "4px",
  threshold: [0, 1],
  onVisibilityChange(entries) {
    void entries[0]?.isIntersecting;
  },
});

type _CountIsReadonly = Expect<Equal<typeof sizes.observedCount, Readonly<ShallowRef<number>>>>;
type _SupportIsBoolean = Expect<Equal<typeof sizes.isSupported, boolean>>;
type _EntriesAreReadonly = Expect<
  Equal<
    Parameters<Parameters<typeof createSizeObserver>[0]["onResize"]>[0],
    readonly SizeObserverEntry[]
  >
>;

// @ts-expect-error margin-box is not an observable box model.
createSizeObserver({ box: "margin-box", onResize: () => {} });
// @ts-expect-error onResize is required.
createSizeObserver({});
// @ts-expect-error consumers cannot mutate readonly reactive state.
sizes.observedCount.value = 5;
// @ts-expect-error visibility thresholds must be numeric.
createVisibilityObserver({ threshold: "half", onVisibilityChange: () => {} });
