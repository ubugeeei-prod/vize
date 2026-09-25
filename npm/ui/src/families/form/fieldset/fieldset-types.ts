import type { FormFieldError } from "../form/form-types.ts";

/** State published through the Fieldset `data-state` contract. */
export type FieldsetState = "disabled" | "invalid" | "valid";

/** State exposed to Fieldset slots and instances. */
export interface FieldsetSlotState {
  /** Fieldset id; description and error ids derive from it. */
  readonly id: string;

  /** Whether the group is invalid. */
  readonly invalid: boolean;

  /** Whether the native fieldset (and every descendant control) is disabled. */
  readonly disabled: boolean;

  /** Errors whose `name` matches the fieldset `name`. */
  readonly errors: readonly FormFieldError[];

  /** First matching error message. */
  readonly errorMessage: string | undefined;

  /** Stable state token. */
  readonly state: FieldsetState;
}

/** Slot state for FieldsetErrorMessage. */
export interface FieldsetErrorMessageSlotState {
  /** Error element id. */
  readonly id: string;

  /** First matching error message. */
  readonly message: string | undefined;

  /** Every matching error. */
  readonly errors: readonly FormFieldError[];
}
