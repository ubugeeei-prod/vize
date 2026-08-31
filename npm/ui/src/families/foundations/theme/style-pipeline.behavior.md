# Style pipeline behavior contract

Normative contract for the `@vizejs/ui` packaged stylesheet (`dist/style.css`,
exported as `@vizejs/ui/style.css`) and the CSS-only theme entrypoints
(`@vizejs/ui/theme.css` and `@vizejs/ui/theme-preset-*.css`). Component styles
are authored in scoped SFC style blocks — `visually-hidden.vue` is the
canonical example — using native CSS only, and the package build lowers them
with the Lightning CSS transform in `vite-plus` pack (`pack.css` in
`vite.config.ts`). Every row is proven by `src/families/foundations/theme/style-pipeline.test.ts` against
the real package build.

| #   | Authored feature                      | At the declared floor | Shipped output                                     | Proven by                                                                                |
| --- | ------------------------------------- | --------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| S1  | CSS Nesting (`&` rules)               | not yet native        | flattened standalone selectors                     | `authored nesting, layers, logical properties, and color functions compile to the floor` |
| S2  | cascade layers (`@layer vize.ui`)     | native                | preserved verbatim                                 | `authored nesting, layers, logical properties, and color functions compile to the floor` |
| S3  | logical properties (`inline-size`, …) | native                | preserved verbatim                                 | `authored nesting, layers, logical properties, and color functions compile to the floor` |
| S4  | native color functions (`oklch()`)    | native                | preserved without legacy fallback                  | `authored nesting, layers, logical properties, and color functions compile to the floor` |
| S5  | scoped `<style scoped>` semantics     | n/a                   | every lowered rule keeps its `[data-v-*]` selector | `scoped style semantics survive the down-compile`                                        |
| S6  | runtime CSS-in-JS                     | forbidden             | styles exist only as opt-in CSS asset files        | `styles never arrive through runtime CSS-in-JS`                                          |

## Target policy

- The browser floor is declared exactly once, as `cssBrowserFloor` in
  `vite.config.ts`, and the pack CSS transform lowers every authored style
  block to it. Nothing else in the package may assume a different floor.
- The current floor (`chrome111`, `edge111`, `firefox113`, `safari16.4`) is
  the earliest evergreen release line where cascade layers, `:where()`,
  logical properties, and `oklch()` are all native. CSS Nesting is newer than
  the floor, so authored nesting always ships flattened.
- Moving the floor is a reviewed contract change: edit `cssBrowserFloor`,
  rebuild, and update the `style-pipeline.test.ts` rows that pin which
  features lower and which ship verbatim.

## How consumers override the floor

- `dist/style.css`, `dist/theme.css`, and `dist/theme-preset-*.css` are
  standard, already-lowered CSS with no preprocessor or toolchain requirement.
  A consumer that needs an older floor runs its own bundler's CSS target over
  the file exactly like any vendored stylesheet.
- Styles remain opt-in files. Component entries reach the stylesheet through
  a static `import "./style.css"` declared under `sideEffects`
  (`./dist/*.css`), so a consumer that never imports a styled component ships
  no CSS, and a consumer that overrides `sideEffects` handling can drop the
  stylesheet entirely and load `@vizejs/ui/style.css`,
  `@vizejs/ui/theme.css`, or one of the preset CSS entrypoints on its own
  terms.
- Every shipped rule lives inside a `vize.*` cascade layer — `vize.tokens`,
  `vize.ui`, `vize.preset`, and `vize.policy`, in that ascending order; see
  `theme.behavior.md` for the order and specificity contract — so consumer
  CSS outside a layer, or in a later layer, always wins without specificity
  fights.
- Preset styles are authored as small native CSS files imported by
  `src/families/foundations/theme/theme.ts`. They share the same `vize.preset` layer and are lowered by
  the package build before reaching `dist/style.css`; the same source files
  also publish CSS-only preset entrypoints with the cascade-layer prelude
  prepended so import order cannot redefine the layer priority.
