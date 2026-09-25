/** Phone number field with consumer-supplied country metadata, pattern formatting, and E.164 output. */
export { default as PhoneField } from "./phone-field.vue";
/** Masked `type="tel"` input for the national number. */
export { default as PhoneFieldInput } from "./phone-field-input.vue";
/** Country picker built on NativeSelect. */
export { default as PhoneFieldCountrySelect } from "./phone-field-country-select.vue";
export {
  definePhoneCountries,
  formatNationalNumber,
  parsePhoneNumber,
  phoneDigitLimit,
  phoneDigits,
  stripTrunkPrefix,
  toE164,
} from "./phone-field-country.ts";
export type { ParsedPhoneNumber, PhoneCountry } from "./phone-field-country.ts";
export type {
  PhoneFieldAriaInvalid,
  PhoneFieldExpose,
  PhoneFieldSlotState,
  PhoneFieldState,
} from "./phone-field-types.ts";
