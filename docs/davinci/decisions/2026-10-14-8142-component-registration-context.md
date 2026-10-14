# Explicit component registration context (#8142)

Issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142). Carries the linter and CLI part of [#8497](https://github.com/ubugeeei-prod/vize/pull/8497) onto current `main`; #8497 depended on the unmerged editor project-context stack (#8476, #8531).

`vue/require-component-registration` (opinionated presets) silently accepted unknown component tags by guessing: any lowercase name, the `x-`, `ion-`, `md-` and `mwc-` prefixes, and a fixed list of Vue Router and Nuxt names. A missing import of `<RouterView>` or a typo in a kebab-case component therefore went unreported.

The rule now registers a tag only from project facts:

- native HTML, SVG and MathML elements, through the shared tag classifier;
- Vue built-ins, actual imports and setup bindings, and the component's own recursive name;
- globals listed in the rule's options;
- `RouterLink` and `RouterView` when `vue-router` resolves from the linted file (`node_modules/vue-router/package.json` in an ancestor directory, the same check that enables the Pinia rule);
- the compiler's custom-element policy (`compiler.customElements`, same case-sensitive literal and glob semantics as the compiler);
- Nuxt auto-imports when the Nuxt preset is selected.

`vize lint` passes the custom-element matcher from the same loaded configuration document it already reads for the lint plan, so configuration is evaluated once.

An app with Vue Router installed keeps `<RouterLink>` and `<RouterView>` registered without configuration, so n8n's current Vize configuration (which lists no globals) stays clean; its ESLint configuration ignored the same two names. Without an installed `vue-router`, Router tags are reported like any other unregistered component.

## Coverage

`crates/vize_patina/tests/component_registration_project_context.rs` replays the 30 authored SFCs in `tests/_fixtures/differential/lint/component-registration-context-8142/` under both configuration orders, plus static-predicate, Nuxt-mode, disabled-rule and JSX/TSX controls. `crates/vize/tests/lint_component_context_cli.rs` checks the same cases through discovered and explicit CLI routes and that compiler and lint policy share one configuration evaluation.

## Unfinished

The editor does not pass the custom-element matcher yet: the LSP builds its linter from per-document project contexts introduced by #8476. Until that lands, a project that relies on `compiler.customElements` can see this rule report those elements in the editor but not in `vize lint`. The transport and its LSP test (`component_context/lsp.rs` in #8497) follow #8476.
