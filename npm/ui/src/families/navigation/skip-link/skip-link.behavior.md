# Skip Link behavior contract

Normative state x input -> outcome table for `skip-link.vue`
(`@vizejs/ui/skip-link`). Every row is proven by the named mounted-DOM,
SSR/hydration, renderer, package, or compile-only type test. A row without a
passing test is a contract violation.

| ID  | State          | Trigger                  | Contract                                                                                                   | Evidence                 |
| --- | -------------- | ------------------------ | ---------------------------------------------------------------------------------------------------------- | ------------------------ |
| S1  | default        | render                   | renders a native `<a href="#main">` with deterministic `id`, `part=root`, `data-vize-ui`, and slot content | `skip-link.test.ts`      |
| S2  | hash target    | render                   | accepts only same-document hash destinations; invalid runtime href values remove native navigation         | `skip-link.test.ts`      |
| S3  | focused        | focus / blur             | exposes link focus through `data-state`, slot props, and instance state without adding global listeners    | `skip-link.test.ts`      |
| S4  | valid target   | click / Enter activation | preserves native anchor activation, emits `activate`, and moves DOM focus to the target by default         | `skip-link.test.ts`      |
| S5  | unfocusable    | target focus             | temporarily adds `tabindex="-1"` to an unfocusable target and restores it after blur                       | `skip-link.test.ts`      |
| S6  | focus disabled | click / Enter activation | `focusTarget=false` keeps native navigation and emit behavior while leaving DOM focus unchanged            | `skip-link.test.ts`      |
| S7  | SSR            | server render            | emits deterministic, classless, styleless anchor markup with no document access during render              | `skip-link-ssr.test.ts`  |
| S8  | hydration      | client mount             | hydrates byte-stable server markup without root replacement or diagnostics                                 | `skip-link-ssr.test.ts`  |
| S9  | packaging      | build                    | root and `./skip-link` consumers retain only the skip-link family and emit zero CSS                        | `check-tree-shaking.mjs` |

## Public surface

Props are `id`, `href`, and `focusTarget`. `href` is typed as a hash fragment and defaults to `#main`; `focusTarget` defaults to `true`.

The default slot receives `focused`, `href`, `state`, `targetId`, and `unavailable`. `state` is `idle`, `focused`, or `invalid`.

The `activate` emit receives the native `MouseEvent` and an immutable activation detail containing `href`, `targetId`, `target`, and `focused`.

The component exposes `element`, `focused`, `href`, `state`, `targetId`, `unavailable`, `focus()`, `getTarget()`, and `focusTarget()`.

SkipLink ships no authored CSS selectors, CSS custom properties, or visual preset. Consumers own placement, visibility, and focus styling.
