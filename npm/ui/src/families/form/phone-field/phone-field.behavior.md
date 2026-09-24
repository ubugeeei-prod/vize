# PhoneField behavior contract

Normative state x input -> outcome table for `phone-field.vue`,
`phone-field-input.vue`, and `phone-field-country-select.vue`
(`@vizejs/ui/phone-field`). No country data is bundled: consumers pass
`countries` (typed with `definePhoneCountries`), whose `code` literals type
`v-model:country`. Formatting reuses the `input-mask` engine; the country picker
reuses `NativeSelect`. Every row is proven by the named test in
`phone-field.test.ts` or `phone-field-ssr.test.ts`; compile-only assertions live
in `phone-field.types.test-d.ts`.

| #   | State                | Input                        | Outcome                                                                                                                                   | Proven by                                                                              |
| --- | -------------------- | ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| PH1 | helpers              | parse / format / E.164       | national text drops the trunk prefix; `+`/`00` text picks the longest dial code (ties prefer the current country); full-width digits fold | `parses national and international text with trunk prefixes and dial-code matching`    |
| PH2 | seeded, named        | render                       | `type="tel"` input (`inputmode="tel"`, `tel-national`), NativeSelect country picker with `aria-controls`, hidden E.164 value              | `renders a tel input, a NativeSelect country picker, and a hidden E.164 value`         |
| PH3 | empty                | typing                       | digits format by the country pattern, the trunk prefix is stripped, `complete(e164, country)` fires when the pattern fills                | `typing formats by the country pattern, strips the trunk prefix, and emits completion` |
| PH4 | number entered       | country select               | the national digits are kept and re-targeted to the new dial code; `update:country` fires                                                 | `choosing a country keeps the digits and re-targets the dial code`                     |
| PH5 | any                  | international typing / paste | `+44…` or pasted `+1…` switches the country and fills the national number                                                                 | `international input and paste switch the country automatically`                       |
| PH6 | controlled / API     | model / expose               | a stored number's country wins over the selected country; `setValue`, `setCountry`, `clear`                                               | `controlled values from another country win, and the API sets, switches, and clears`   |
| PH7 | in a form / disabled | reset / edits                | form reset restores defaults; disabled fields disable input and select and ignore edits                                                   | `form reset restores defaults and disabled fields ignore input`                        |
| PH8 | invalid setup        | empty countries / no root    | throws `VIZE_UI_PHONE_FIELD_COUNTRIES` or `VIZE_UI_CONTEXT_MISSING: PhoneField`                                                           | `rejects empty country lists and parts outside a PhoneField`                           |
| PH9 | SSR / hydration      | isolated requests            | byte-identical markup with the formatted number and selected country; hydration without diagnostics                                       | `renders byte-identical phone markup and hydrates without mismatches`                  |

## Public extension contract

| Surface         | Contract                                                                                                                   |
| --------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Parts           | `root` (group), `input`, `country-select`.                                                                                 |
| Data attributes | `data-vize-ui`, root `data-state` (`empty`/`incomplete`/`complete`/`disabled`) and `data-country`; input `data-dial-code`. |
| Metadata        | `PhoneCountry { code, name, dialCode, pattern?, trunkPrefix? }`, supplied by the consumer.                                 |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
