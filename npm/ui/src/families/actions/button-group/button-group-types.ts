import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Accessible grouping semantics emitted by {@link ButtonGroup}. */
export type ButtonGroupRole = "group" | "toolbar";

/** Directional layout hint used by toolbar arrow-key roving focus. */
export type ButtonGroupOrientation = "horizontal" | "vertical";

/** Stable root state exposed for consumer-owned styling and tests. */
export type ButtonGroupState = "disabled" | "idle";

/** Stable item state exposed for consumer-owned styling and tests. */
export type ButtonGroupItemState = "disabled" | "idle";

/** State exposed to the ButtonGroup default slot. */
export interface ButtonGroupSlotState {
  /** Whether the group suppresses every item activation. */
  readonly disabled: boolean;

  /** Directional layout hint used by toolbar arrow-key roving focus. */
  readonly orientation: ButtonGroupOrientation;

  /** Accessible grouping role rendered by the root. */
  readonly role: ButtonGroupRole;

  /** Whether items participate in a single-tabstop roving focus model. */
  readonly rovingFocus: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ButtonGroupState;
}

/** Public instance exposed by ButtonGroup. */
export interface ButtonGroupExpose extends ButtonGroupSlotState {
  /** Currently active item value, or `null` before an item is focusable. */
  readonly activeValue: string | null;

  /** Rendered root element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Move focus to the active or first enabled item. */
  readonly focus: (options?: FocusOptions) => void;

  /** Move focus to an enabled item by value and report whether it was found. */
  readonly focusValue: (value: string, options?: FocusOptions) => boolean;
}

/** State exposed to a ButtonGroupItem slot. */
export interface ButtonGroupItemSlotState {
  /** Item value used by the group navigation and press contract. */
  readonly value: string;

  /** Whether this item or its group suppresses activation. */
  readonly disabled: boolean;

  /** Directional layout hint inherited from the group. */
  readonly orientation: ButtonGroupOrientation;

  /** Stable state token for styling and tests. */
  readonly state: ButtonGroupItemState;
}

/** Public instance exposed by ButtonGroupItem. */
export interface ButtonGroupItemExpose extends ButtonGroupItemSlotState {
  /** Rendered item element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Move focus to the rendered item. */
  readonly focus: (options?: FocusOptions) => void;
}
