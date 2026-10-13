# Explicit component registration context (#8142)

Issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142). Carries the linter and CLI part of [#8497](https://github.com/ubugeeei-prod/vize/pull/8497) onto current `main`; #8497 depended on the unmerged editor project-context stack (#8476, #8531).

`vue/require-component-registration` (opinionated presets) silently accepted unknown component tags by guessing: any lowercase name, the `x-`, `ion-`, `md-` and `mwc-` prefixes, and a fixed list of Vue Router and Nuxt names. A missing import of `<RouterView>` or a typo in a kebab-case component therefore went unreported.

The rule now registers a tag only from project facts:

- native HTML, SVG and MathML elements, through the shared tag classifier;
- Vue built-ins, actual imports and setup bindings, and the component's own recursive name;
- globals listed in the rule's options;
- the compiler's custom-element policy (`compiler.customElements`, same case-sensitive literal and glob semantics as the compiler);
- Nuxt auto-imports when the Nuxt preset is selected.

`vize lint` passes the custom-element matcher from the same loaded configuration document it already reads for the lint plan, so configuration is evaluated once.

This is a behavior change for opinionated users: an app that registers Router components globally through `app.use(router)` now has to list `RouterView` and `RouterLink` as globals in the rule options. That matches eslint-plugin-vue's `vue/no-undef-components`, which also has no implicit Router names.

## Coverage

`crates/vize_patina/tests/component_registration_project_context.rs` replays the 30 authored SFCs in `tests/_fixtures/differential/lint/component-registration-context-8142/` under both configuration orders, plus static-predicate, Nuxt-mode, disabled-rule and JSX/TSX controls. `crates/vize/tests/lint_component_context_cli.rs` checks the same cases through discovered and explicit CLI routes and that compiler and lint policy share one configuration evaluation.

## Unfinished

The editor does not pass the custom-element matcher yet: the LSP builds its linter from per-document project contexts introduced by #8476. Until that lands, a project that relies on `compiler.customElements` can see this rule report those elements in the editor but not in `vize lint`. The transport and its LSP test (`component_context/lsp.rs` in #8497) follow #8476.
