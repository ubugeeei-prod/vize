import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../dismissable-layer/dismissable-layer.ts";
import type { Placement, PositionerStrategy, Rect } from "../positioner/positioner.ts";

/** Open state mirrored to the HoverCard data contract. */
export type HoverCardState = "closed" | "open";

/**
 * How touch input interacts with the trigger.
 *
 * - `"ignore"`: touch never opens the card; taps keep native link behavior.
 * - `"long-press"`: holding the trigger opens the card and suppresses the
 *   follow-up click and context menu.
 */
export type HoverCardTouchBehavior = "ignore" | "long-press";

/** Why the hover card most recently opened. */
export type HoverCardOpenReason = "focus" | "hover" | "long-press" | "programmatic";

/** Placement accepted by HoverCardContent. */
export type HoverCardPlacement = Placement;

/** CSS positioning strategy accepted by HoverCardContent. */
export type HoverCardPositionerStrategy = PositionerStrategy;

/** Viewport override accepted by HoverCardContent. */
export type HoverCardViewport = Rect;

/** Preventable Escape event emitted by HoverCardContent. */
export type HoverCardEscapeKeyDownEvent = DismissableLayerEscapeKeyDownEvent;

/** Preventable outside pointer-down event emitted by HoverCardContent. */
export type HoverCardPointerDownOutsideEvent = DismissableLayerPointerDownOutsideEvent;

/** Dismissal notification emitted by HoverCardContent. */
export type HoverCardDismissEvent = DismissableLayerDismissEvent;

/** State exposed to compound HoverCard slots. */
export interface HoverCardSlotState {
  /** Whether the card content is visible. */
  readonly open: boolean;

  /** Whether pointer, focus, and touch opening are disabled. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: HoverCardState;

  /** Why the card last opened, or `null` while closed. */
  readonly reason: HoverCardOpenReason | null;
}

/** State exposed to HoverCardContent slots. */
export interface HoverCardContentSlotState extends HoverCardSlotState {
  /** Preferred placement handed to the positioner. */
  readonly placement: HoverCardPlacement;
}

/** Public instance exposed by HoverCardRoot. */
export interface HoverCardRootExpose extends HoverCardSlotState {
  /** Root-owned base id. */
  readonly id: string;

  /** Id wired to the trigger. */
  readonly triggerId: string;

  /** Id wired to the content. */
  readonly contentId: string;

  /** Milliseconds before pointer or focus intent opens the card. */
  readonly openDelay: number;

  /** Milliseconds before pointer or focus departure closes the card. */
  readonly closeDelay: number;

  /** Request a specific open value immediately and report whether it differs. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Open after `openDelay`, cancelling a pending close. */
  readonly scheduleOpen: (event?: Event | null) => boolean;

  /** Close after `closeDelay`, cancelling a pending open. */
  readonly scheduleClose: (event?: Event | null) => boolean;

  /** Cancel any pending delayed open or close. Returns whether a timer was cleared. */
  readonly cancelPending: () => boolean;
}

/** Public instance exposed by HoverCardTrigger. */
export interface HoverCardTriggerExpose {
  /** Rendered trigger element. */
  readonly element: HTMLElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by HoverCardContent. */
export interface HoverCardContentExpose extends HoverCardSlotState {
  /** Rendered card element. */
  readonly element: HTMLDivElement | null;
}
