# Locale behavior contract

Normative state × input → outcome table for `locale-provider.vue` (`@vizejs/ui/locale`).
Every row is proven by the named test in `src/locale.test.ts` or
`src/locale-ssr.test.ts`; compile-only assertions live in
`src/locale.types.test-d.ts`.

| #   | State         | Input                                      | Outcome                                                    | Proven by                                                                  |
| --- | ------------- | ------------------------------------------ | ---------------------------------------------------------- | -------------------------------------------------------------------------- |
| I1  | provider      | default props                              | `lang=en-US` and `dir=ltr`                                 | `provides default locale and direction`                                    |
| I2  | provider      | `locale` and `direction=rtl`               | subtree lang/dir and slot props update                     | `publishes an explicit rtl locale`                                         |
| I3  | provider      | `direction=auto` with RTL tag              | resolved direction is `rtl` when the engine can tell       | `resolves auto direction from the locale`                                  |
| I4  | locale helper | invalid or lower-case locale tags          | canonical locale or `en-US` fallback is used               | `canonicalizes invalid locale tags before formatter construction`          |
| I5  | provider      | formatter composables with options         | number, date, list, and relative-time use provider locale  | `resolves formatters from the provider locale and explicit options`        |
| I6  | provider      | display-name and search-collator helpers   | localized code names and accent-insensitive search compare | `resolves display names and search collators from the provider locale`     |
| I7  | locale helper | decomposed Unicode and whitespace          | text is NFC-normalized before exact, prefix, or contains   | `matches normalized locale text with exact, prefix, and contains policies` |
| I8  | provider      | locale or formatter options change         | formatter composables update without stale Intl objects    | `updates formatter composables when provider locale or options change`     |
| I9  | provider      | invalid locale tag                         | context, markup, direction, and formatters fall back       | `normalizes invalid provider locales before publishing context`            |
| I10 | no provider   | `useLocale` / `useDirection`               | document or SSR fallbacks are used                         | `falls back without a provider`                                            |
| I11 | outside setup | composable call                            | stable setup diagnostic is thrown                          | `rejects composable use outside setup`                                     |
| I12 | SSR           | identical trees                            | byte-identical lang and dir                                | SSR test                                                                   |
| I13 | SSR           | formatter without provider                 | `en-US` fallback locale is used                            | SSR formatter test                                                         |
| I14 | SSR           | display names and search collator fallback | `en-US` formatter locale and search usage are used         | SSR display/search test                                                    |
| I15 | SSR           | invalid provider locale                    | fallback lang and direction are rendered                   | SSR invalid-locale test                                                    |
| I16 | public types  | invalid direction or formatter/search opts | compilation rejects misuse                                 | `src/locale.types.test-d.ts`                                               |
