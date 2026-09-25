# MediaPreferences behavior contract

Normative behavior for `@vizejs/ui/media-preferences`. `media-preferences-provider.vue`
(`MediaPreferencesProvider`) and the composables expose `prefers-reduced-motion`,
`prefers-reduced-transparency`, `forced-colors`, `prefers-contrast`, and `prefers-color-scheme`
without touching `window` before mount, so server and hydration renders agree. Every row is
proven by the named test.

| State x input                            | Observable outcome                                                                                                                                                                         | Proven by                                                                        |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| mounted provider, matching media queries | Root publishes `data-reduced-motion`, `data-reduced-transparency`, `data-forced-colors` (present when true), `data-prefers-contrast`, `data-color-scheme`; the slot receives the snapshot. | `provider publishes detected preferences as data attributes after mount`         |
| media query `change` events              | Every preference is re-read and attributes update; unmount removes every listener.                                                                                                         | `provider follows media query change events and cleans up listeners`             |
| `force`                                  | Forced values always win over detection; clearing `force` restores detected values.                                                                                                        | `force overrides detection and initial is only a pre-detection hint`             |
| `initial`                                | Hints apply before detection only; detected values replace them after mount.                                                                                                               | `force overrides detection and initial is only a pre-detection hint`             |
| composables inside a provider            | `usePrefersReducedMotion`, `usePrefersReducedTransparency`, `useForcedColors`, `usePrefersContrast`, `usePrefersColorScheme` read the provider's effective values.                         | `composables read the provider context`                                          |
| standalone composables                   | In components they return defaults during setup and subscribe after mount; in bare effect scopes they subscribe immediately; outside a scope they throw `VIZE_UI_MEDIA_PREFERENCES_SETUP`. | `standalone composables subscribe after mount and in bare effect scopes`         |
| tracker helpers                          | `readMediaPreferences`, `mergeMediaPreferences`, and the tracker's `start`/`stop`/`dispose` lifecycle behave as documented.                                                                | `tracker helpers read, merge, stop, and dispose`                                 |
| provider expose                          | Exposes each effective preference and the root element.                                                                                                                                    | `provider exposes effective preferences`                                         |
| SSR                                      | Server markup is byte-identical and built from defaults plus `initial` hints.                                                                                                              | `renders byte-identical markup from defaults and initial hints`                  |
| hydration                                | Hydration matches the server snapshot with zero warnings, then detected preferences apply.                                                                                                 | `hydrates with the server snapshot, then applies detected preferences`           |
| public types                             | Contrast, color scheme, overrides, and composable return types are closed contracts.                                                                                                       | `src/families/accessibility/media-preferences/media-preferences.types.test-d.ts` |
| DOM/SSR/Vapor                            | `media-preferences-provider.vue` compiles in every renderer lane.                                                                                                                          | `scripts/check-renderers.ts`                                                     |

| Target | Public contract                                                                                                                                                                    |
| ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Root   | `part="root"`, `data-vize-ui="media-preferences-provider"`, `data-reduced-motion`, `data-reduced-transparency`, `data-forced-colors`, `data-prefers-contrast`, `data-color-scheme` |

MediaPreferences ships no stylesheet.
