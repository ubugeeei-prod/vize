# SafeArea behavior contract

Normative state x input -> outcome table for `safe-area.vue` and
`useSafeAreaInsets` (`@vizejs/ui/safe-area`). Every row is proven by the named
test in `safe-area.test.ts` or `safe-area-ssr.test.ts`; compile-only assertions
live in `safe-area.types.test-d.ts`.

| #   | State           | Input               | Outcome                                                                                                                          | Proven by                                                                                |
| --- | --------------- | ------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| SA1 | any             | render              | `--vize-safe-area-inset-*` map to `env(safe-area-inset-*, 0px)`; `data-edges` lists edges; `apply` pads/margins only those edges | `exposes env() insets as CSS variables and applies only the selected edges`              |
| SA2 | mounted         | microtask / resize  | a hidden probe measures insets after hydration; non-zero edges appear in `data-insets` and the slot; unmount removes the probe   | `measures insets after a microtask, publishes non-zero edges, and re-measures on resize` |
| SA3 | composable      | scope / no document | `useSafeAreaInsets` runs in any effect scope, stops with it, and stays at zero without a document                                | `useSafeAreaInsets works in any scope and is inert without a document`                   |
| SA4 | slot            | render              | the default slot renders contents with typed insets                                                                              | `renders slot content with typed insets`                                                 |
| SA5 | SSR / hydration | isolated requests   | byte-identical markup with CSS variables and no measured data; hydration without diagnostics                                     | `renders byte-identical safe-area markup and hydrates without mismatches`                |

Non-zero insets require `<meta name="viewport" content="viewport-fit=cover">`.
The subpath is tree-shakable and ships no CSS.
