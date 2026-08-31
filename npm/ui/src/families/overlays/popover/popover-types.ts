import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../dismissable-layer/dismissable-layer.ts";
import type { FocusScopeAutoFocusEvent } from "../../accessibility/focus-scope/focus-scope.ts";
import type {
  Placement,
  PlacementAlign,
  PlacementSide,
  PositionerStrategy,
  Rect,
} from "../positioner/positioner.ts";

/** Open state mirrored to the Popover data contract. */
export type PopoverState = "closed" | "open";

/** Placement accepted by PopoverContent. */
export type PopoverPlacement = Placement;

/** Side token resolved from PopoverContent placement. */
export type PopoverSide = PlacementSide;

/** Alignment token resolved from PopoverContent placement. */
export type PopoverAlign = PlacementAlign;

/** CSS positioning strategy accepted by PopoverContent. */
export type PopoverPositionerStrategy = PositionerStrategy;

/** Viewport override accepted by PopoverContent. */
export type PopoverViewport = Rect;

/** Preventable Popover auto-focus lifecycle event. */
export type PopoverAutoFocusEvent = FocusScopeAutoFocusEvent;

/** Preventable Escape lifecycle event emitted by PopoverContent. */
export type PopoverEscapeKeyDownEvent = DismissableLayerEscapeKeyDownEvent;

/** Preventable outside pointer-down event emitted by PopoverContent. */
export type PopoverPointerDownOutsideEvent = DismissableLayerPointerDownOutsideEvent;

/** Preventable outside focus event emitted by PopoverContent. */
export type PopoverFocusOutsideEvent = DismissableLayerFocusOutsideEvent;

/** Preventable outside interaction event emitted by PopoverContent. */
export type PopoverInteractOutsideEvent = DismissableLayerInteractOutsideEvent;

/** Dismissal notification emitted by PopoverContent. */
export type PopoverDismissEvent = DismissableLayerDismissEvent;

/** State exposed to compound Popover slots. */
export interface PopoverSlotState {
  /** Whether the popover content is currently visible and interactive. */
  readonly open: boolean;

  /** Whether outside content is inert and focus-contained. */
  readonly modal: boolean;

  /** Whether trigger-driven opening is disabled by the root. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: PopoverState;
}

/** State exposed to PopoverContent slots. */
export interface PopoverContentSlotState extends PopoverSlotState {
  /** Resolved side published by the positioner after collision handling. */
  readonly side: PopoverSide;

  /** Resolved alignment published by the positioner after collision handling. */
  readonly align: PopoverAlign;

  /** Full resolved placement published by the positioner. */
  readonly placement: PopoverPlacement;
}

/** State exposed to PopoverArrow slots. */
export interface PopoverArrowSlotState {
  /** Current arrow x coordinate when the positioner can measure it. */
  readonly x: number | null;

  /** Current arrow y coordinate when the positioner can measure it. */
  readonly y: number | null;
}

/** Public instance exposed by PopoverRoot. */
export interface PopoverRootExpose extends PopoverSlotState {
  /** Root-owned base id for the compound popover family. */
  readonly id: string;

  /** Id wired to the native trigger button. */
  readonly triggerId: string;

  /** Id wired from PopoverTrigger to PopoverContent. */
  readonly contentId: string;

  /** Request a specific open value and report whether it differs. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Request the open state. */
  readonly openPopover: (event?: Event | null) => boolean;

  /** Request the closed state. */
  readonly close: (event?: Event | null) => boolean;

  /** Request the opposite open state. */
  readonly toggle: (event?: Event | null) => boolean;
}

/** Public instance exposed by PopoverTrigger. */
export interface PopoverTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by PopoverContent. */
export interface PopoverContentExpose extends PopoverSlotState {
  /** Rendered popover content element. */
  readonly element: HTMLDivElement | null;

  /** Focus the first eligible descendant in the popover content. */
  readonly focusFirst: () => HTMLElement | null;

  /** Move focus to the content fallback target. */
  readonly focusContent: (options?: FocusOptions) => void;
}

/** Public instance exposed by PopoverArrow. */
export interface PopoverArrowExpose extends PopoverArrowSlotState {
  /** Rendered arrow element. */
  readonly element: HTMLDivElement | null;
}
