import type { ComputedRef, MaybeRefOrGetter } from "vue";

import type { DeterministicId } from "./deterministic-id.ts";

/** Options accepted by {@link useFieldWiring}. */
export interface FieldWiringOptions {
  /**
   * Consumer-owned control id. `null` and `undefined` select the generated
   * deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: MaybeRefOrGetter<string | null | undefined>;

  /**
   * Whether the field currently fails validation.
   *
   * @default false
   */
  readonly invalid?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Whether a description element is rendered with `descriptionProps`.
   *
   * @default false
   */
  readonly hasDescription?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Whether an error message element is rendered with `errorMessageProps`
   * while the field is invalid.
   *
   * @default true
   */
  readonly hasErrorMessage?: MaybeRefOrGetter<boolean | undefined>;
}

/** Attributes for the label element naming the control. */
export interface FieldLabelProps {
  readonly id: string;
  readonly for: string;
}

/** Attributes for the labelled form control. */
export interface FieldControlProps {
  readonly id: string;
  readonly "aria-labelledby": string;
  readonly "aria-describedby": string | undefined;
  readonly "aria-errormessage": string | undefined;
  readonly "aria-invalid": "true" | undefined;
}

/** Attributes for a description or error message element. */
export interface FieldTextProps {
  readonly id: string;
}

/** Accessible name, description, and error message wiring for one field. */
export interface FieldWiringController {
  /** Stable id of the form control. */
  readonly fieldId: ComputedRef<DeterministicId>;

  /** Stable id of the label element. */
  readonly labelId: ComputedRef<DeterministicId>;

  /** Stable id of the description element. */
  readonly descriptionId: ComputedRef<DeterministicId>;

  /** Stable id of the error message element. */
  readonly errorMessageId: ComputedRef<DeterministicId>;

  /** Whether the control is currently marked invalid. */
  readonly isInvalid: ComputedRef<boolean>;

  /** Bind to the label element. */
  readonly labelProps: ComputedRef<FieldLabelProps>;

  /** Bind to the form control. */
  readonly fieldProps: ComputedRef<FieldControlProps>;

  /** Bind to the description element. */
  readonly descriptionProps: ComputedRef<FieldTextProps>;

  /** Bind to the error message element. */
  readonly errorMessageProps: ComputedRef<FieldTextProps>;
}
