/** Layout axis of the top-level list. */
export type NavigationMenuOrientation = "horizontal" | "vertical";

/** Reading direction used for arrow keys and motion direction. */
export type NavigationMenuDirection = "ltr" | "rtl";

/** Open value of a NavigationMenuRoot. `null` means every flyout is closed. */
export type NavigationMenuValue = string | null;

/** Input that opened or closed a flyout. */
export type NavigationMenuChangeReason =
  | "dismiss"
  | "keyboard"
  | "link"
  | "pointer"
  | "programmatic"
  | "toggle";

/** Visibility state of a flyout, the viewport, or the indicator. */
export type NavigationMenuOpenState = "closed" | "open";

/**
 * Direction a flyout enters from or leaves toward, relative to the previously
 * open item, so consumers can animate content horizontally.
 */
export type NavigationMenuMotion = "from-end" | "from-start" | "to-end" | "to-start";

/** State exposed to the NavigationMenuRoot default slot. */
export interface NavigationMenuSlotState {
  /** Open item value, or `null`. */
  readonly value: NavigationMenuValue;

  /** Layout axis. */
  readonly orientation: NavigationMenuOrientation;

  /** Whether any flyout is open. */
  readonly open: boolean;
}

/** State exposed to item, trigger, and content slots. */
export interface NavigationMenuItemSlotState {
  /** Item value. */
  readonly value: string;

  /** Whether this item's flyout is open. */
  readonly open: boolean;

  /** Stable visibility token. */
  readonly state: NavigationMenuOpenState;
}

/** State exposed to NavigationMenuLink slots. */
export interface NavigationMenuLinkSlotState {
  /** Whether the link represents the current page. */
  readonly active: boolean;
}

/** State exposed to NavigationMenuIndicator and NavigationMenuViewport slots. */
export interface NavigationMenuMeasuredSlotState {
  /** Open item value, or `null`. */
  readonly value: NavigationMenuValue;

  /** Stable visibility token. */
  readonly state: NavigationMenuOpenState;
}

/** Public instance exposed by NavigationMenuRoot. */
export interface NavigationMenuRootExpose {
  /** Rendered navigation landmark. */
  readonly element: HTMLElement | null;

  /** Open item value. */
  readonly value: NavigationMenuValue;

  /** Open one item's flyout immediately. */
  readonly open: (value: string) => boolean;

  /** Close the open flyout immediately. */
  readonly close: () => boolean;
}
