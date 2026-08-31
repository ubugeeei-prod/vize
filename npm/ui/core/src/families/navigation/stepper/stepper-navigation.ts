import type { CollectionItem } from "../../foundations/collection/collection.ts";
import type { StepperCollectionValue } from "./stepper-context.ts";
import type { StepperItemState, StepperNavigationMode, StepperValue } from "./stepper-types.ts";

type StepperItemSnapshot = CollectionItem<string, StepperCollectionValue>;

export interface StepperSelectableOptions {
  readonly disabled: boolean;
  readonly item: StepperItemSnapshot | undefined;
  readonly items: readonly StepperItemSnapshot[];
  readonly navigationMode: StepperNavigationMode;
  readonly selected: StepperValue;
  readonly value: string;
}

/** Return a stable visual state for one Stepper item. */
export function getStepperItemState(options: {
  readonly completed: boolean;
  readonly current: boolean;
  readonly disabled: boolean;
}): StepperItemState {
  if (options.disabled) return "disabled";
  if (options.current) return "current";
  if (options.completed) return "completed";
  return "pending";
}

/** Return whether linear Stepper rules allow activating a target value. */
export function stepperLinearAllows(
  items: readonly StepperItemSnapshot[],
  selected: StepperValue,
  value: string,
): boolean {
  const enabledItems = items.filter((item) => !item.disabled);
  const targetIndex = enabledItems.findIndex((item) => item.key === value);
  if (targetIndex < 0) return false;

  const selectedIndex =
    selected === null ? -1 : enabledItems.findIndex((item) => item.key === selected);
  if (selectedIndex >= 0 && targetIndex <= selectedIndex) return true;

  return enabledItems.slice(0, targetIndex).every((item) => item.value.completed.value);
}

/** Return whether a Stepper value can be selected under the current root state. */
export function isStepperSelectable(options: StepperSelectableOptions): boolean {
  if (options.disabled || options.item === undefined || options.item.disabled) return false;
  if (options.selected === options.value) return true;
  return (
    options.navigationMode === "free" ||
    stepperLinearAllows(options.items, options.selected, options.value)
  );
}

/** Return the next or previous enabled Stepper value from a current value. */
export function getRelativeStepperValue(
  items: readonly StepperItemSnapshot[],
  selected: StepperValue,
  direction: "next" | "previous",
): StepperValue {
  if (items.length === 0) return null;
  const selectedIndex = selected === null ? -1 : items.findIndex((item) => item.key === selected);
  const targetIndex =
    direction === "next"
      ? selectedIndex < 0
        ? 0
        : selectedIndex + 1
      : selectedIndex < 0
        ? items.length - 1
        : selectedIndex - 1;
  return items[targetIndex]?.key ?? null;
}
