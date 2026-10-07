# Enum template bindings

Issue: [#7893](https://github.com/ubugeeei-prod/vize/issues/7893); the shared
const-enum lint symptom is also tracked by
[#7896](https://github.com/ubugeeei-prod/vize/issues/7896).

Non-ambient regular and const TypeScript enums declare runtime template names
in both script blocks. The SFC transform preserves their values under isolated
TypeScript transformation. An ambient `declare enum` remains type-only.

The compiler registers plain-script enums, including named exports, with the
same existing static initializer classifier used for setup enums. Static enum
bindings are `LiteralConst`; runtime-dependent enum initializers remain
`SetupConst`. Setup template metadata applies that classifier to the retained
script AST, without another parse or a new pipeline stage.

Croquis includes const enums in its runtime name and span facts. The independent
Bindings spec accepts the same declarations; no lint-rule exception is needed.
The original reporter input is retained without source changes in
`tests/_fixtures/differential/compiler/enum-template-bindings-7893/Badge.vue.txt`.
The separate authored `tests/fixtures/sfc/enum-template-bindings/Badge.vue`
fixture covers all four plain-script forms and both setup forms. Compiler integration
tests require literal binding metadata, JavaScript parseability and no enum
proxy reads in DOM, SSR and Vapor. A runtime-dependent setup enum remains inside
setup, and lint regressions retain unrelated undefined-reference diagnostics.

Hosted source execution on `5282cb5a0e` proved DOM and SSR, then exposed Vapor's
remaining unconditional component-proxy reads for literal bindings. Vapor now
resolves `LiteralConst` through its existing scope resolver after lexical loop
and slot bindings, preserving those shadowing rules. The regression parses the
whole emitted module and compares the complete component-proxy member-read
vector with an empty vector; this replaces fragment checks without an allowlist.

A scope review found that literal metadata is broader than actual SFC hoisting:
template literals, negative literals and multiple-declarator consts can stay
inside setup. The Vapor generation-only metadata normalizes those names to
setup constants using the retained program. Its declaration predicate is shared
with the existing hoister. Public metadata stays unchanged, and enums and
eligible module constants remain lexical. Complete-module controls require the
unhoisted literals and plain Options API setup locals to remain proxy reads.

Hosted run `37591703876` proved the original enum matrix and lint controls, and
reported exactly four snapshot changes: hoisted `message`, `title` and `visible`
now read directly. Complete decoded Rspack bundles differ only at those reads;
reactive `count`, CSS, HMR and component resolution stay the same. The narrow
snapshot updates retain that observed behavior; the scope guards require fresh
exact-head Actions before admission.
The existing conversion call owns enum enrichment so the oversized entry file
does not grow. Regenerated source inventories retain the actual added uses.

Exact-head Rust and tooling validation runs in Actions before merge admission.
This change does not establish Davinci compiler or linter migration closure.
