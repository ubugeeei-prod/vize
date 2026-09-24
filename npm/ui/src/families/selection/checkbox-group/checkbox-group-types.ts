/** Values accepted by the native `aria-invalid` attribute. */
export type CheckboxGroupAriaInvalid = boolean | "grammar" | "spelling";

/** Layout direction published for styling. */
export type CheckboxGroupOrientation = "horizontal" | "vertical";

/** Selection summary published through `data-state`. */
export type CheckboxGroupState = "all" | "disabled" | "none" | "some";

/** Visual state of one checkbox (items and the select-all parent). */
export type CheckboxGroupItemState = "checked" | "indeterminate" | "unchecked";

/** Identity key used to compare option values, for example an id. */
export type CheckboxGroupKey = string | number | bigint | boolean | symbol | null | undefined;

/** State exposed to the CheckboxGroup slot and instance. */
export interface CheckboxGroupSlotState<Value> {
  /** Selected values in option order. */
  readonly values: readonly Value[];

  /** Every option, in order. */
  readonly options: readonly Value[];

  /** Whether every enabled option is selected. */
  readonly allSelected: boolean;

  /** Whether at least one but not every enabled option is selected. */
  readonly someSelected: boolean;

  /** Whether the whole group is disabled. */
  readonly disabled: boolean;

  /** Selection summary. */
  readonly state: CheckboxGroupState;
}

/** Public instance API of CheckboxGroup. */
export interface CheckboxGroupExpose<Value> extends CheckboxGroupSlotState<Value> {
  /** Rendered group element. */
  readonly root: HTMLDivElement | null;

  /** Whether an option value is selected. */
  readonly isSelected: (value: Value) => boolean;

  /** Select or clear one option; returns whether the selection changed. */
  readonly setSelected: (value: Value, selected: boolean) => boolean;

  /** Select or clear every enabled option; returns whether the selection changed. */
  readonly setAll: (selected: boolean) => boolean;

  /** Restore the default selection; returns whether it changed. */
  readonly reset: () => boolean;
}
