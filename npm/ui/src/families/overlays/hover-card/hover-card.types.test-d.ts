/** Compile-only assertions for the public HoverCard contract. */

import type {
  HoverCardContentExpose,
  HoverCardContentSlotState,
  HoverCardOpenReason,
  HoverCardRootExpose,
  HoverCardSlotState,
  HoverCardState,
  HoverCardTouchBehavior,
  HoverCardTriggerExpose,
} from "./hover-card.ts";
import {
  HoverCard,
  HoverCardArrow,
  HoverCardContent,
  HoverCardRoot,
  HoverCardTrigger,
} from "./hover-card.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: HoverCardRootExpose;
declare const trigger: HoverCardTriggerExpose;
declare const content: HoverCardContentExpose;
declare const slot: HoverCardSlotState;
declare const contentSlot: HoverCardContentSlotState;

type _State = Expect<Equal<HoverCardState, "closed" | "open">>;
type _Touch = Expect<Equal<HoverCardTouchBehavior, "ignore" | "long-press">>;
type _Reason = Expect<
  Equal<HoverCardOpenReason, "focus" | "hover" | "long-press" | "programmatic">
>;
type _SlotReason = Expect<Equal<typeof slot.reason, HoverCardOpenReason | null>>;
type _TriggerElement = Expect<Equal<typeof trigger.element, HTMLElement | null>>;
type _ContentElement = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _OpenDelay = Expect<Equal<typeof root.openDelay, number>>;
type _CloseDelay = Expect<Equal<typeof root.closeDelay, number>>;
type _Placement = Expect<
  Equal<(typeof contentSlot)["placement"], import("../positioner/positioner.ts").Placement>
>;

root.scheduleOpen();
root.scheduleClose(new Event("pointerleave"));
root.setOpen(true);
root.cancelPending();
trigger.focus({ preventScroll: true });

const rootProps: InstanceType<typeof HoverCardRoot>["$props"] = {
  closeDelay: 200,
  defaultOpen: false,
  disabled: false,
  longPressDelay: 600,
  openDelay: 400,
  touchBehavior: "long-press",
  "onUpdate:open": (value: boolean) => value,
};
const triggerProps: InstanceType<typeof HoverCardTrigger>["$props"] = { as: "a", disabled: false };
const contentProps: InstanceType<typeof HoverCardContent>["$props"] = {
  closeOnPointerDownOutside: false,
  placement: "top-start",
  portalDisabled: true,
};

// @ts-expect-error touch behavior is a closed union.
const badTouch: InstanceType<typeof HoverCardRoot>["$props"] = { touchBehavior: "tap" };

// @ts-expect-error delays are numbers.
const badDelay: InstanceType<typeof HoverCardRoot>["$props"] = { openDelay: "fast" };

// @ts-expect-error placement must be a positioner placement.
const badPlacement: InstanceType<typeof HoverCardContent>["$props"] = { placement: "middle" };

// @ts-expect-error reasons are a closed union.
const badReason: HoverCardOpenReason = "click";

void HoverCard;
void HoverCardArrow;
void badDelay;
void badPlacement;
void badReason;
void badTouch;
void contentProps;
void rootProps;
void triggerProps;
