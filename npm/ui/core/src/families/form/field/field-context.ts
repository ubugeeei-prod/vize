import type { ComputedRef } from "vue";

import { createContext } from "../../../context.ts";
import type {
  FieldControlProps,
  FieldLabelProps,
  FieldTextProps,
} from "../field-wiring/field-wiring-types.ts";
import type { FormFieldError } from "../form/form-types.ts";

/** Shared state for the Field compound components. */
export interface FieldContextValue {
  readonly id: ComputedRef<string>;
  readonly name: ComputedRef<string>;
  readonly invalid: ComputedRef<boolean>;
  readonly errors: ComputedRef<readonly FormFieldError[]>;
  readonly errorMessage: ComputedRef<string | undefined>;
  readonly fieldProps: ComputedRef<FieldControlProps>;
  readonly labelProps: ComputedRef<FieldLabelProps>;
  readonly descriptionProps: ComputedRef<FieldTextProps>;
  readonly errorMessageProps: ComputedRef<FieldTextProps>;
}

export const fieldContext = createContext<FieldContextValue>("Field");
