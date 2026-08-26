/** Compile-only assertions for the public locale contract. */

import type { ComputedRef } from "vue";

import {
  resolveDirection,
  type DirectionPreference,
  type TextDirection,
  useDirection,
  useLocale,
} from "./locale.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const directions: readonly TextDirection[] = ["ltr", "rtl"];
export const preferences: readonly DirectionPreference[] = ["auto", "ltr", "rtl"];
// @ts-expect-error vertical is not a writing direction.
export const invalidDirection: TextDirection = "vertical";

export const resolved: TextDirection = resolveDirection("auto", "en-US");

type _LocaleIsComputedString = Expect<Equal<ReturnType<typeof useLocale>, ComputedRef<string>>>;
type _DirectionIsComputed = Expect<
  Equal<ReturnType<typeof useDirection>, ComputedRef<TextDirection>>
>;

// @ts-expect-error auto is a preference, not a resolved direction.
export const unresolved: TextDirection = "auto";
