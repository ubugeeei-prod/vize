/** Compile-only assertions for the public Sticky contract. */

import type { StickyExpose, StickySide, StickySlotState, StickyState } from "./sticky.ts";
import { Affix, Sticky, isStickyStuck, stickyRootMargin } from "./sticky.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const sticky: StickyExpose;
declare const slot: StickySlotState;

type _Side = Expect<Equal<StickySide, "bottom" | "top">>;
type _State = Expect<Equal<StickyState, "idle" | "stuck">>;
type _Element = Expect<Equal<typeof sticky.element, HTMLElement | null>>;
type _Stuck = Expect<Equal<typeof slot.stuck, boolean>>;
type _Refresh = Expect<Equal<ReturnType<typeof sticky.refresh>, boolean>>;
type _Helper = Expect<Equal<ReturnType<typeof isStickyStuck>, boolean>>;
type _Margin = Expect<Equal<ReturnType<typeof stickyRootMargin>, string>>;

const props: InstanceType<typeof Sticky>["$props"] = {
  as: "header",
  disabled: false,
  offset: 16,
  root: null,
  side: "top",
  "onStuck-change": (stuck: boolean) => stuck,
};

// @ts-expect-error side is a closed union.
const badSide: InstanceType<typeof Sticky>["$props"] = { side: "left" };

// @ts-expect-error `as` accepts native tag names only.
const badTag: InstanceType<typeof Sticky>["$props"] = { as: "not-a-tag" };

// @ts-expect-error offset is numeric.
const badOffset: InstanceType<typeof Sticky>["$props"] = { offset: "8px" };

void Affix;
void badOffset;
void badSide;
void badTag;
void props;
