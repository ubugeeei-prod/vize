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

## Genuine current-main refresh

The existing PR's actual `a851939c52c6f50eb9b6419bc3bf917b49959c48`
head is refreshed by merging genuine signed main
`8c7727613de6213e0a52cde96eeb295d01c9bef5`. All nineteen original
non-document paths remain byte-identical, including every compiler/parser/lint
law, snapshot, the 269-byte original reporter fixture (SHA256
`02a9907355de1b26720b9a7cd847f81c42626a3600b2663c334fa64fb89db113`)
and the 693-byte authored six-form fixture (SHA256
`5fb181261ebe4896b50495ee3f8f12c5d96ea0ef595a23a8523f3b5acdac916c`).
The merged actual-source generator resolves the Croquis consumption conflict:
`BindingMetadata` has thirteen files/thirty-four sites and `BindingType` has
nineteen files/142 sites. Consumer inventory checks and locked, offline Cargo
metadata pass; all four affected manifests and their dependency/layout surface
match actual main. The common 350-line decision record is coordinated separately.

The prior `a851` source/native/fact successes remain historical. The refreshed
head needs fresh exact-head Actions, protected qualification and actual merge;
the refresh itself grants no execution, runtime, publication or issue-closure
credit. The six declaration forms from #7893's reported matrix are covered,
while direct execution of the reported three CLI build modes and a separate
Vue runtime replay of the original component have not been established by the
compiler module-inspection tests. Those are explicit validation boundaries for
the root's closure decision, not additional claimed compiler functionality.

The const-enum half of #7896 requires this enum source to actually deliver.
Its plugin-global half separately requires actual delivery of #8146; that PR
is still open at `b39adcc2a082b83206e0cfe60b91dda66bf87043` at this review.
Neither one alone closes #7896. Broader binding/type-resolution and native
compiler/linter migration remain outside this reported enum slice.

The next genuine signed-main union is
`3ed1cc90908c88016310b90a855500c94a156f1f`, descended from actual `8c`.
Its four delivered linter slices add no enum product conflicts; the nineteen
original non-document paths and both original/authored fixture hashes remain
exact. The earlier #8146 status above is historical: its actual merge at
`2026-10-07T13:06:26Z` is `6681b8986ab09e794807cf2764dc0a66779a44e4`,
an ancestor of this genuine main. This establishes the plugin-global delivery
prerequisite for #7896; the enum delivery prerequisite still remains open.
The approved common decision record is applied byte-for-byte at 350 lines,
SHA256 `f4bd4c030fe794c89c7872cdfdcfaa3823d1dc93bdd44295b83fce11637b570d`.
