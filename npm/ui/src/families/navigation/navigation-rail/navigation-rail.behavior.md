# NavigationRail behavior contract

Normative state x input -> outcome table for `navigation-rail.vue`,
`navigation-rail-item.vue`, and `navigation-rail-toggle.vue`
(`@vizejs/ui/navigation-rail`). A navigation rail is the side counterpart of a bottom tab
bar: it is navigation, not an APG tablist, so it renders a labelled `<nav>` landmark and marks
the current destination with `aria-current="page"`. Expanding the rail only publishes state;
labels stay in the DOM in both states so collapsed destinations keep their accessible names.
Every row is proven by the named test in `navigation-rail.test.ts` or
`navigation-rail-ssr.test.ts`.

| #   | State                           | Input              | Outcome                                                                                                                                     | Proven by                                                                |
| --- | ------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| NR1 | seeded                          | render             | `<nav aria-label="Primary">`; items with `href` render links, others `type="button"`; badges render; current item has `aria-current="page"` | `renders a labelled navigation landmark with links, buttons, and badges` |
| NR2 | uncontrolled                    | click              | the item becomes current (`aria-current`, `data-state="active"`) and emits `update:modelValue` then `select`                                | `activating an item moves aria-current and emits typed selections`       |
| NR3 | collapsed / expanded            | toggle click       | `aria-expanded` and `data-expanded` flip on the toggle, the rail, and every item; the toggle's `aria-controls` names the rail               | `the toggle expands and collapses the rail and names what it controls`   |
| NR4 | controlled `expanded` / expose  | toggle, `toggle()` | controlled values win and still emit `update:expanded`; `toggle()` and `setExpanded()` request changes                                      | `controlled expansion wins and the exposed surface requests changes`     |
| NR5 | disabled toggle                 | click              | native `disabled`; the expanded state is unchanged; a deterministic id is generated without an `id` prop                                    | `a disabled toggle keeps the expanded state`                             |
| NR6 | controlled / disabled / unknown | click              | controlled values win; disabled buttons are native-disabled, disabled links drop `href` with `aria-disabled`; unknown values never emit     | `controlled values win, and disabled or unknown items do not activate`   |
| NR7 | no provider                     | setup              | items and the toggle throw `VIZE_UI_CONTEXT_MISSING: NavigationRail`                                                                        | `items and the toggle require a NavigationRail provider`                 |
| NR8 | SSR / hydration                 | isolated requests  | byte-identical markup, toggle wired to the server-rendered rail id, and hydration without diagnostics                                       | `renders byte-identical rail markup and hydrates without mismatches`     |

Pair with `SafeArea` (`edges: ["left"]` or `["right"]`) to clear display cutouts. The subpath
ships no CSS.
