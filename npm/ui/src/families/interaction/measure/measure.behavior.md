# Measurement observers behavior contract

Normative state × input → outcome table for `@vizejs/ui/measure`. Every row is
exercised by `src/families/interaction/measure/measure.test.ts` or
`src/families/interaction/measure/measure-ssr.test.ts`; compile-only assertions
live in `src/families/interaction/measure/measure.types.test-d.ts`.

| #   | State          | Input                                 | Outcome                                             | Proven by                                                  |
| --- | -------------- | ------------------------------------- | --------------------------------------------------- | ---------------------------------------------------------- |
| M1  | observing      | platform reports size changes         | one batched `onResize` with only observed targets   | `reports batched size changes for observed elements`       |
| M2  | observing      | platform entry carries box sizes      | configured box wins, content rect is the fallback   | `prefers the configured box size over the content rect`    |
| M3  | idle           | `observe` for one element twice       | one platform observation and `observedCount` of one | `keeps observation idempotent per element`                 |
| M4  | observing      | `unobserve` or `disconnect`           | later platform entries for that target are dropped  | `stops reporting after unobserve and disconnect`           |
| M5  | no platform    | `observe` without observer support    | no-op with `isSupported` false, nothing throws      | `no-ops without platform observer support`                 |
| M6  | observing      | platform reports intersection changes | one batched `onVisibilityChange` with observed only | `reports batched visibility changes for observed elements` |
| M7  | any            | invalid callbacks, box, or target     | `VIZE_UI_MEASURE_OPTION` TypeError                  | `validates options and observation targets`                |
| M8  | disposed       | any observation call                  | `VIZE_UI_MEASURE_DISPOSED` Error                    | `throws for observation after dispose`                     |
| M9  | any            | `useSizeObserver` outside a scope     | `VIZE_UI_MEASURE_SETUP` Error                       | `binds disposal to the owning effect scope`                |
| M10 | scoped         | owning effect scope stops             | controller is disposed automatically                | `binds disposal to the owning effect scope`                |
| M11 | concurrent SSR | identical trees                       | byte-identical markup without observer construction | `renders byte-identical SSR output without observers`      |
| M12 | hydration      | server markup mounts                  | no replacement and no hydration diagnostics         | `hydrates measurement consumers without diagnostics`       |
| M13 | public types   | invalid box or mutating readonly refs | compilation rejects misuse                          | `src/families/interaction/measure/measure.types.test-d.ts` |
