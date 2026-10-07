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
The authored `tests/fixtures/sfc/enum-template-bindings/Badge.vue` corpus fixture
covers all four plain-script forms and both setup forms. Compiler integration
tests require literal binding metadata, JavaScript parseability and no enum
proxy reads in DOM, SSR and Vapor. A runtime-dependent setup enum remains inside
setup, and lint regressions retain unrelated undefined-reference diagnostics.

Exact-head Rust and tooling validation runs in Actions before merge admission.
This change does not establish Davinci compiler or linter migration closure.
