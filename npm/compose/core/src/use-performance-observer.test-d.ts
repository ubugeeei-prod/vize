/** Compile-only assertions for the `use-performance-observer` type contracts. */

import { usePerformanceObserver } from "./use-performance-observer.ts";
import type {
  EventTimingEntryLike,
  LargestContentfulPaintEntryLike,
  LayoutShiftEntryLike,
  PerformanceObserverHost,
} from "./use-performance-observer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

usePerformanceObserver("largest-contentful-paint", (entries) => {
  type _Single = Expect<Equal<(typeof entries)[number], LargestContentfulPaintEntryLike>>;
});

const observer = usePerformanceObserver(["layout-shift", "event"], (entries) => {
  type _Union = Expect<
    Equal<(typeof entries)[number], LayoutShiftEntryLike | EventTimingEntryLike>
  >;
  for (const entry of entries) {
    if (entry.entryType === "layout-shift") {
      type _Narrowed = Expect<Equal<typeof entry, LayoutShiftEntryLike>>;
    }
  }
});

type _Records = Expect<
  Equal<
    ReturnType<typeof observer.takeRecords>[number],
    LayoutShiftEntryLike | EventTimingEntryLike
  >
>;

PerformanceObserver satisfies PerformanceObserverHost;

// @ts-expect-error unknown entry types are rejected.
usePerformanceObserver("scroll", () => undefined);

// @ts-expect-error the active flag is read-only.
observer.isActive.value = true;
