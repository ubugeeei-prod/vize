import type { PhoneCountry } from "./phone-field-country.ts";

/** Values accepted by the native `aria-invalid` attribute. */
export type PhoneFieldAriaInvalid = boolean | "grammar" | "spelling";

/** State published through the PhoneField `data-state` contract. */
export type PhoneFieldState = "complete" | "disabled" | "empty" | "incomplete";

/** State exposed to PhoneField slots and instances. */
export interface PhoneFieldSlotState<Code extends string> {
  /** Selected country. */
  readonly country: PhoneCountry<Code>;

  /** National significant number (digits only). */
  readonly nationalNumber: string;

  /** E.164 value (`+819012345678`), or `""` while empty. */
  readonly e164: string;

  /** National number formatted with the country pattern. */
  readonly formatted: string;

  /** International display form (`+81 90-1234-5678`), or `""` while empty. */
  readonly international: string;

  /** Whether every slot of the country pattern is filled. */
  readonly complete: boolean;

  /** Whether the field is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: PhoneFieldState;
}

/** Public instance API of PhoneField. */
export interface PhoneFieldExpose<Code extends string> extends PhoneFieldSlotState<Code> {
  /** Select a country, keeping the national digits. */
  readonly setCountry: (code: Code) => boolean;

  /** Parse and store any national or international text; returns whether it was understood. */
  readonly setValue: (text: string) => boolean;

  /** Clear the number. */
  readonly clear: () => void;
}
