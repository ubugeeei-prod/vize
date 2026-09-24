/** Compile-only assertions for the public Lightbox contract. */

import {
  Lightbox,
  LightboxRoot,
  classifyLightboxSwipe,
  defaultLightboxMessages,
  type LightboxButtonExpose,
  type LightboxChangeReason,
  type LightboxContentExpose,
  type LightboxCounterSlotState,
  type LightboxDirection,
  type LightboxIndexSlotState,
  type LightboxMessageOverrides,
  type LightboxMessages,
  type LightboxRootExpose,
  type LightboxSlotState,
  type LightboxState,
  type LightboxSwipe,
} from "./lightbox.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Photo {
  readonly src: string;
}

declare const root: LightboxRootExpose<Photo>;
declare const slot: LightboxSlotState<Photo>;
declare const content: LightboxContentExpose;
declare const button: LightboxButtonExpose;

type _StateIsLiteral = Expect<Equal<LightboxState, "closed" | "open">>;
type _DirectionIsLiteral = Expect<Equal<LightboxDirection, "ltr" | "rtl">>;
type _ReasonIsLiteral = Expect<
  Equal<
    LightboxChangeReason,
    "api" | "keyboard" | "next" | "previous" | "swipe" | "thumbnail" | "trigger"
  >
>;
type _SwipeIsLiteral = Expect<Equal<LightboxSwipe, "close" | "next" | "none" | "previous">>;
type _ItemInferred = Expect<Equal<typeof slot.item, Photo | undefined>>;
type _ItemsInferred = Expect<Equal<typeof root.items, readonly Photo[]>>;
type _OpenAtReports = Expect<Equal<typeof root.openAt, (index: number) => boolean>>;
type _ContentElement = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _ButtonElement = Expect<Equal<typeof button.element, HTMLButtonElement | null>>;
type _CounterSlot = Expect<
  Equal<
    LightboxCounterSlotState,
    { readonly position: number; readonly count: number; readonly text: string }
  >
>;
type _IndexSlot = Expect<
  Equal<LightboxIndexSlotState, { readonly index: number; readonly current: boolean }>
>;
type _Overrides = Expect<Equal<LightboxMessageOverrides, Partial<LightboxMessages>>>;
type _CounterMessage = Expect<
  Equal<typeof defaultLightboxMessages.counter, (position: number, count: number) => string>
>;
type _AliasIsRoot = Expect<Equal<typeof Lightbox, typeof LightboxRoot>>;
type _SwipeReturns = Expect<Equal<ReturnType<typeof classifyLightboxSwipe>, LightboxSwipe>>;

// @ts-expect-error reasons are a closed union.
const _unknownReason: LightboxChangeReason = "wheel";
// @ts-expect-error counters take a position and a count.
const _badCounter: LightboxMessageOverrides = { counter: (label: string) => label };
// @ts-expect-error exposed state is read-only.
root.index = 1;
