# Template expression probe ownership for #8752–#8755

Status: source proposal and independently authored corpus; Actions before/after,
protected merge, and included installed release remain pending.

## Reports and boundary

[8752](https://github.com/ubugeeei-prod/vize/issues/8752),
[8753](https://github.com/ubugeeei-prod/vize/issues/8753),
[8754](https://github.com/ubugeeei-prod/vize/issues/8754), and
[8755](https://github.com/ubugeeei-prod/vize/issues/8755) report false unsafe
bindings on public 0.443.0 with Opinionated and type-aware lint. The original
four complete SFCs, issue/config bodies, and public source custody are frozen
under `crates/vize_patina/tests/fixtures/unsafe-template-binding/reports-8752-8755`.
The original `vize check` observations are reporter evidence; they are not a
new installed or current-source qualification.

The first three reports concern a boolean-returning call with an object argument,
a bound literal beginning on a new line, and a multiline ternary with an object
argument in its alternate. The fourth concerns `RouterView v-slot="{ Component }"`
with genuine vue-router 5.3.1. A binding pattern introduces names; it does not
supply an object expression whose whole inferred type should be linted as a value.

## Bounded correction

Use the existing checker-document binding pass and projection rows. Replace only
mapped whole `void` expressions and mapped whole generated handler arguments;
keep parentheses around inferred initializers so comma/object expression grammar
is preserved. Multiline values use their complete owned mapped ranges, rather
than the first newline or a textual closing delimiter. Unmapped author strings
containing generator markers remain byte-exact.

Add an inferred identifier row to the existing mapping with the exact authored
owner range and HOVER/TemplateExpression metadata. The row moves through existing
relative-import, macro-marker and boolean-alias edits. Whole-expression queries
use that owner; nested callees retain their separate positions and unsafe checks.
A property token or a typed synthetic receiver cannot lend its type to the whole
initializer. No new pipeline stage, document field, public config, mode, or
Canon/Corsa producer API is introduced.

Only a complete first `if` condition can gain an alias immediately before its
existing condition in the same lexical scope. Preserve the original condition and
its narrowing. Else-if chains and synthetic enclosing guards remain explicitly
outside this owner transformation; hoisting from either needs its own correct
scope proof. Unequal-width rewritten receivers retain their existing lookup.
This change does not claim universal condition coverage.

For `v-slot`, remove only the newly collected whole-pattern Expression query.
Keep computed-key callee/Promise candidates, descendant values, unsafe sibling
values and their complete order. Existing unparsed destructuring defaults remain
an independently named collector limitation; no blanket slot subtree suppression
or textual any/unknown exemption is permitted.

## Corpus and dependency custody

The 25 independently authored complete scoped API/formatter/status packets contain
four unchanged original inputs, eight additional safe controls, ten unsafe
controls and three mixed-branch owner controls. They include any/unknown values,
callees/returns, CRLF/Unicode, multiline expressions, comma/object grammar,
loop/slot locals, neighboring owner isolation and inert generator-marker strings.
The scoped Incremental rule configuration is explicit. These packets do not
substitute for the separate four original Opinionated/type-aware packets.
Four independent full collector vectors keep slot computed-key callees and
Promise candidates. Source-only ownership controls exercise exact generated
text, full-owner versus nested-token identity, later mapping shifts, same-scope
conditions, refusal cases and handler delimiter ownership.

The Router original uses an isolated private fixture workspace installed by the
existing ordinary Actions install. Pin genuine vue-router 5.3.1 and the existing
Vue production catalog 3.5.42; preserve every incoming lock importer/package/
snapshot body. The additive graph is one importer, one package identity and
two snapshot contexts, with a byte-exact old-lock inverse and complete
resolved dependency closure. Retain the raw broader lock-only proposal as
preparation evidence, but no unrelated existing pin/peer rewrites are adopted.
Official package integrity and complete declaration/license bytes are frozen;
metadata inspection or lock-only validation is not provider execution.

The reporter trailer is supported by the signed public commit
[5e929628](https://github.com/naitokosuke/chirashitabi/commit/5e92962832595edf50c48ca7f24a8c7fed1f4e86):
`ナイトウコウスケ <102337893+naitokosuke@users.noreply.github.com>`,
account id 102337893. Preserve the whole public commit receipt and signed payload.

## Qualification and remaining work

Publish a draft test-first revision with the original production bodies and
normal required-Corsa regressions. Retain its genuine complete before packets
and failure, then publish the reviewed bounded production successor and qualify
its own ordinary/native/protected source. Do not transfer the before run or a
source-only peer receipt into runtime qualification. Existing Actions performs
native builds; no local native build or manual campaign is required.

Legacy fix-history inputs and source preimages also live in
`tests/_fixtures/differential/lint/safe-template-expression-probes-8752`.
Original installed 392/357/443 corpora and their source authorities remain intact;
this corpus is a separate denominator. Theme/transition acquisition and NoMutating
producer ownership stay separate. Broader real-project/default parity remains
unfinished.

Actual merge and a fresh official release including that merge are required
before installed completion. Run all four whole original projects with their
original config and genuine dependencies; preserve full process JSON/status/
stdout/stderr and the authenticated public source/runtime tuple. Keep the third-
party issues open until that released observation succeeds. No Fixes autoclose,
check weakening, provider budget/exclusion change, or upstream write is authorized.

## Initial test-first source gate observation

Draft [8765](https://github.com/ubugeeei-prod/vize/pull/8765), exact `9a699b69`,
ordinary Check `38229343824` refused the initial carrier before runtime regressions.
The owned consumer-surface shard lacked its test-hook line shift and four new
test-only records. Regenerate that sole shard with the exact existing Node
scanner/renderer; no inventory scope/gate/source body is changed. The initial
JS check had zero errors and twenty warnings only in copied official declaration
references. Move those sixteen passive declaration bodies to `.fixture` names in
a move-only commit, retaining all bytes and installed genuine declaration checks.
No lint rule, budget, checker, exclusion, or provider expectation is weakened.
Frozen actual dependency installation passed at that exact hosted head. Nuxt 3
independently stopped at the existing pinned MoonBit Linux HTTP403 transport;
its central source owner remains separate. Preserve all three raw logs and the
first failure. This attempt does not prove any product regression before/after.
