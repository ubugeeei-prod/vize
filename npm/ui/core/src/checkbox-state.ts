/** Visual state exposed by the Checkbox Native CSS contract. */
export type CheckboxState = "checked" | "unchecked" | "indeterminate";

/** Resolve the visual state while giving the mixed state precedence. */
export function getCheckboxState(checked: boolean, indeterminate: boolean): CheckboxState {
  if (indeterminate) return "indeterminate";
  return checked ? "checked" : "unchecked";
}
