/** Compile-only assertions for the public PhoneField contract. */

import {
  PhoneField,
  PhoneFieldCountrySelect,
  PhoneFieldInput,
  definePhoneCountries,
  type PhoneCountry,
  type PhoneFieldExpose,
  type PhoneFieldSlotState,
  type PhoneFieldState,
} from "./phone-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type RootProps<Code extends string> = Parameters<typeof PhoneField<Code>>[0];

/** Infers country codes exactly as a template usage would. */
declare function inferCode<Code extends string>(props: RootProps<Code>): Code;

const countries = definePhoneCountries([
  { code: "JP", name: "Japan", dialCode: "81", pattern: "99-9999-9999" },
  { code: "US", name: "United States", dialCode: "1" },
]);
const inferred = inferCode({ countries });
declare const control: PhoneFieldExpose<"JP" | "US">;
declare const slot: PhoneFieldSlotState<"JP" | "US">;

type _InfersCodes = Expect<Equal<typeof inferred, "JP" | "US">>;
type _SlotCountry = Expect<Equal<typeof slot.country, PhoneCountry<"JP" | "US">>>;
type _StateIsClosed = Expect<
  Equal<PhoneFieldState, "complete" | "disabled" | "empty" | "incomplete">
>;
type _DefinePreservesLiterals = Expect<Equal<(typeof countries)[number]["code"], "JP" | "US">>;

const props: RootProps<"JP" | "US"> = {
  countries,
  defaultCountry: "US",
  "onUpdate:country": (code: "JP" | "US") => code,
  onComplete: (e164: string, country: PhoneCountry<"JP" | "US">) => {
    void e164;
    void country;
  },
};
const inputProps: InstanceType<typeof PhoneFieldInput>["$props"] = { autocomplete: "tel" };
const selectProps: InstanceType<typeof PhoneFieldCountrySelect>["$props"] = {
  getOptionLabel: (country) => country.name,
};

control.setCountry("JP");

// @ts-expect-error only supplied country codes are accepted.
control.setCountry("FR");

// @ts-expect-error the default country must be supplied.
const wrongDefault: RootProps<"JP" | "US"> = { countries, defaultCountry: "FR" };

void inputProps;
void props;
void selectProps;
void wrongDefault;
