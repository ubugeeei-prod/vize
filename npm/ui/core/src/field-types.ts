import type { PrimitiveElement } from "./primitive.ts";
import type { FieldControlProps, FieldLabelProps, FieldTextProps } from "./field-wiring-types.ts";
import type { FormFieldError } from "./form-types.ts";

/** State exposed by the Field root default slot. */
export interface FieldRootSlotState {
  /** Stable id for the consumer-rendered form control. */
  readonly id: string;

  /** Normalized HTML form field name. */
  readonly name: string;

  /** Whether this field is currently invalid. */
  readonly invalid: boolean;

  /** Current normalized errors for this field only. */
  readonly errors: readonly FormFieldError[];

  /** First current error message for this field. */
  readonly errorMessage: string | undefined;

  /** Attributes to bind to the consumer-rendered form control. */
  readonly fieldProps: FieldControlProps;

  /** Attributes bound by FieldLabel. */
  readonly labelProps: FieldLabelProps;

  /** Attributes bound by FieldDescription. */
  readonly descriptionProps: FieldTextProps;

  /** Attributes bound by FieldErrorMessage. */
  readonly errorMessageProps: FieldTextProps;
}

/** State exposed by FieldLabel's default slot. */
export interface FieldLabelSlotState {
  /** Stable id for the rendered label element. */
  readonly id: string;

  /** Stable id for the labelled control. */
  readonly for: string;

  /** Normalized HTML form field name. */
  readonly name: string;

  /** Whether this field is currently invalid. */
  readonly invalid: boolean;
}

/** State exposed by FieldDescription's default slot. */
export interface FieldDescriptionSlotState {
  /** Stable id for the rendered description element. */
  readonly id: string;

  /** Normalized HTML form field name. */
  readonly name: string;

  /** Whether this field is currently invalid. */
  readonly invalid: boolean;
}

/** State exposed by FieldErrorMessage's default slot. */
export interface FieldErrorMessageSlotState {
  /** Stable id for the rendered error message element. */
  readonly id: string;

  /** Normalized HTML form field name. */
  readonly name: string;

  /** Whether this field is currently invalid. */
  readonly invalid: boolean;

  /** First current error message for this field. */
  readonly message: string | undefined;

  /** Current normalized errors for this field only. */
  readonly errors: readonly FormFieldError[];
}

/** Public instance exposed by Field. */
export interface FieldRootExpose extends FieldRootSlotState {
  /** Rendered root element or component instance. */
  readonly element: PrimitiveElement | null;
}

/** Public instance exposed by FieldLabel. */
export interface FieldLabelExpose {
  /** Rendered label element or component instance. */
  readonly element: PrimitiveElement | null;
}

/** Public instance exposed by FieldDescription. */
export interface FieldDescriptionExpose {
  /** Rendered description element or component instance. */
  readonly element: PrimitiveElement | null;
}

/** Public instance exposed by FieldErrorMessage. */
export interface FieldErrorMessageExpose {
  /** Rendered error message element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Whether this field is currently invalid. */
  readonly invalid: boolean;

  /** First current error message for this field. */
  readonly message: string | undefined;
}
