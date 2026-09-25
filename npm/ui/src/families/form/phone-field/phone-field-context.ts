import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { PhoneCountry } from "./phone-field-country.ts";

/** Type-erased PhoneField state shared with its parts. */
export interface PhoneFieldContextValue {
  readonly inputId: ComputedRef<string>;
  readonly countries: ComputedRef<readonly PhoneCountry[]>;
  readonly country: ComputedRef<PhoneCountry>;
  readonly nationalNumber: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly ariaLabel: ComputedRef<string | undefined>;
  readonly ariaLabelledby: ComputedRef<string | undefined>;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
  readonly ariaInvalid: ComputedRef<"grammar" | "spelling" | "true" | undefined>;
  readonly setCountryCode: (code: string) => boolean;
  readonly setNationalNumber: (digits: string) => void;
  readonly setText: (text: string) => boolean;
}

/** Typed context shared by PhoneField, its input, and its country select. */
export const phoneFieldContext = createContext<PhoneFieldContextValue>("PhoneField");
