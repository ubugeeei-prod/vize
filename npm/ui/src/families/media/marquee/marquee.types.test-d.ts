/** Compile-only assertions for the public Marquee contract. */

import {
  Marquee,
  MarqueeContent,
  MarqueePauseButton,
  MarqueeRoot,
  marqueeCopies,
  type MarqueeContentExpose,
  type MarqueeDirection,
  type MarqueeMessageOverrides,
  type MarqueeMessages,
  type MarqueeOrientation,
  type MarqueePauseButtonExpose,
  type MarqueePauseReason,
  type MarqueeRootExpose,
  type MarqueeSlotState,
  type MarqueeState,
} from "./marquee.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: MarqueeRootExpose;
declare const content: MarqueeContentExpose;
declare const button: MarqueePauseButtonExpose;

type _DirectionIsLiteral = Expect<Equal<MarqueeDirection, "down" | "left" | "right" | "up">>;
type _OrientationIsLiteral = Expect<Equal<MarqueeOrientation, "horizontal" | "vertical">>;
type _StateIsLiteral = Expect<Equal<MarqueeState, "paused" | "running">>;
type _ReasonIsLiteral = Expect<
  Equal<MarqueePauseReason, "focus" | "hover" | "reduced-motion" | "user">
>;
type _SlotReason = Expect<Equal<MarqueeSlotState["pauseReason"], MarqueePauseReason | null>>;
type _DurationNullable = Expect<Equal<typeof root.duration, number | null>>;
type _PlayReports = Expect<Equal<typeof root.play, () => boolean>>;
type _TrackElement = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _ButtonElement = Expect<Equal<typeof button.element, HTMLButtonElement | null>>;
type _Overrides = Expect<Equal<MarqueeMessageOverrides, Partial<MarqueeMessages>>>;
type _AliasIsRoot = Expect<Equal<typeof Marquee, typeof MarqueeRoot>>;
type _CopiesIsNumber = Expect<Equal<ReturnType<typeof marqueeCopies>, number>>;

void MarqueeContent;
void MarqueePauseButton;

// @ts-expect-error directions are a closed union.
const _diagonal: MarqueeDirection = "up-left";
// @ts-expect-error repeat accepts a number or "auto" only.
marqueeCopies("fill", null, null);
// @ts-expect-error exposed state is read-only.
root.state = "paused";
