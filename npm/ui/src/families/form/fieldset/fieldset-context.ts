import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { FormFieldError } from "../form/form-types.ts";

/** Shared state for Fieldset parts. */
export interface FieldsetContextValue {
  readonly descriptionId: ComputedRef<string>;
  readonly errorMessageId: ComputedRef<string>;
  readonly invalid: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly errors: ComputedRef<readonly FormFieldError[]>;
  readonly errorMessage: ComputedRef<string | undefined>;
}

/** Typed context shared by Fieldset and its legend, description, and error message. */
export const fieldsetContext = createContext<FieldsetContextValue>("Fieldset");
