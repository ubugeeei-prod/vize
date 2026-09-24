/** Compile-only assertions for the `use-segmenter` type contracts. */

import { useSegmenter } from "./use-segmenter.ts";
import type { TextSegment } from "./use-segmenter.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const segmenter = useSegmenter("text", { granularity: "word" });
type _SegmentsAreTyped = Expect<Equal<typeof segmenter.segments.value, readonly TextSegment[]>>;
segmenter.truncate(10) satisfies string;

// @ts-expect-error granularity is a closed union.
useSegmenter("text", { granularity: "line" });
