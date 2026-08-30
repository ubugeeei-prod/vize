/** Compile-only assertions for the public Tooltip contract. */

import type {
  TooltipContentExpose,
  TooltipContentSlotState,
  TooltipPlacement,
  TooltipPositionerStrategy,
  TooltipRootExpose,
  TooltipSlotState,
  TooltipState,
  TooltipTriggerExpose,
} from "./tooltip.ts";
import { Tooltip, TooltipContent, TooltipRoot, TooltipTrigger } from "./tooltip.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: TooltipRootExpose;
declare const trigger: TooltipTriggerExpose;
declare const content: TooltipContentExpose;
declare const slot: TooltipSlotState;
declare const contentSlot: TooltipContentSlotState;

type _StateIsLiteral = Expect<Equal<TooltipState, "closed" | "open">>;
type _PlacementIncludesAlignedSides = Expect<
  Equal<Extract<TooltipPlacement, "top-start">, "top-start">
>;
type _StrategyIsLiteral = Expect<Equal<TooltipPositionerStrategy, "absolute" | "fixed">>;
type _RootOpenIsBoolean = Expect<Equal<typeof root.open, boolean>>;
type _TriggerElementIsButton = Expect<Equal<typeof trigger.element, HTMLButtonElement | null>>;
type _ContentElementIsDiv = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly open: boolean;
      readonly disabled: boolean;
      readonly state: TooltipState;
    }
  >
>;
type _ContentSlotStateHasPlacement = Expect<Equal<typeof contentSlot.placement, TooltipPlacement>>;

const rootProps: InstanceType<typeof TooltipRoot>["$props"] = {
  defaultOpen: true,
  delayDuration: 300,
  disabled: false,
  id: "help",
  open: false,
  skipDelayDuration: 100,
  "onUpdate:open": (value: boolean) => value,
};
const triggerProps: InstanceType<typeof TooltipTrigger>["$props"] = {
  ariaLabel: "More info",
  disabled: false,
  type: "button",
};
const contentProps: InstanceType<typeof TooltipContent>["$props"] = {
  closeOnEscape: true,
  collisionPadding: 8,
  forceMount: true,
  offset: 4,
  placement: "bottom-start",
  portalDisabled: true,
  strategy: "fixed",
};

root.cancelOpen();
root.close();
root.openTooltip();
root.scheduleOpen();
root.setOpen(true);
trigger.focus();

// @ts-expect-error Tooltip state has a closed token contract.
const invalidState: TooltipState = "opening";

// @ts-expect-error placement is limited to Positioner placements.
const invalidPlacement: TooltipPlacement = "middle";

// @ts-expect-error open is boolean-only.
const badRootProps: InstanceType<typeof TooltipRoot>["$props"] = { open: "true" };

// @ts-expect-error trigger type remains a native button type.
const badTriggerProps: InstanceType<typeof TooltipTrigger>["$props"] = { type: "menu" };

// @ts-expect-error strategy must be a Positioner strategy.
const badContentProps: InstanceType<typeof TooltipContent>["$props"] = { strategy: "sticky" };

void Tooltip;
void badContentProps;
void badRootProps;
void badTriggerProps;
void contentProps;
void invalidPlacement;
void invalidState;
void rootProps;
void triggerProps;
