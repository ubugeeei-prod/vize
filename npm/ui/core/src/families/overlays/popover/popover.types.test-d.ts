/** Compile-only assertions for the public Popover contract. */

import type {
  PopoverAlign,
  PopoverArrowExpose,
  PopoverArrowSlotState,
  PopoverContentExpose,
  PopoverContentSlotState,
  PopoverPlacement,
  PopoverPositionerStrategy,
  PopoverRootExpose,
  PopoverSide,
  PopoverSlotState,
  PopoverState,
  PopoverTriggerExpose,
} from "./popover.ts";
import { Popover, PopoverArrow, PopoverContent, PopoverRoot, PopoverTrigger } from "./popover.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: PopoverRootExpose;
declare const trigger: PopoverTriggerExpose;
declare const content: PopoverContentExpose;
declare const arrow: PopoverArrowExpose;
declare const slot: PopoverSlotState;
declare const contentSlot: PopoverContentSlotState;
declare const arrowSlot: PopoverArrowSlotState;

type _StateIsLiteral = Expect<Equal<PopoverState, "closed" | "open">>;
type _PlacementIncludesAlignedSides = Expect<
  Equal<Extract<PopoverPlacement, "bottom-start">, "bottom-start">
>;
type _SideIsLiteral = Expect<Equal<PopoverSide, "bottom" | "left" | "right" | "top">>;
type _AlignIsLiteral = Expect<Equal<PopoverAlign, "center" | "end" | "start">>;
type _StrategyIsLiteral = Expect<Equal<PopoverPositionerStrategy, "absolute" | "fixed">>;
type _RootOpenIsBoolean = Expect<Equal<typeof root.open, boolean>>;
type _TriggerElementIsButton = Expect<Equal<typeof trigger.element, HTMLButtonElement | null>>;
type _ContentElementIsDiv = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _ArrowElementIsDiv = Expect<Equal<typeof arrow.element, HTMLDivElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly open: boolean;
      readonly modal: boolean;
      readonly disabled: boolean;
      readonly state: PopoverState;
    }
  >
>;
type _ContentSlotStateHasPlacement = Expect<Equal<typeof contentSlot.placement, PopoverPlacement>>;
type _ArrowSlotHasCoordinates = Expect<Equal<typeof arrowSlot.x, number | null>>;

const rootProps: InstanceType<typeof PopoverRoot>["$props"] = {
  defaultOpen: true,
  disabled: false,
  id: "filters",
  modal: true,
  open: false,
  "onUpdate:open": (value: boolean) => value,
};
const triggerProps: InstanceType<typeof PopoverTrigger>["$props"] = {
  ariaLabel: "Filters",
  disabled: false,
  type: "button",
};
const contentProps: InstanceType<typeof PopoverContent>["$props"] = {
  ariaDescribedby: "filters-description",
  ariaLabel: "Filters",
  ariaLabelledby: "filters-title",
  arrowPadding: 4,
  autoFocus: true,
  closeOnEscape: true,
  closeOnFocusOutside: true,
  closeOnPointerDownOutside: true,
  collisionPadding: 8,
  direction: "ltr",
  forceMount: true,
  inertOutside: true,
  lockScroll: true,
  offset: 4,
  placement: "bottom-start",
  portalDisabled: true,
  restoreFocus: true,
  safeArea: true,
  size: true,
  strategy: "fixed",
  trapFocus: true,
};

root.close();
root.openPopover();
root.setOpen(true);
root.toggle();
trigger.focus();
content.focusContent();
content.focusFirst();

// @ts-expect-error Popover state has a closed token contract.
const invalidState: PopoverState = "opening";

// @ts-expect-error placement is limited to Positioner placements.
const invalidPlacement: PopoverPlacement = "middle";

// @ts-expect-error open is boolean-only.
const badRootProps: InstanceType<typeof PopoverRoot>["$props"] = { open: "true" };

// @ts-expect-error trigger type remains a native button type.
const badTriggerProps: InstanceType<typeof PopoverTrigger>["$props"] = { type: "menu" };

// @ts-expect-error strategy must be a Positioner strategy.
const badContentProps: InstanceType<typeof PopoverContent>["$props"] = { strategy: "sticky" };

void Popover;
void PopoverArrow;
void badContentProps;
void badRootProps;
void badTriggerProps;
void contentProps;
void invalidPlacement;
void invalidState;
void rootProps;
void triggerProps;
