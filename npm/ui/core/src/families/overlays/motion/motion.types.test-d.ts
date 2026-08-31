/** Compile-only assertions for the public motion contract. */

import {
  motionDurations,
  motionEasings,
  motionTokenVar,
  setMotionTokens,
  startViewTransition,
} from "./motion.ts";
import type {
  MotionDurationToken,
  MotionEasingToken,
  MotionEnterExitRecipe,
  MotionRecipe,
  MotionTokenName,
  ViewTransitionHandle,
} from "./motion.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const durations: readonly MotionDurationToken[] = [
  "instant",
  "fast",
  "base",
  "slow",
  "deliberate",
];
export const easings: readonly MotionEasingToken[] = [
  "standard",
  "decelerate",
  "accelerate",
  "emphasized",
  "linear",
];
export const enterExit: readonly MotionEnterExitRecipe[] = ["fade", "scale", "slide"];
export const emphasis: readonly MotionRecipe[] = ["enter", "move", "pulse", "shake", "reveal"];

export const reference: string = motionTokenVar("duration-fast");
export const easeReference: string = motionTokenVar("ease-emphasized");
export const hookReference: string = motionTokenVar("slide-distance");

type _TokenNamesAreClosed = Expect<
  Equal<Extract<MotionTokenName, "duration-fast" | "ease-linear">, "duration-fast" | "ease-linear">
>;
type _HandleNativeIsBoolean = Expect<Equal<ViewTransitionHandle["native"], boolean>>;
type _HandleFinishedIsPromise = Expect<Equal<ViewTransitionHandle["finished"], Promise<void>>>;

export const handle: ViewTransitionHandle = startViewTransition(() => undefined, {
  respectReducedMotion: false,
});

export const restore: () => void = setMotionTokens(document.createElement("div"), {
  "duration-base": "150ms",
  "ease-standard": "linear",
});

// @ts-expect-error unknown token names never compile.
motionTokenVar("duration-bogus");
// @ts-expect-error the duration record is readonly.
motionDurations.fast = "1ms";
// @ts-expect-error the easing record is readonly.
motionEasings.standard = "linear";
// @ts-expect-error the update callback is mandatory.
startViewTransition();
// @ts-expect-error respectReducedMotion is a boolean option.
startViewTransition(() => undefined, { respectReducedMotion: "never" });
// @ts-expect-error overrides only accept known token names.
setMotionTokens(document.createElement("div"), { "duration-bogus": "1ms" });
// @ts-expect-error the handle surface is readonly.
handle.native = false;
// @ts-expect-error recipe unions reject arbitrary strings.
export const badRecipe: MotionRecipe = "wiggle";
