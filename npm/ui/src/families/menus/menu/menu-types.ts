import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../../overlays/dismissable-layer/dismissable-layer.ts";
import type { FocusScopeAutoFocusEvent } from "../../accessibility/focus-scope/focus-scope.ts";
import type {
  Placement,
  PlacementAlign,
  PlacementSide,
  PositionerStrategy,
  Rect,
} from "../../overlays/positioner/positioner.ts";

/** Open state mirrored to every menu data contract. */
export type MenuState = "closed" | "open";

/** Reading direction used for horizontal arrow keys and submenu placement. */
export type MenuDirection = "ltr" | "rtl";

/**
 * Which surface owns a menu tree. The kind only changes defaults and data hooks;
 * every kind shares the same keyboard, focus, and dismissal core.
 */
export type MenuKind = "context-menu" | "dropdown-menu" | "menu" | "menubar";

/**
 * Where focus lands when a menu opens.
 *
 * - `first` / `last`: the first or last navigable item (keyboard opening).
 * - `content`: the menu container itself, with no highlighted item (pointer opening).
 * - `none`: focus stays where it is (a submenu opened by pointer hover).
 */
export type MenuEntryFocus = "content" | "first" | "last" | "none";

/** Checked value accepted by checkbox menu items. */
export type MenuCheckedState = boolean | "indeterminate";

/** Stable checked-state token published on checkable items and indicators. */
export type MenuItemCheckedState = "checked" | "indeterminate" | "unchecked";

/** Placement accepted by menu content. */
export type MenuPlacement = Placement;

/** Side token resolved from menu content placement. */
export type MenuSide = PlacementSide;

/** Alignment token resolved from menu content placement. */
export type MenuAlign = PlacementAlign;

/** CSS positioning strategy accepted by menu content. */
export type MenuPositionerStrategy = PositionerStrategy;

/** Viewport override accepted by menu content. */
export type MenuViewport = Rect;

/** Preventable auto-focus lifecycle event emitted by menu content. */
export type MenuAutoFocusEvent = FocusScopeAutoFocusEvent;

/** Preventable Escape event emitted by menu content. */
export type MenuEscapeKeyDownEvent = DismissableLayerEscapeKeyDownEvent;

/** Preventable outside pointer-down event emitted by menu content. */
export type MenuPointerDownOutsideEvent = DismissableLayerPointerDownOutsideEvent;

/** Preventable outside focus event emitted by menu content. */
export type MenuFocusOutsideEvent = DismissableLayerFocusOutsideEvent;

/** Preventable outside interaction event emitted by menu content. */
export type MenuInteractOutsideEvent = DismissableLayerInteractOutsideEvent;

/** Dismissal notification emitted by menu content. */
export type MenuDismissEvent = DismissableLayerDismissEvent;

/**
 * Preventable item activation event.
 *
 * Emitted before the menu tree closes. Calling `preventDefault()` keeps every
 * open menu open, which is how "stay open after toggling" menus are built.
 */
export interface MenuSelectEvent {
  /** Stable discriminator. */
  readonly type: "select";

  /** Activated item element, or `null` when activation was imperative. */
  readonly target: HTMLElement | null;

  /** Native click or keydown responsible for the activation, or `null`. */
  readonly originalEvent: Event | null;

  /** Whether `preventDefault()` was called by any listener. */
  readonly defaultPrevented: boolean;

  /** Keep the menu tree open after this activation. */
  readonly preventDefault: () => void;
}

/** State exposed to menu root and trigger slots. */
export interface MenuSlotState {
  /** Whether the menu content is visible and interactive. */
  readonly open: boolean;

  /** Stable state token for styling and tests. */
  readonly state: MenuState;

  /** Whether outside content is inert and scroll-locked while open. */
  readonly modal: boolean;

  /** Resolved reading direction. */
  readonly dir: MenuDirection;
}

/** State exposed to menu content slots. */
export interface MenuContentSlotState extends MenuSlotState {
  /** Resolved side after collision handling. */
  readonly side: MenuSide;

  /** Resolved alignment after collision handling. */
  readonly align: MenuAlign;

  /** Full resolved placement. */
  readonly placement: MenuPlacement;
}

/** State exposed to every item slot. */
export interface MenuItemSlotState {
  /** Whether the item currently owns the menu highlight (keyboard or pointer). */
  readonly highlighted: boolean;

  /** Whether activation is disabled. Disabled items stay focusable per WAI-ARIA APG. */
  readonly disabled: boolean;
}

/** State exposed to checkbox item slots. */
export interface MenuCheckboxItemSlotState extends MenuItemSlotState {
  /** Current checked value. */
  readonly checked: MenuCheckedState;

