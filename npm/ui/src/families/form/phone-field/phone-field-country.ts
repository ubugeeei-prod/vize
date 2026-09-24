import { createInputMask } from "../input-mask/input-mask.ts";

/**
 * Consumer-supplied metadata for one country or region. No data is bundled:
 * pass only the countries your product supports (for example generated from
 * libphonenumber metadata at build time).
 */
export interface PhoneCountry<Code extends string = string> {
  /** Stable code, typically ISO 3166-1 alpha-2 (`"JP"`). */
  readonly code: Code;

  /** Human-readable name shown by the country select. */
  readonly name: string;

  /** Country calling code without `+` (`"81"`). */
  readonly dialCode: string;

  /**
   * National-number format using `9` for a digit, for example `"99-9999-9999"`.
   * Without a pattern, digits are shown ungrouped (up to 15).
   */
  readonly pattern?: string;

  /** National trunk prefix stripped before building E.164 (`"0"` in Japan). */
  readonly trunkPrefix?: string;
}

/** Result of {@link parsePhoneNumber}. */
export interface ParsedPhoneNumber<Code extends string> {
  /** Resolved country. */
  readonly country: PhoneCountry<Code>;

  /** National significant number: digits only, trunk prefix removed. */
  readonly nationalNumber: string;
}

const maximumDigits = 15;
const fallbackPattern = "9".repeat(maximumDigits);

/**
 * Keep literal country codes while declaring metadata.
 *
 * @example
 * const countries = definePhoneCountries([
 *   { code: "JP", name: "Japan", dialCode: "81", pattern: "99-9999-9999", trunkPrefix: "0" },
 *   { code: "US", name: "United States", dialCode: "1", pattern: "(999) 999-9999" },
 * ]);
 */
export function definePhoneCountries<const Countries extends readonly PhoneCountry[]>(
  countries: Countries,
): Countries {
  for (const country of countries) {
    if (!/^\d{1,4}$/.test(country.dialCode)) {
      throw new TypeError(
        `VIZE_UI_PHONE_FIELD_DIAL_CODE: ${country.code} dial code must be 1-4 digits without "+"`,
      );
    }
  }
  return countries;
}

/** Digits of any text, folding full-width digits. */
export function phoneDigits(text: string): string {
  return text.normalize("NFKC").replace(/\D+/g, "");
}

/** Remove the country's trunk prefix from national digits. */
export function stripTrunkPrefix(digits: string, country: PhoneCountry): string {
  const prefix = country.trunkPrefix;
  return prefix !== undefined && prefix.length > 0 && digits.startsWith(prefix)
    ? digits.slice(prefix.length)
    : digits;
}

/** Maximum national digits accepted for a country (its pattern slots, else 15). */
export function phoneDigitLimit(country: PhoneCountry): number {
  return createInputMask(country.pattern ?? fallbackPattern).slotCount;
}

/** Format national digits with the country's pattern (partial input keeps its literals). */
export function formatNationalNumber(nationalNumber: string, country: PhoneCountry): string {
  return createInputMask(country.pattern ?? fallbackPattern).fromRaw(nationalNumber).masked;
}

/** Build an E.164 string (`+819012345678`), or `""` without national digits. */
export function toE164(nationalNumber: string, country: PhoneCountry): string {
  return nationalNumber.length === 0 ? "" : `+${country.dialCode}${nationalNumber}`;
}

/**
 * Parse typed, pasted, or stored text. International text (`+`, `00`, or an
 * E.164 value) selects the country with the longest matching dial code (ties
 * prefer `preferred`); national text keeps `preferred` and drops its trunk prefix.
 *
 * @returns The country and national number, or `undefined` when no country
 *   matches an international number.
 */
export function parsePhoneNumber<Code extends string>(
  text: string,
  countries: readonly PhoneCountry<Code>[],
  preferred: PhoneCountry<Code>,
): ParsedPhoneNumber<Code> | undefined {
  const trimmed = text.normalize("NFKC").trim();
  const international = trimmed.startsWith("+") || trimmed.startsWith("00");
  const digits = phoneDigits(trimmed.startsWith("00") ? trimmed.slice(2) : trimmed);
  if (!international) {
    const nationalNumber = stripTrunkPrefix(digits, preferred).slice(0, phoneDigitLimit(preferred));
    return { country: preferred, nationalNumber };
  }
  let match: PhoneCountry<Code> | undefined;
  for (const country of countries) {
    if (!digits.startsWith(country.dialCode)) continue;
    const longer = match === undefined || country.dialCode.length > match.dialCode.length;
    const preferredTie =
      match !== undefined &&
      country.dialCode.length === match.dialCode.length &&
      country === preferred;
    if (longer || preferredTie) match = country;
  }
  if (match === undefined) return undefined;
  const national = stripTrunkPrefix(digits.slice(match.dialCode.length), match);
  return { country: match, nationalNumber: national.slice(0, phoneDigitLimit(match)) };
}
