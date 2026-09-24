import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistry } from "../../foundations/collection/collection.ts";
import type { PositionerElement } from "../../overlays/positioner/positioner.ts";
import type {
  MenuDirection,
  MenuEntryFocus,
  MenuItemCheckedState,
  MenuKind,
  MenuState,
} from "./menu-types.ts";

/** Direction of a horizontal hand-off requested by content keyboard navigation. */
export type MenuEdgeDirection = "next" | "previous";

/**
 * One open/closed menu level: the root menu or a submenu.
 *
 * Levels form a chain through `parent`; closing a level closes every
 * descendant level because descendant `open` state is gated on the parent.
 */
export interface MenuLevelContextValue {
  /** Parent level, or `null` for the root menu. */
  readonly parent: MenuLevelContextValue | null;
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<MenuState>;
  /** Whether this level refuses to open. */
  readonly disabled: ComputedRef<boolean>;
  /** Whether a labelling trigger exists; set during setup so SSR output can reference it. */
  readonly hasTrigger: ShallowRef<boolean>;
  /** Element the trigger registers for ARIA wiring and focus return. */
  readonly triggerElement: ShallowRef<HTMLElement | null>;
  /** Element or virtual point the content is positioned against. */
  readonly reference: ComputedRef<PositionerElement | null>;
  readonly contentElement: ShallowRef<HTMLDivElement | null>;
  /** Focus target requested by whoever opened this level. */
  readonly entryFocus: ShallowRef<MenuEntryFocus>;
  /** Parent-content collection key of the item that opens this level. */
  readonly triggerItemKey: ShallowRef<string | null>;
  /** Focus the first or last item of this level's rendered content, when mounted. */
  readonly focusContentEdge: ShallowRef<((edge: "first" | "last") => HTMLElement | null) | null>;
  /** Suppress automatic focus return for the next close (outside interaction). */
  readonly skipFocusReturn: ShallowRef<boolean>;
  /** Request an open value, optionally recording the entry focus target. */
  readonly setOpen: (value: boolean, event?: Event | null, entry?: MenuEntryFocus) => boolean;
  /** Element focus returns to when this level closes, or `null` for the captured element. */
  readonly restoreTarget: () => HTMLElement | null;
}

/** State shared by every level of one menu tree. */
export interface MenuTreeContextValue {
  readonly kind: MenuKind;
  readonly root: MenuLevelContextValue;
  readonly modal: ComputedRef<boolean>;
  readonly dir: ComputedRef<MenuDirection>;
  readonly loop: ComputedRef<boolean>;
  /** Rendered submenu contents that the modal root keeps outside its inert region. */
  readonly layers: ShallowRef<readonly HTMLElement[]>;
  /** Close every level of the tree. */
  readonly closeAll: (event?: Event | null) => boolean;
  /**
   * Horizontal hand-off used by menubars. Returns `true` when the key was
   * consumed by moving to an adjacent menu.
   */
  readonly edgeNavigate: (direction: MenuEdgeDirection, event: KeyboardEvent) => boolean;
}

/** Collection value stored for each registered item. */
export interface MenuItemRecord {
  /** Submenu level owned by a submenu trigger item, otherwise `null`. */
  readonly subLevel: MenuLevelContextValue | null;
}

/** Point in viewport coordinates. */
export interface MenuPoint {
  readonly x: number;
  readonly y: number;
}

/** Item registry, highlight, and pointer-grace state owned by one rendered content. */
export interface MenuContentContextValue {
  readonly level: MenuLevelContextValue;
  readonly registry: CollectionRegistry<string, MenuItemRecord>;
  /** Pending typeahead query; Space extends a pending query instead of selecting. */
  readonly typeaheadQuery: Readonly<ShallowRef<string>>;
  /** Focus an item by key. */
  readonly focusItem: (key: string) => void;
  /** Focus the content container and clear the highlight. */
  readonly focusContent: () => void;
  /** Whether a pointer event is travelling through the safe triangle to an open submenu. */
  readonly isPointerInGrace: (event: PointerEvent) => boolean;
  /** Arm the safe triangle from `origin` to the open submenu content. */
  readonly startGrace: (origin: MenuPoint, target: HTMLElement) => void;
  /** Clear any armed safe triangle. */
  readonly clearGrace: () => void;
}

/** Checked state published by checkbox and radio items to their indicators. */
export interface MenuItemIndicatorContextValue {
  readonly state: ComputedRef<MenuItemCheckedState>;
}

/**
 * Value-erased radio group contract.
 *
 * Vue injection cannot carry a type parameter from `MenuRadioGroup<Value>` to
 * `MenuRadioItem<Value>`, so the group publishes value-erased operations and
 * each generic SFC re-establishes its own parameter at the public boundary.
 */
export interface MenuRadioGroupContextValue {
  readonly isChecked: (value: unknown) => boolean;
  readonly select: (value: unknown, event: Event | null) => boolean;
  readonly disabled: ComputedRef<boolean>;
}

/** Group labelling contract shared by MenuGroup and MenuLabel. */
export interface MenuGroupContextValue {
  readonly labelId: ShallowRef<string | null>;
}

export const menuTreeContext = createContext<MenuTreeContextValue>("MenuTree");
export const menuLevelContext = createContext<MenuLevelContextValue>("MenuLevel");
export const menuContentContext = createContext<MenuContentContextValue>("MenuContent");
export const menuItemIndicatorContext =
  createContext<MenuItemIndicatorContextValue>("MenuItemIndicator");
export const menuRadioGroupContext = createContext<MenuRadioGroupContextValue>("MenuRadioGroup");
export const menuGroupContext = createContext<MenuGroupContextValue>("MenuGroup");
