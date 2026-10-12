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

## Observed source failures and required runtime routing

The repaired old-production head `94014812` built successfully under ordinary
Check `38230673353`. Rust shards 1 and 3 stop at overly strict empty-parser-notice
assertions before the unchanged-author-marker and complete slot query/Promise checks.
These failures do not prove those product behaviors before the fix. The full
three build/test logs remain in `preparation/hosted-before-940`. Those PR
Rust shards explicitly set `VIZE_TEST_DISABLE_TSGO=1`; the 25 scoped and four
original typed packets therefore have **no executed before credit**. Preserve
that boundary rather than count successful optional returns as native outcomes.

The production successor is the independently reviewed four private bodies and
six mapping ownership controls. To require actual whole 25+4 outcomes, extend the
already selected `documented_type_aware_examples_match_with_the_required_corsa_runtime`
integration test with a test-only helper using the public `Linter::lint_sfc` path.
It requires Corsa, genuine dependency bytes, all frozen source/config bodies,
complete ordered API/formatter/status packets, and prints all 29 before its final
whole-array assertion. Only physical caller filenames adapt; no target projection
or new oracle is introduced. Existing documented examples remain intact. Reuse
the existing required native Actions command; no workflow, new builder or product
pipeline stage is added. Ordinary optional unit results remain separate from this
required native public API role and from installed release acceptance.

The first production source `61f47f8c` hit the repository's ordinary-module-layout
gate because the test helper used an explicit path attribute. Preserve that full
failure log; move only the helper's whole bytes to `tests/public_expression_reports/mod.rs`
in a move-only commit and use an ordinary `mod` declaration. The reviewed four
production bodies and all original expectations remain byte-exact. No module
layout gate or discovery rule changes.

The next ordinary Rust build at `9eb54b68` rejected the integration helper's private rule import (`E0603`). The helper now imports the same existing public `rules::type_aware::NoUnsafeTemplateBinding` reexport. Production and all frozen whole oracles remain unchanged; compile/runtime qualification still awaits fresh Actions.

Actual required Native run `38235278441` on `adab3a31` produced all 29 complete packets and failed exactly three: scoped/original `DirectiveObjectArg` and scoped `SafeObjectPredicateMultiline`. The other 26 entire packets match the authored oracles, including all 13 unsafe warnings; the complete raw failure and ordered 29 packets are retained in `preparation/hosted-native-adab-*`.

Independent production-source traces show Croquis wraps a first condition in parentheses, and Canon emits `if ((expression))` while mapping the inner complete authored value, including multiline whitespace. The original exact `if (` boundary rejected that owner. The bounded correction allows only balanced extra parentheses around that same complete value, preserving `else if`, compound guards, width mismatch and unmatched delimiters refusal. Actual Canon projections of the two unchanged SFCs now have an explicit source test for their inferred owner and collector query offset. No authored diagnostic/config/vector was changed; fresh whole runtime, protected merge and released installed acceptance remain pending.

The ordinary source run on the same `adab3a31` also revealed that the two static controls rejected the existing Standard-syntax compatibility notices for their unchanged self-closing `<p>` inputs before reaching their behavioral assertions. A separate independently authored oracle now asserts every full notice (code, message and complete location), preserving the original source/query/diagnostic vectors. These are harness repairs, without parser-policy changes or discarded diagnostics. The prior old-source static failures are explicitly not product-before qualification.

The source-length gate rejected growth of the incoming 589-line collector to 601 lines. The unchanged slot-spread offset helper moved in a separate move-only commit to the existing query module; its whole body is preserved, and the collector is 575 lines. No cap, check or pipeline stage changes.

Required native step 13 at `fe3b0dcd` passed the complete original 4 + scoped 25 public packets with actual required Corsa and original dependency bytes. The remaining job and fresh protected qualification are separate. Its ordinary Rust controls then reached two scaffold assumptions: Canon emits each user setup line with two additional spaces (`generator.rs` lines 459 and 580–590), so the two complete literal declarations must be asserted in that emitted form and order; the unchanged expression Promise collector does not descend computed keys of `ObjectExpression` (`calls.rs` lines 375–407), so all four original slot cases have an existing empty Promise vector before and after this fix.

The original `slot-query-direct-controls.json` remains byte-exact (`b7cee3c3…`) as failure history. `slot-existing-promise-contract.json` is an explicit finite correction: it retains every original source/config/NoUnsafe query field and the former unexecuted Promise expectations, authenticates the entire unchanged 17,726-byte producer (`53109ca0…`), and corrects only four before/four after Promise arrays to the existing empty vectors. Runtime tests reconstruct all original case fields and compare the complete current producer with its archived whole body before admitting that correction. Computed-key floating-Promise coverage remains an independently named unimplemented gap; this slice does not claim to improve it or change that rule's production stages. The source failures are preserved in `preparation/hosted-fe3-*`; all 29 urgent native/configured oracles remain unchanged.

Required Native `38237783356` on `fe3b0dcd` is terminal success. The complete raw log (`fdac0ae3…`, 3,658,723 bytes) and all 29 ordered packets are retained in `preparation/hosted-native-fe3-*`; independent full-field comparison against the untouched oracles gives 29/29, the original four empty vectors, and all 13 scoped unsafe warnings. The next change only corrects the two named static harness contracts; every production body and all 29 oracles remain literal. Its own exact source/native/protected gates and public installed release still require qualification.

The exact `746d9eea` source Check `38240067027` and required Native `38240066952` are terminal success. Its full 29 ordered native packets remain exactly equal to the frozen oracles, with all original four empty and 13 unsafe control warnings. Protected candidate `41470a7c` completed n8n, both Nuxt consumers and Musea successfully, but Check `38242094920` failed the existing Oxlint project-transport qualification (job `114787233269`): shared 180-second deadline, both pinned 1.78.0/1.86.0 hosts. Every other Rust/differential/canonical/tooling gate completed without an additional failure. The whole 1,453,528-byte JS log (`8777b5d7…`) is preserved in `preparation/hosted-protected-414-transport-failure.log.gz`. The PR was promptly dequeued and restored to draft; CI owns original source/evidence triage. Keep the actual candidate red, preserve all existing deadlines/checks, and compose a genuine included repair once before fresh protected admission. Actual merge and installed public acceptance remain unfinished.

The source is composed once onto actual main `ace25e4f` after the actual protected merges of shared transport repair #8771 (`a2a54097`) and RustScript repair #8775. All four production bodies, all 29 urgent oracles, dependency/reference custody and legacy fixture bodies remain literal to `746d9eea`; the old protected failure is retained. The foundation preserves all 350 incoming rows with only the reserved paragraph 202 suffix. Shared source transport qualification keeps the original 180-second deadline and complete workloads; it does not establish the old timeout cause or a paired performance gain. Fresh exact-source, required native and protected candidate qualification remains required before actual merge. Installed release acceptance remains separate.
