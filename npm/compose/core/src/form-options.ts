/**
 * Options for {@link useForm}: when validation runs, and what reset keeps.
 */

import type { MaybeRefOrGetter } from "vue";

import type { StandardSchemaV1 } from "./standard-schema.ts";
import type { FormFieldValidators, FormValidator } from "./form-path-types.ts";

/** When validation runs automatically. */
export type FormValidationTrigger = "change" | "blur" | "submit";

/** How many messages each path keeps after validation. */
export type FormCriteriaMode = "all" | "first";

/** Discriminated outcome of validation and submission. */
export type FormResult<Output> =
  | {
      /** Every validator passed. */
      readonly status: "valid";
      /** Schema output, or the values when no schema is configured. */
      readonly value: Output;
    }
  | {
      /** At least one validator failed. */
      readonly status: "invalid";
      /** Errors keyed by path. */
      readonly errors: Readonly<Record<string, readonly string[]>>;
    }
  | {
      /** A newer validation started before this one finished. */
      readonly status: "superseded";
    };

/** Options for `useForm`. */
export interface UseFormOptions<Values extends object, Output> {
  /** Initial values, or a factory producing fresh initial values. */
  readonly initialValues: Values | (() => Values);

  /**
   * Standard Schema v1 validator for the whole form. Its output becomes the
   * submitted value.
   *
   * @default undefined
   */
  readonly schema?: StandardSchemaV1<NoInfer<Values>, Output>;

  /**
   * Plain whole-form validator.
   *
   * @default undefined
   */
  readonly validate?: NoInfer<FormValidator<Values>>;

  /**
   * Plain per-field validators keyed by path.
   *
   * @default {}
   */
  readonly validators?: NoInfer<FormFieldValidators<Values>>;

  /**
   * When fields validate before the first submit.
   *
   * @default "submit"
   */
  readonly validateOn?: FormValidationTrigger;

  /**
   * When fields re-validate after the first submit.
   *
   * @default "change"
   */
  readonly revalidateOn?: FormValidationTrigger;

  /**
   * `"all"` records every message. `"first"` keeps the earliest message in
   * source order: schema, then the form validator, then the field validator.
   *
   * @default "all"
   */
  readonly criteriaMode?: FormCriteriaMode;

  /**
   * When true, value writes, field-array edits, and submission do nothing.
   * `setError`, `setErrors`, and `reset` still run. A ref or getter tracks.
   *
   * @default false
   */
  readonly disabled?: MaybeRefOrGetter<boolean>;

  /**
   * Default submit handler for `submit`.
   *
   * @default undefined
   */
  readonly onSubmit?: (value: Output, context: { readonly signal: AbortSignal }) => unknown;
}

/** Options for `setValue`. */
export interface SetFieldValueOptions {
  /**
   * Mark the field touched.
   *
   * @default false
   */
  readonly touch?: boolean;

  /**
   * Validate the field after setting it.
   *
   * @default true while the active trigger is "change"
   */
  readonly validate?: boolean;
}

/**
 * What {@link FormControls.reset} retains. Every omitted flag clears that state.
 */
export interface FormResetOptions {
  /**
   * Keep values that differ from the previous initial values. Clean fields
   * take the reset values. Added keys stay, and removed keys stay removed.
   *
   * @default false
   */
  readonly keepDirtyValues?: boolean;

  /**
   * Keep the current errors.
   *
   * @default false
   */
  readonly keepErrors?: boolean;

  /**
   * Keep touched fields.
   *
   * @default false
   */
  readonly keepTouched?: boolean;

  /**
   * Keep the submit count.
   *
   * @default false
   */
  readonly keepSubmitCount?: boolean;

  /**
   * Keep whether the latest finished submit succeeded.
   *
   * @default false
   */
  readonly keepIsSubmitSuccessful?: boolean;
}