  /** Stable checked-state token. */
  readonly state: MenuItemCheckedState;
}

/** State exposed to radio item slots. */
export interface MenuRadioItemSlotState extends MenuItemSlotState {
  /** Whether this item holds the group value. */
  readonly checked: boolean;

  /** Stable checked-state token. */
  readonly state: MenuItemCheckedState;
}

/** State exposed to radio group slots. */
export interface MenuRadioGroupSlotState<Value> {
  /** Current group value, or `null` when nothing is selected. */
  readonly value: Value | null;
}

/** State exposed to item indicator slots. */
export interface MenuItemIndicatorSlotState {
  /** Checked-state token of the owning checkbox or radio item. */
  readonly state: MenuItemCheckedState;
}

/** State exposed to submenu slots. */
export interface MenuSubSlotState {
  /** Whether the submenu is open. */
  readonly open: boolean;

  /** Stable state token. */
  readonly state: MenuState;
}

/** State exposed to submenu trigger slots. */
export interface MenuSubTriggerSlotState extends MenuItemSlotState, MenuSubSlotState {}

/** State exposed to arrow slots. */
export interface MenuArrowSlotState {
  /** Current arrow x coordinate when measured. */
  readonly x: number | null;

  /** Current arrow y coordinate when measured. */
  readonly y: number | null;
}

/** Imperative surface exposed by every root-level menu provider. */
export interface MenuRootExpose extends MenuSlotState {
  /** Root-owned base id. */
  readonly id: string;

  /** Id wired to the trigger. */
  readonly triggerId: string;

  /** Id wired to the content. */
  readonly contentId: string;

  /** Request an open value; returns whether it changed. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Open the menu and move focus to `entry` once content mounts. */
  readonly openMenu: (entry?: MenuEntryFocus, event?: Event | null) => boolean;

  /** Close the whole menu tree. */
  readonly close: (event?: Event | null) => boolean;

  /** Toggle the menu. */
  readonly toggle: (event?: Event | null) => boolean;
}

/** Imperative surface exposed by MenuTrigger. */
export interface MenuTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Imperative surface exposed by MenuContent and MenuSubContent. */
export interface MenuContentExpose {
  /** Rendered `role="menu"` element. */
  readonly element: HTMLDivElement | null;

  /** Whether the content is open. */
  readonly open: boolean;

  /** Stable state token. */
  readonly state: MenuState;

  /** Focus the first navigable item. */
  readonly focusFirst: () => HTMLElement | null;

  /** Focus the last navigable item. */
  readonly focusLast: () => HTMLElement | null;

  /** Focus the menu container and clear the highlight. */
  readonly focusContent: () => void;
}

/** Imperative surface exposed by menu items. */
export interface MenuItemExpose {
  /** Rendered `role="menuitem"` element. */
  readonly element: HTMLDivElement | null;

  /** Stable item id, also used as its collection key. */
  readonly id: string;

  /** Whether the item owns the highlight. */
  readonly highlighted: boolean;

  /** Whether activation is disabled. */
  readonly disabled: boolean;

  /** Move focus (and the highlight) to this item. */
  readonly focus: () => void;

  /** Activate the item as if the user selected it. */
  readonly select: (event?: Event | null) => boolean;
}

/** Imperative surface exposed by checkbox items. */
export interface MenuCheckboxItemExpose extends MenuItemExpose {
  /** Current checked value. */
  readonly checked: MenuCheckedState;
}

/** Imperative surface exposed by radio items. */
export interface MenuRadioItemExpose extends MenuItemExpose {
  /** Whether this item holds the group value. */
  readonly checked: boolean;
}

/** Imperative surface exposed by radio groups. */
export interface MenuRadioGroupExpose<Value> {
  /** Current group value. */
  readonly value: Value | null;

  /** Request a group value; returns whether it changed. */
  readonly setValue: (value: Value | null) => boolean;
}

/** Imperative surface exposed by MenuSub. */
export interface MenuSubExpose extends MenuSubSlotState {
  /** Id wired to the submenu trigger. */
  readonly triggerId: string;

  /** Id wired to the submenu content. */
  readonly contentId: string;

  /** Request the submenu open value. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
}

/** Imperative surface exposed by structural menu parts. */
export interface MenuElementExpose {
  /** Rendered element. */
  readonly element: HTMLElement | null;
}

/** Imperative surface exposed by MenuArrow. */
export interface MenuArrowExpose extends MenuArrowSlotState {
  /** Rendered arrow element. */
  readonly element: HTMLDivElement | null;
}
