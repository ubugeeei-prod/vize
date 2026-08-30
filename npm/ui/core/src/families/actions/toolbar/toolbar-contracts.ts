import type { PrimitiveElement } from "../../../primitive.ts";
import type { ToolbarItemSlotState, ToolbarSlotState } from "./toolbar-types.ts";

/** Slots exposed by the root Toolbar component. */
export interface ToolbarSlots {
  /** Renders grouped action controls with toolbar navigation state. */
  default(props: ToolbarSlotState): unknown;
}

/** Slots exposed by ToolbarItem. */
export interface ToolbarItemSlots {
  /** Renders item contents with current availability and navigation state. */
  default(props: ToolbarItemSlotState): unknown;
}

/** Public component instance state exposed by the root Toolbar component. */
export interface ToolbarExpose extends ToolbarSlotState {
  /** Currently active item value, or `null` before an item is focusable. */
  readonly activeValue: string | null;

  /** Rendered root element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Move focus to the active or first enabled item. */
  readonly focus: (options?: FocusOptions) => void;

  /** Move focus to an enabled item by value and report whether it was found. */
  readonly focusValue: (value: string, options?: FocusOptions) => boolean;
}

/** Public component instance state exposed by ToolbarItem. */
export interface ToolbarItemExpose extends ToolbarItemSlotState {
  /** Rendered item element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Move focus to the rendered item. */
  readonly focus: (options?: FocusOptions) => void;
}
