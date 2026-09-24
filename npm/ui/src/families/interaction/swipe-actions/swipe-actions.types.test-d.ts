/** Compile-only assertions for the public SwipeActions contract. */

import {
  SwipeActions,
  SwipeActionsAction,
  SwipeActionsTray,
  type SwipeActionsExpose,
  type SwipeActionsOpen,
  type SwipeActionsSide,
  type SwipeActionsState,
} from "./swipe-actions.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const row: SwipeActionsExpose;

type _SideIsClosed = Expect<Equal<SwipeActionsSide, "leading" | "trailing">>;
type _OpenAllowsNull = Expect<Equal<SwipeActionsOpen, "leading" | "trailing" | null>>;
type _StateIsClosed = Expect<Equal<SwipeActionsState, "closed" | "dragging" | "open">>;
type _FullSwipe = Expect<Equal<typeof row.fullSwipeSide, SwipeActionsSide | null>>;

const props: InstanceType<typeof SwipeActions>["$props"] = {
  as: "li",
  fullSwipeThreshold: 0.7,
  "onUpdate:open": (open: SwipeActionsOpen) => open,
  onFullSwipe: (side: SwipeActionsSide) => side,
};
const trayProps: InstanceType<typeof SwipeActionsTray>["$props"] = { side: "trailing" };
const actionProps: InstanceType<typeof SwipeActionsAction>["$props"] = { value: "delete" };

row.openSide("leading", true);

// @ts-expect-error sides are logical, not physical.
row.openSide("left");

// @ts-expect-error trays need a side.
const missingSide: InstanceType<typeof SwipeActionsTray>["$props"] = {};

void actionProps;
void missingSide;
void props;
void trayProps;
