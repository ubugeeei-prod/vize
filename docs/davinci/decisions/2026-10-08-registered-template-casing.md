# Registered component template casing

Decision paired with [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

`vue/component-name-in-template-casing` checks registered components by default.
An imported `MyWidget` rendered as `<my-widget />` produces a casing diagnostic;
an unregistered dashed tag produces no casing diagnostic. Registration checking
remains independent and can still report the missing component.

The independent oracle uses ESLint 10.4.1, eslint-plugin-vue 10.9.2,
vue-eslint-parser 10.4.1 and @typescript-eslint/parser 8.65.0 on Node 24.14.0.
The original casing configuration is `['error', 'PascalCase']`, with the
registered-only default unchanged. Four paired cases also enable
`vue/no-undef-components`. Separate casing-only and explicitly configured option
cohorts retain their own configurations. All sources are authored for this fix;
no non-master n8n source was executed or redistributed.

At original source `b8ea16711117c502b14724e1eb1e4a7022a7f1d7`, an actual
`Linter::lint_sfc` observer produced an extra casing diagnostic and fix for both
scripted and scriptless unregistered tags. The committed
[before packet](../../../crates/vize_patina/tests/fixtures/issue-8142-template-casing/source-before.json)
preserves complete source API findings, severities, byte ranges, help, labels,
fixes, filenames and counts for 32 cases. It is a temporary source API test
capture, not a packaged CLI, native host, or installed-public receipt.

The [corpus provenance](../../../crates/vize_patina/tests/fixtures/issue-8142-template-casing/README.md)
records all five complete independent packets and their provider hashes.
The regression asserts complete source envelopes and final fix bytes for those
32 cases plus 18 explicit policy and definition boundary controls. The independent registration
membership and final-byte law covers every case; provider-specific diagnostic
wording and ranges retain their separate complete envelopes.

Collect a separate authored registration inventory in the existing script AST
walk. Module declaration names count only when `<script setup>` exists; type-only
imports do not count, while local type/interface declarations do. Direct static
`components` keys count independently of their values. Exported options aliases,
call-argument aliases and TypeScript-wrapped definitions do not count. Component identity
resolution remains unchanged: casing does not follow object aliases or spreads.
Both script blocks contribute when setup exists. No second script parse, extra
pipeline stage, hoisting demand, or template-expression collection is added.
Compiler demand skips the inventory allocations. Casing-only SFC lint now
requests existing shared semantic analysis; timing claims require measurements.

Expose `registeredComponentsOnly` (default `true`) and literal `globals` (default
empty) in typed CLI/LSP configuration. Existing casing-only SDK options inherit
the corrected default. Preserve the older standalone-template edit corpus by
explicitly opting into `registeredComponentsOnly: false`; its diagnostics and
fix bytes remain unchanged. The older scriptless Opinionated/Vuetify test also
declares that same all-tag policy; it retains its original source and finding.
Generated rule examples and explain pages now supply the authored setup import.
The public-fact census resolves the actual linter's typed inventory access;
regenerate and byte-check both consumption and migration shards without exemptions.
HTML/SVG/MathML and `slot`/dynamic `component` syntax
are not component casing targets. Nuxt framework exemptions remain intentional.

The new public `Croquis.template_component_registrations` field and removal of
`Copy` from the expanded public config options require the unchanged Rust SemVer
release checks and an appropriate compatibility boundary. Do not replace the
public field with a private field: `Croquis` remains externally constructible.

Unfinished: regex globals/ESLint `ignores`, their explicit native SDK/oxlint option
transport, complete native-host coverage, actual upstream package-build
acceptance, performance qualification, and final protected delivery/publication.
The bounded authored corpus establishes this source rule correction only.
