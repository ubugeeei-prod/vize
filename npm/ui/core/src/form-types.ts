import type { ComputedRef, MaybeRefOrGetter } from "vue";

import type { ErrorSummaryField } from "./error-summary-types.ts";

/** Structural Standard Schema V1 contract accepted by the form utilities. */
export interface StandardSchemaV1<Input = unknown, Output = Input> {
  /** Standard Schema metadata and validation entry point. */
  readonly "~standard": StandardSchemaV1.Props<Input, Output>;
}

export namespace StandardSchemaV1 {
  /** Standard Schema metadata and validation entry point. */
  export interface Props<Input = unknown, Output = Input> {
    /** Version number of the Standard Schema contract. */
    readonly version: 1;

    /** Schema-library vendor name. */
    readonly vendor: string;

    /** Validate unknown input and return either a value or issues. */
    readonly validate: (
      value: unknown,
      options?: StandardSchemaV1.Options,
    ) => Result<Output> | Promise<Result<Output>>;

    /** Optional phantom input and output types advertised by the schema. */
    readonly types?: Types<Input, Output> | undefined;
  }

  /** Standard Schema validation result. */
  export type Result<Output> = SuccessResult<Output> | FailureResult;

  /** Successful Standard Schema validation result. */
  export interface SuccessResult<Output> {
    /** Typed output value. */
    readonly value: Output;

    /** Success is represented by no issues. */
    readonly issues?: undefined;
  }

  /** Standard Schema validation options. */
  export interface Options {
    /** Vendor-specific options passed through to the schema library. */
    readonly libraryOptions?: Record<string, unknown> | undefined;
  }

  /** Failed Standard Schema validation result. */
  export interface FailureResult {
    /** Failed-validation issues. */
    readonly issues: ReadonlyArray<Issue>;
  }

  /** One Standard Schema validation issue. */
  export interface Issue {
    /** Human-readable validation message. */
    readonly message: string;

    /** Path of the invalid value, if known. */
    readonly path?: ReadonlyArray<PropertyKey | PathSegment> | undefined;
  }

  /** Object path segment used by some schema libraries. */
  export interface PathSegment {
    /** Key represented by this segment. */
    readonly key: PropertyKey;
  }

  /** Phantom input and output types advertised by the schema. */
  export interface Types<Input = unknown, Output = Input> {
    /** Input type accepted by the schema. */
    readonly input: Input;

    /** Output type returned by the schema. */
    readonly output: Output;
  }

  /** Infer the input type advertised by a Standard Schema. */
  export type InferInput<Schema extends StandardSchemaV1> = NonNullable<
    Schema["~standard"]["types"]
  >["input"];

  /** Infer the output type advertised by a Standard Schema. */
  export type InferOutput<Schema extends StandardSchemaV1> = NonNullable<
    Schema["~standard"]["types"]
  >["output"];
}

/** Property key after a Standard Schema path segment has been unwrapped. */
export type FormPathKey = PropertyKey;

/** One normalized field validation error. */
export interface FormFieldError {
  /** HTML form field name derived from the issue path. */
  readonly name: string;

  /** Human-readable validation message. */
  readonly message: string;

  /** Unwrapped Standard Schema path keys. */
  readonly path: readonly FormPathKey[];
}

/** Options for converting Standard Schema issues into field errors. */
export interface FormFieldErrorOptions {
  /**
   * Convert an unwrapped Standard Schema issue path into a form field name.
   *
   * @default dot/bracket path formatting
   */
  readonly nameForPath?: (path: readonly FormPathKey[], issue: StandardSchemaV1.Issue) => string;
}

/** Options for converting normalized field errors into error-summary fields. */
export interface FormErrorSummaryFieldOptions {
  /**
   * Resolve the document id for a field error.
   *
   * @default the normalized field name, or `rootId` for form-level errors
   */
  readonly idForName?: (name: string, error: FormFieldError) => string | null | undefined;

  /**
   * Resolve the accessible field label displayed before the summary message.
   *
   * @default undefined
   */
  readonly labelForName?: (name: string, error: FormFieldError) => string | undefined;

  /**
   * Document id used for form-level errors with no field path.
   *
   * @default "form"
   */
  readonly rootId?: string;

  /**
   * Label used for form-level errors with no field path.
   *
   * @default undefined
   */
  readonly rootLabel?: string;
}

/** Normalized successful form validation state. */
export interface FormValidationSuccess<Output> {
  /** Whether validation succeeded. */
  readonly valid: true;

  /** Typed output value from the schema. */
  readonly value: Output;

  /** Normalized field errors. Empty on success. */
  readonly errors: readonly FormFieldError[];

  /** Error-summary fields. Empty on success. */
  readonly summaryFields: readonly [];
}

/** Normalized failed form validation state. */
export interface FormValidationFailure {
  /** Whether validation succeeded. */
  readonly valid: false;

  /** Normalized field errors. */
  readonly errors: readonly FormFieldError[];

  /** Error-summary fields, one per document id. */
  readonly summaryFields: readonly ErrorSummaryField[];
}

/** Normalized form validation state. */
export type FormValidationResult<Output> = FormValidationSuccess<Output> | FormValidationFailure;

/** Options for validating and normalizing a Standard Schema form result. */
export interface StandardSchemaValidationOptions
  extends FormFieldErrorOptions, FormErrorSummaryFieldOptions, StandardSchemaV1.Options {}

/** Options accepted by {@link useFormField}. */
export interface FormFieldOptions {
  /** Normalized HTML form field name. */
  readonly name: MaybeRefOrGetter<string>;

  /**
   * Full normalized form error list.
   *
   * @default []
   */
  readonly errors?: MaybeRefOrGetter<readonly FormFieldError[] | undefined>;
}

/** Reactive normalized errors and invalid state for one form field. */
export interface FormFieldController {
  /** Normalized HTML form field name. */
  readonly name: ComputedRef<string>;

  /** Whether this field currently has at least one normalized error. */
  readonly isInvalid: ComputedRef<boolean>;

  /** Current errors for this field only. */
  readonly errors: ComputedRef<readonly FormFieldError[]>;

  /** First current error for this field. */
  readonly firstError: ComputedRef<FormFieldError | undefined>;

  /** First current error message for this field. */
  readonly errorMessage: ComputedRef<string | undefined>;
}

/** Options accepted by {@link useFormErrorSummary}. */
export interface FormErrorSummaryOptions extends Omit<FormErrorSummaryFieldOptions, "rootId"> {
  /**
   * Full normalized form error list.
   *
   * @default []
   */
  readonly errors?: MaybeRefOrGetter<readonly FormFieldError[] | undefined>;

  /**
   * Document id used for form-level errors with no field path.
   *
   * @default "form"
   */
  readonly rootId?: MaybeRefOrGetter<string | undefined>;
}

/** Reactive error-summary fields derived from normalized form errors. */
export interface FormErrorSummaryController {
  /** Fields ready to pass into the error-summary component or controller. */
  readonly fields: ComputedRef<readonly ErrorSummaryField[]>;

  /** Whether at least one summary field is currently available. */
  readonly hasErrors: ComputedRef<boolean>;
}
