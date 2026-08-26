# Locale behavior contract

Normative state × input → outcome table for `locale-provider.vue` (`@vizejs/ui/locale`).
Every row is proven by the named test in `src/locale.test.ts` or
`src/locale-ssr.test.ts`; compile-only assertions live in
`src/locale.types.test-d.ts`.

| #   | State         | Input                         | Outcome                                              | Proven by                                 |
| --- | ------------- | ----------------------------- | ---------------------------------------------------- | ----------------------------------------- |
| I1  | provider      | default props                 | `lang=en-US` and `dir=ltr`                           | `provides default locale and direction`   |
| I2  | provider      | `locale` and `direction=rtl`  | subtree lang/dir and slot props update               | `publishes an explicit rtl locale`        |
| I3  | provider      | `direction=auto` with RTL tag | resolved direction is `rtl` when the engine can tell | `resolves auto direction from the locale` |
| I4  | no provider   | `useLocale` / `useDirection`  | document or SSR fallbacks are used                   | `falls back without a provider`           |
| I5  | outside setup | composable call               | stable setup diagnostic is thrown                    | `rejects composable use outside setup`    |
| I6  | SSR           | identical trees               | byte-identical lang and dir                          | SSR test                                  |
| I7  | public types  | invalid direction             | compilation rejects misuse                           | `src/locale.types.test-d.ts`              |
