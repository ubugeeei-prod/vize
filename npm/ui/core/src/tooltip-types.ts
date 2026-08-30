import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
} from "./dismissable-layer.ts";
import type { Placement, PositionerStrategy, Rect } from "./positioner.ts";

/** Open state mirrored to the Tooltip data contract. */
export type TooltipState = "closed" | "open";

/** Placement accepted by TooltipContent. */
export type TooltipPlacement = Placement;

/** CSS positioning strategy accepted by TooltipContent. */
export type TooltipPositionerStrategy = PositionerStrategy;

/** Viewport override accepted by TooltipContent. */
export type TooltipViewport = Rect;

/** Preventable Escape lifecycle event emitted by TooltipContent. */
export type TooltipEscapeKeyDownEvent = DismissableLayerEscapeKeyDownEvent;

/** Dismissal notification emitted by TooltipContent. */
export type TooltipDismissEvent = DismissableLayerDismissEvent;

/** State exposed to compound Tooltip slots. */
export interface TooltipSlotState {
  /** Whether the tooltip content is currently visible. */
  readonly open: boolean;

  /** Whether trigger-driven opening is disabled by the root. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: TooltipState;
}

/** State exposed to TooltipContent slots. */
export interface TooltipContentSlotState extends TooltipSlotState {
  /** Preferred placement handed to the positioner. */
  readonly placement: TooltipPlacement;
}

/** Public instance exposed by TooltipRoot. */
export interface TooltipRootExpose extends TooltipSlotState {
  /** Root-owned base id for the compound tooltip family. */
  readonly id: string;

  /** Id wired to the native trigger button. */
  readonly triggerId: string;

  /** Id wired from TooltipTrigger to TooltipContent. */
  readonly contentId: string;

  /** Delay before hover or focus opens the tooltip. */
  readonly delayDuration: number;

  /** Window after closing where hover or focus opens without delay. */
  readonly skipDelayDuration: number;

  /** Request a specific open value and report whether it differs. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Request the open state immediately. */
  readonly openTooltip: (event?: Event | null) => boolean;

  /** Request the closed state and clear a pending delayed open. */
  readonly close: (event?: Event | null) => boolean;

  /** Request opening after the configured delay. */
  readonly scheduleOpen: (event?: Event | null) => boolean;

  /** Clear a pending delayed open. */
  readonly cancelOpen: () => boolean;
}

/** Public instance exposed by TooltipTrigger. */
export interface TooltipTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by TooltipContent. */
export interface TooltipContentExpose extends TooltipSlotState {
  /** Rendered tooltip content element. */
  readonly element: HTMLDivElement | null;
}
