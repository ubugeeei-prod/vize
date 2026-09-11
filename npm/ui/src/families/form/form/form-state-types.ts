import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import type { ErrorSummaryField } from "../error-summary/error-summary-types.ts";
import type {
  FormErrorSummaryFieldOptions,
  FormFieldError,
  FormValidationResult,
  FormValidationSuccess,
  StandardSchemaV1,
  StandardSchemaValidationOptions,
} from "./form-types.ts";

/** Element-like focus target registered for invalid-field focus movement. */
export interface FormFocusableElement {
  /** Move browser focus to the control. */
  focus(options?: FocusOptions): void;
}

/** One tracked field's aggregate form state. */
export interface FormFieldState {
  /** Normalized HTML form field name. */
  readonly name: string;

  /** Whether the field value has been changed by the consumer. */
  readonly isDirty: boolean;

  /** Whether the field has received focus or equivalent visitation. */
  readonly isVisited: boolean;

  /** Whether the field has been blurred or equivalent confirmation. */
  readonly isTouched: boolean;

  /** Whether this field currently has at least one normalized error. */
  readonly isInvalid: boolean;

  /** Current errors for this field only. */
  readonly errors: readonly FormFieldError[];

  /** First current error for this field. */
  readonly firstError: FormFieldError | undefined;

  /** First current error message for this field. */
  readonly errorMessage: string | undefined;
}

/** Options accepted by {@link normalizeNativeConstraintErrors}. */
export interface NativeConstraintErrorOptions {
  /**
   * Resolve a field name for one invalid native form control.
   *
   * @default the control's `name` attribute
   */
  readonly nameForControl?: (control: Element) => string | null | undefined;

  /**
   * Resolve the message for one invalid native form control.
   *
   * @default the control's `validationMessage`, or `"Invalid value"`
   */
  readonly messageForControl?: (control: Element) => string | undefined;
}

/** Options accepted by {@link useFormState}. */
export interface FormStateOptions<
  Input = unknown,
  Output = Input,
> extends FormErrorSummaryFieldOptions {
  /**
   * Standard Schema used by `validate` and `submit`.
   *
   * @default undefined
   */
  readonly schema?: StandardSchemaV1<Input, Output>;

  /**
   * Initial normalized errors.
   *
   * @default []
   */
  readonly initialErrors?: readonly FormFieldError[];

  /**
   * Submit callback invoked only for the latest valid submission.
   *
   * @default undefined
   */
  readonly onSubmit?: (value: Output, result: FormValidationSuccess<Output>) => unknown;

  /**
   * Move focus to the first registered invalid field after an invalid submit.
   *
   * @default true
   */
  readonly focusFirstInvalidOnSubmit?: boolean;
}

/** Per-submit options layered on top of Standard Schema normalization. */
export interface FormSubmitOptions<Output> extends StandardSchemaValidationOptions {
  /**
   * Submit callback for this call. Overrides the controller default.
   *
   * @default controller `onSubmit`
   */
  readonly onSubmit?: (value: Output, result: FormValidationSuccess<Output>) => unknown;

  /**
   * Move focus to the first registered invalid field after this invalid submit.
   *
   * @default controller `focusFirstInvalidOnSubmit`
   */
  readonly focusFirstInvalid?: boolean;
}

/** Options accepted by {@link FormStateController.reset}. */
export interface FormStateResetOptions {
  /**
   * Errors to install after reset.
   *
   * @default []
   */
  readonly errors?: readonly FormFieldError[];

  /**
   * Field names that should remain dirty after reset.
   *
   * @default []
   */
  readonly dirtyFields?: readonly string[];

  /**
   * Field names that should remain touched after reset.
   *
   * @default []
   */
  readonly touchedFields?: readonly string[];

  /**
   * Field names that should remain visited after reset.
   *
   * @default []
   */
  readonly visitedFields?: readonly string[];
}

/** Reactive controller for Standard Schema form state and submission. */
export interface FormStateController<Input = unknown, Output = Input> {
  /** Current normalized validation and server errors. */
  readonly errors: Readonly<Ref<readonly FormFieldError[]>>;

  /** Error-summary fields derived from current errors. */
  readonly summaryFields: ComputedRef<readonly ErrorSummaryField[]>;

  /** Whether at least one current error exists. */
  readonly hasErrors: ComputedRef<boolean>;

  /** Whether any field has been marked dirty. */
  readonly isDirty: ComputedRef<boolean>;

  /** Whether any field has been marked touched. */
  readonly isTouched: ComputedRef<boolean>;

  /** Whether any field has been marked visited. */
  readonly isVisited: ComputedRef<boolean>;

  /** Whether at least one schema validation is currently pending. */
  readonly isValidating: ComputedRef<boolean>;

  /** Whether at least one submission is currently pending. */
  readonly isSubmitting: ComputedRef<boolean>;

  /** Number of submit attempts made through this controller. */
  readonly submitCount: Readonly<Ref<number>>;

  /** Aggregate state for one field name. */
  fieldState(name: MaybeRefOrGetter<string>): ComputedRef<FormFieldState>;

  /** Register a focus target for first-invalid focus movement. */
  registerField(name: string, element: FormFocusableElement | null | undefined): () => void;

  /** Mark or unmark one field as dirty. */
  markFieldDirty(name: string, dirty?: boolean): void;

  /** Mark or unmark one field as touched. */
  markFieldTouched(name: string, touched?: boolean): void;

  /** Mark or unmark one field as visited. */
  markFieldVisited(name: string, visited?: boolean): void;

  /** Replace current errors with normalized validation or server errors. */
  setErrors(errors: readonly FormFieldError[]): void;

  /** Replace current errors with server-returned normalized field errors. */
  setServerErrors(errors: readonly FormFieldError[]): void;

  /** Clear current errors. */
  clearErrors(): void;

  /** Reset errors and tracked field interaction state. */
  reset(options?: FormStateResetOptions): void;

  /** Validate with the configured Standard Schema, discarding stale side effects. */
  validate(
    value: Input,
    options?: StandardSchemaValidationOptions,
  ): Promise<FormValidationResult<Output>>;

  /** Validate, submit the latest valid result, and focus the first invalid field. */
  submit(value: Input, options?: FormSubmitOptions<Output>): Promise<FormValidationResult<Output>>;

  /** Focus the first registered field with a current error. */
  focusFirstInvalid(options?: FocusOptions): boolean;
}
