import type { PrimitiveElement } from "../../../primitive.ts";

/** Selection mode owned by the ToggleGroup root. */
export type ToggleGroupType = "single" | "multiple";

/** Controlled or uncontrolled ToggleGroup value. */
export type ToggleGroupValue = string | readonly string[] | null;

/** Directional layout hint used for roving keyboard focus. */
export type ToggleGroupOrientation = "horizontal" | "vertical";

/** State exposed by the ToggleGroup root data contract. */
export type ToggleGroupState = "disabled" | "empty" | "selected";

/** State exposed by each ToggleGroup item data contract. */
export type ToggleGroupItemState = "disabled" | "pressed" | "unpressed";

/** State exposed to the ToggleGroup default slot. */
export interface ToggleGroupSlotState {
  /** Current selected value: a string, an array for multiple mode, or null for no single selection. */
  readonly value: ToggleGroupValue;

  /** Current pressed values normalized to an immutable array. */
  readonly pressedValues: readonly string[];

  /** Whether the group suppresses every item activation. */
  readonly disabled: boolean;

  /** Selection mode used by item activation. */
  readonly type: ToggleGroupType;

  /** Directional layout hint used by roving focus. */
  readonly orientation: ToggleGroupOrientation;

  /** Stable state token for styling and tests. */
  readonly state: ToggleGroupState;
}

/** Public instance exposed by ToggleGroup. */
export interface ToggleGroupExpose extends ToggleGroupSlotState {
  /** Rendered root element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Move focus to the active, selected, or first enabled item. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a group value update and report whether it differs. */
  readonly setValue: (value: ToggleGroupValue) => boolean;

  /** Toggle one item value programmatically and report whether it differs. */
  readonly toggleValue: (value: string, pressed?: boolean) => boolean;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;
}

/** State exposed to a ToggleGroupItem slot. */
export interface ToggleGroupItemSlotState {
  /** Item value used by the group selection model. */
  readonly value: string;

  /** Whether this item is pressed by the group. */
  readonly pressed: boolean;

  /** Whether this item or its group suppresses activation. */
  readonly disabled: boolean;

  /** Directional layout hint inherited from the group. */
  readonly orientation: ToggleGroupOrientation;

  /** Stable state token for styling and tests. */
  readonly state: ToggleGroupItemState;
}

/** Public instance exposed by ToggleGroupItem. */
export interface ToggleGroupItemExpose extends ToggleGroupItemSlotState {
  /** Rendered item element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Move focus to the rendered item. */
  readonly focus: (options?: FocusOptions) => void;
}
