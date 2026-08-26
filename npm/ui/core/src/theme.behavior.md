# Theme behavior contract

Normative contract for the theme family (`@vizejs/ui/theme`): the semantic
design-token contract, the package-wide cascade-layer order, and the opt-in
presets. The stylesheet contract is proven on the packaged `dist/style.css`
(the pack pipeline lowers `src/theme.css` and the preset files
`src/theme-preset-atelier.css`, `src/theme-preset-midnight.css`,
`src/theme-preset-paper.css`, and `src/theme-preset-signal.css` to the
declared browser floor; see `style-pipeline.behavior.md`) in
`src/theme-stylesheet.test.ts`; runtime rows are proven by the named test in
`src/theme.test.ts` or `src/theme-ssr.test.ts`; compile-only assertions live
in `src/theme.types.test-d.ts`.

| #   | State               | Input                                                             | Outcome                                                                                                   | Proven by                                                          |
| --- | ------------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| T1  | packaged stylesheet | consumer imports any styled entry                                 | cascade layers ship in ascending priority order `vize.tokens` → `vize.ui` → `vize.preset` → `vize.policy` | `ships the documented cascade layer order`                         |
| T2  | packaged stylesheet | any styled entry                                                  | the semantic token contract ships in `@layer vize.tokens` at zero specificity, matching the TS mirrors    | `ships layered zero-specificity theme tokens matching the mirrors` |
| T3  | packaged stylesheet | no `data-vize-theme` attribute                                    | headless default: colors stay on the system palette and every elevation is `none`                         | `keeps the headless default free of visual opinion`                |
| T4  | packaged stylesheet | `data-vize-density="compact" \| "comfortable"`                    | the density factor retunes, scaling every space and control-size token in the subtree                     | `ships density scopes that retune the shared factor`               |
| T5  | packaged stylesheet | `data-vize-theme~="atelier" \| "midnight" \| "paper" \| "signal"` | published presets assign visual tokens in `@layer vize.preset`, scoped to their opt-in attribute          | `scopes published presets to their opt-in attributes`              |
| T6  | packaged stylesheet | light and dark schemes                                            | preset `light-dark()` values ship lowered to the floor and follow the user's color scheme                 | `lowers preset color schemes to the declared floor`                |
| T7  | packaged stylesheet | `forced-colors: active`                                           | `@layer vize.policy` snaps color roles to the system palette and flattens elevation, beating presets      | `stands down to system colors under forced colors`                 |
| T8  | any                 | `setThemeTokens(element, overrides)`                              | overrides apply to the element inline and the restore callback reinstates values                          | `applies and restores token overrides on a real element`           |
| T9  | any                 | unknown token name or empty value                                 | `themeTokenProperty`/`setThemeTokens` throw `VIZE_UI_THEME_TOKEN`                                         | `rejects unknown tokens and empty override values`                 |
| T10 | mounted             | consumer binds scope attributes and tokens                        | preset and density attributes render, and scoped overrides apply through the mounted DOM                  | `scopes presets and densities in a mounted consumer`               |
| T11 | SSR                 | render with token helpers in setup                                | byte-identical markup; no platform global is required                                                     | `renders byte-identical SSR markup without platform globals`       |
| T12 | SSR                 | hydration                                                         | hydrating a themed consumer emits no diagnostics and keeps server nodes                                   | `hydrates a themed consumer without replacement or diagnostics`    |
| T13 | public types        | invalid token names or mutated records                            | compilation rejects misuse                                                                                | `src/theme.types.test-d.ts`                                        |

## Cascade-layer order and specificity budget

- The package ships exactly four cascade layers, in ascending priority:
  `vize.tokens` (semantic defaults) < `vize.ui` (behavior-critical component
  styles) < `vize.preset` (opt-in visual presets) < `vize.policy`
  (accessibility stand-downs such as forced colors). `src/theme.css` declares
  the order once; packaged CSS concatenates in `src/index.ts` module order, so
  the theme export stays directly after `field-wiring` and the statement leads
  `dist/style.css`.
- Specificity budget: every rule in every vize layer sits on a `:where()`
  selector at specificity `(0,0,0)` — pseudo-elements keep only their
  intrinsic element specificity. Consumer CSS outside the vize layers always
  wins, and consumer `@layer` rules win by declaring their layers later.

## Extension contract

- Every token is a `--vize-ui-*` custom property defined at zero specificity:
  a consumer rule outside the layers — or a `setThemeTokens` override on an
  ancestor — replaces any role for a whole subtree without forking
  components. `--vize-ui-focus-ring-color` resolves through
  `--vize-ui-color-accent`, and space/size tokens resolve through
  `--vize-ui-density`, so one override retunes a whole phase.
- `data-vize-theme` holds space-separated preset names and is inert without
  the shipped preset CSS; `atelier` is the Vize brand default, `midnight` is
  the dark-first application preset, `paper` is the editorial preset for
  content-forward surfaces, and `signal` is the dense data/tooling preset.
  When multiple presets are present, package source order resolves ties, so
  later shipped presets win. `data-vize-density` accepts `compact` and
  `comfortable`. Both scope by inheritance, so nesting attributes nests
  themes.
- SSR bootstrapping is CSS-only and flash-free by construction: server-render
  the attribute (typically on `<html>`), and preset `light-dark()` values
  follow the user's scheme with no runtime script. Motion tokens stay in the
  motion family (`motion.behavior.md`).
