# BottomNavigation behavior contract

Normative state x input -> outcome table for `bottom-navigation.vue` and
`bottom-navigation-item.vue` (`@vizejs/ui/bottom-navigation`, also exported as
`TabBar`/`TabBarItem`). A bottom tab bar is navigation, not an APG tablist: it
renders a labelled `<nav>` landmark and marks the current destination with
`aria-current="page"`. Every row is proven by the named test in
`bottom-navigation.test.ts` or `bottom-navigation-ssr.test.ts`.

| #   | State                           | Input             | Outcome                                                                                                                                     | Proven by                                                                |
| --- | ------------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| BN1 | seeded                          | render            | `<nav aria-label="Primary">`; items with `href` render links, others `type="button"`; badges render; current item has `aria-current="page"` | `renders a labelled navigation landmark with links, buttons, and badges` |
| BN2 | uncontrolled                    | click             | the item becomes current (`aria-current`, `data-state="active"`) and emits `update:modelValue` then `select`                                | `activating an item moves aria-current and emits typed selections`       |
| BN3 | controlled / disabled / unknown | click             | controlled values win; disabled buttons are native-disabled, disabled links drop `href` with `aria-disabled`; unknown values never emit     | `controlled values win, and disabled or unknown items do not activate`   |
| BN4 | no provider                     | setup             | items throw `VIZE_UI_CONTEXT_MISSING: BottomNavigation`                                                                                     | `items require a BottomNavigation provider`                              |
| BN5 | SSR / hydration                 | isolated requests | byte-identical markup and hydration without diagnostics                                                                                     | `renders byte-identical tab bar markup and hydrates without mismatches`  |

Pair with `SafeArea` (`edges: ["bottom"]`) to clear home indicators. The subpath
ships no CSS.
