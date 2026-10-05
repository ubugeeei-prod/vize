# Original setup dynamic argument ownership

Issue: [#7881](https://github.com/ubugeeei-prod/vize/issues/7881).

## Reservation and source cause

The independent P0 lane owns only #7881; #7970 remains unreserved. The current
open PR inventory and active compiler owner confirm no duplicate source lane.
The base is signed main `b416f850aed501d8bc2f5413a96103710087d173`, after the
actual #8048 merge. Its worktree and source provenance stay unchanged.

The existing directive expression visitor processes values and dynamic model,
slot and custom-directive arguments. It omits dynamic bind/on arguments. Their
shared emitter blindly prefixes simple names with `_ctx.`, losing the same
original setup binding ownership that correctly rewrites their values. Route
these setup-owned arguments through the existing expression transform and emit
transformed arguments once. The guard is the original is-script-setup metadata;
ordinary argument carriers retain their previous processing. Preserve local/slot/v-for ownership, ordinary/static behavior,
helper registration and original spans; add no pipeline stage or serialization.

## Original inputs and acceptance

The complete issue body and original App heredoc are copied byte-for-byte to
`tests/_fixtures/differential/compiler/setup-dynamic-args-7881/`. No original
fixture or expected result is replaced. Full current/reference modules and map
fields, inline/separate modes and original scoped binding controls must be
retained, alongside actual Vue listener/attribute/update outcomes. Existing
Actions own execution; source inspection does not grant runtime credit.

Fresh exact-source canonical/full Rust and protected unchanged 100+4 instruction
ceilings are required before actual signed merge and issue closure. The next
frequent release is owned by the publisher. This existing product repair gives
no native Davinci, browser/hydration, default/history replacement or performance
credit. #6880 remains unfinished.

The verified reporter is `ubugeeei`, public user ID `71201308`. Meaningful
commits retain `Co-authored-by: ubugeeei
<71201308+ubugeeei@users.noreply.github.com>` as a single trailer line.

## Frozen verification recipe

Nine full scripted SFCs include the unchanged App, seven independently authored
controls (constant, maybe-ref, mutable, valid `_`/`$` names, For/slot shadowing,
and static keys), and the genuine Child dependency. Inline and separate modes
retain eighteen complete current public results and eighteen complete official
modules with their original raw script/template map fields. Current maps qualify
script provenance only; independent decoded coordinates do not prove template
semantics. Plain/mapped results must differ only in the public map field.

Actual Vue 3.6.0-rc.9 is loaded from the existing pinned public browser bundle,
with the matching compiler and existing Happy DOM host. Thirty-two mounted
configurations require 132 complete physical child-tree phases, exact ordered
attributes/text, original node retention, listener/attribute updates, zero
warnings/errors and empty unmount. The mutable case changes both dynamic names
and checks that the old listener stops responding. For and slot controls shadow
same-named outer refs; static keys and original non-setup carriers stay intact.
Expected trees are authored from these source contracts, not recaptured from
current output. Complete code/reference/map input and original raw streams,
write/exit/source identity are retained before runtime assertions; partial
reference graphs and phases survive failures.

The reported rc.10 primary processes every dynamic argument with its existing
expression transform. This bounded repair applies only to setup-owned bind/on
arguments. The pinned primary is
[transformExpression.ts](https://github.com/vuejs/core/blob/v3.6.0-rc.10/packages/compiler-core/src/transforms/transformExpression.ts#L84).
Existing Vapor runtime contracts and their original sources are unchanged;
whole Rust/canonical Actions must qualify them. No new workflow/manual campaign
or local native build is introduced. All new Rust/runtime laws are uncompiled
and unexecuted until genuine fresh Actions; pure formatting/lint is distinct.

## Authentic first-source failure and bounded successor

Check37350694819 at source `a0823adbba7e7db5f1af68246cdea63f178a4d72`
executes producer `446593b01ebccf9d8fe21faf2f713505f3ecea54` with the exact
reviewed tree. The new mounted law fails on the unchanged original App: current
initial attributes are empty, while all eight official inline cases preserve
their independently authored 33 phases. The saved packet retains eighteen
complete current modules, nine partial official modules/map graphs, original
exit 1 and raw streams. This run grants no complete runtime acceptance.

The public SFC entry selects the existing L2 DOM emitter. Its independent
dynamic-key consumer still used the original `_ctx.` heuristic, bypassing the
repaired compatibility visitor. Extract the unchanged consumer in a move-only
commit, then use the real retained `JsExpr`, original setup `BindingTable` and
existing expression rewrite/unref recording in that consumer. Propagate exact
expression refusals through both key writers. Ordinary non-setup/default key
spelling remains unchanged; simple local slot/For carriers keep their scope
before setup-ref rewriting. A nine-original-template matrix requires genuine
accepted selected emission and entire independent compatibility code/map bytes,
with both modes and map dispositions. Its explicitly authored metadata equals
the complete current source binding artifacts; the full SFC/runtime law still
qualifies actual script analysis independently.

The shared visitor also reached SSR vnode fallback: the same canonical run
reports eight divergences across all 42,625 original SSR comparisons. Preserve
this failure and every compared output. The repair is specifically VDOM: guard
the new arm against existing SSR/Vapor contexts, with full original carrier
controls, and require a fresh zero-divergence canonical run. No SSR writer or
Vapor/default/reference inputs are changed. The separate direct HTML parser
control retains its entire self-closing button and exact recoverable ExtendPoint
diagnostic; this is valid Vue syntax, not a malformed Vue input. An independently
authored paired-tag counterpart exercises the static-key invariant without that
HTML-parser notice. All original nine SFCs, nineteen custody pins and 132 expected
physical phases remain immutable. Successor Rust/runtime/canonical execution
and all protected 100+4 measurements remain pending.

## Genuine mutable-write failure and same-source repair

Check37354280504 at `29744d8844dc000d7035cde3b57e65c9dc559d68`
executes producer `eb3cbf855cf3904d6c7ec7b224e594b9747dd2df` with tree
`bf3cb06571be96eb5749aec55d63bcdac05d880b`. The unchanged App, Const and
Maybe current inline cases pass every original phase, but Mutable's first click
throws `Cannot create property 'value' on string 'click'`. Preserve original
exit 1, all eighteen complete current module/result/maps, nineteen source pins,
nine partial official modules/graphs and all twelve partial observations:
47 observed phases are not the required complete 132-phase acceptance. Official
worker ZIP11364172943 has SHA256
`e8c26264472b65f9bf29eee926cb68a67506552603ad47bbf0f507ce878fb33d`.
The source role law/selected byte matrix and other three Rust workers pass;
whole current runtime/protected acceptance remains unqualified. The same actual
297 canonical job111912887459 subsequently passes original DOM42609/42625
(with sixteen prior error cases), SSR production42625/42625 and retained
SSR42606/42625 (nineteen prior errors), all with zero divergences. This current
297 result repairs the prior eight SSR differences without granting successor
source or complete runtime acceptance.

The existing identifier collectors assume every inline SetupLet write uses
`.value`. Pinned official rc.10 transformExpression.ts159-192 instead branches
on `isRef` for original simple assignment/update targets. Preserve that bounded
VDOM contract in both actual retained-AST collectors: select only inline,
script-setup SetupLet writes outside lexical/For/slot locals; retain original
assignment operators, parentheses and RHS bytes. Existing child visits already
produce all RHS edits; the same insertion splicer renders their original span
for two exclusive branches, without another AST walk, parser or pipeline stage.
Carry an explicit used-is-ref fact into the real helper imports/order. Nested
branch tails retain the splicer's documented same-position construction order.
The shared SSR/Vapor/string rewrite door leaves this VDOM write role disabled,
and separate/ordinary/default contexts retain their existing bytes. Destructure
writes and unrelated target/history replacement remain outside this correction.

The original Mutable file, complete nine-file corpus, nineteen pins and 132
physical expected phases remain unchanged. Thirteen independently authored
whole expressions require exact generated bytes and 26 genuine pinned-Vue
primitive/ref write observations, including assignment, compound/logical
assignment, nested RHS, side-effect cardinality, both update forms and lexical
shadowing; this is expression semantics, not additional mounted/native credit.
Raw stdin/write error, stdout/stderr and original Node exit are retained before
runtime assertions. Fresh automatic Actions must execute these laws plus all
original full-SFC/runtime/map/canonical comparisons before readiness.

The same source tooling shard4 finds the move-only storage ledger omission:
review Cx's one String import/ten uses and the extracted dynamic-key module's
one import/three uses; the new conditional-write module also has one import and
three uses. Record only these actual per-file sites without classifier, ceiling
or source-policy waiver. Move the unchanged core lexical scope implementation
in its own commit to preserve the existing 350-line source limit. Generated
consumer rows follow these real files. No source/runtime/protected acceptance
transfers from either failed run, and every unchanged 100+4 cap remains required.

## Actual helper retention and first-use custody

Independent source review finds two necessary import boundaries before any
write-source execution. Raw processed expressions do not call codegen helper
registration; retain exactly root-owned IsRef alongside the existing Unref
exception, while unrelated optimized-away helpers remain filtered. The selected
emitter records each actual prefixed result in original order: Unref precedes
IsRef when one result requires both, but a later Unref read cannot move an earlier
IsRef write. Retain the first write's real visit and registration tie, without
source scanning or another traversal. Five independently authored full-template
controls cover static event write/read order, reversed order, both split-element
orders and both helpers in one result; all twenty mode/map combinations require
actual selected admission, whole compatibility modules/maps and the exact complete registration sequence
for the two owned helpers.
The complete original nine-SFC corpus, nineteen pins and 132 mounted phases are
unchanged. These new laws are uncompiled/unexecuted until fresh Actions.

The admitted assignment already materializes two existing-edit worklists for
its RHS splice. Declare those exact inferred buffers as StdVec and record their
one import/two uses under the existing emit category, plus the unchanged String
one import/three uses. This exposes the real allocations without adding or
removing them; no memory, timing or protected instruction gain is claimed.

## Exact source warning-budget failure

The first corrected-source Check37361181662 at
`bff8888bc7582c1a0f3757ffd99ee3c23772679d` fails check-js job111935879270:
typescript(no-implied-eval) flags the new Function constructor in the thirteen
expression observer. Preserve the raw failure and unchanged zero warning
budget. Use node:vm compileFunction in the default current context with the
identical complete generated body and parameters `context`, `asRef`; retain the
same actual pinned Vue ref/isRef/unref functions, thirteen independent expected
objects, all twenty-six required observations and raw failure cleanup. This
changes only the observer's compilation API, without a new realm, source
rewriting or warning suppression. The compiler, original corpus, whole module
map laws and 132 mounted phases stay byte-exact. Fresh exact-source Actions
still owns execution; neither the failed source nor static review qualifies
runtime/protected/merge acceptance.

## Original SetupLet whole snapshots

Exact source `8065f19b080fe39334bb1c01d9c12862f257a5bf` /
Check37362139186 executes signed merge producer
`3cb96e46028b74e794e6776e56fcc06a9def5d5c` / tree
`f35cd9fa6a069240c510c2b5d3ce46b97c573d70`, with parents literal main
`760c5f3f903394cfc5f93743945be534d341ae83` and source8065.
All18 current/18 official modules, all45 complete raw map graphs (2,488
mapped anchors), original19 custody pins and all32 mounts/132 independent
physical phases pass with no diagnostics or cleanup errors. All13 independent
write expressions/twenty-six real pinned Vue primitive/ref outcomes pass.
These current maps establish script provenance only; browser, hydration,
native Davinci and performance remain unqualified.

The four actual Rust workers execute16,248 tests with16,246 passes/two failures
and no skipped cases. Both failures are the first original PKL SetupLet
assignment whole JS/TS snapshots; canonical-corpus job111942832035 passes.
Whole source Check remains failed regardless of these positive laws.
The source-bound receipt has SHA256
`e8634399fda97f16604d512b718ab3f92ca38e95d307562c5d4d614730b379b6`.

Preserve the original `patches.pkl` whole bytes, including `let count = $ref(0)`
and the following `let value = $ref(10)` case. The actual initializer parser
passes these unknown calls to `infer_binding_type`, whose let/var fallback is
SetupLet; `$ref` spelling grants no known-ref authority. The corrected original
AST write role consequently needs the same ref/primitive branch already proved
by the thirteen-expression law. Independently author only four complete golden
refinements, two original inputs times JS/TS: assignment
`(_isRef(count) ? count.value = _unref(count) + 1 : count = _unref(count) + 1)`
and postfix `(_isRef(value) ? value.value++ : value++)`. The outer handler
parentheses and all other module bytes remain exact. Assignment registers
Unref before IsRef from its rewritten RHS; postfix registers IsRef before the
later interpolation's Unref. Preserve original TS event annotations.

The assignment pair is an authenticated current failure. The subsequent
postfix pair is source-proven affected but not an observed failure because
the original whole-fixture test stops at the first mismatch. Archive all four
old complete snapshots, both original raw PKL case blocks and both authentic
JUnit failure bodies in
[the scoped history](./fixtures/setup-write-snapshot-history-7881/authority.json).
Its literal authority pins old/current full bytes and the unchanged ordered
inventory of every other snapshot. No bulk regeneration, source input change,
snapshot comparator waiver, `$ref` reclassification or instruction cap change.
Fresh exact-source full snapshots/runtime/canonical and protected100+4/full
Rust/actual signed merge still qualify this same Draft; prior positive source
execution transfers no acceptance to the successor.

## Hosted runner acquisition and genuine source successor

The composed source `0a47e315468b055aa1bfc7912bd0c9142e86fd05` executes
signed producer `4527835e89ed981bc02c96bd8cd709510d7b1d3f`, whose tree
`4f41ad132e3a3e7f3c04536dc322c412c97a866c` is exactly the source tree.
Check37367937278 does not qualify Rust, the four whole snapshots, the
canonical corpus or the 132 mounted phases: planner111958183532,
fmt111958183369 and dependency111958183290 never acquired a hosted runner.
All three official annotations state that acquisition failed after multiple
attempts; runner identity is zero and no steps executed. Source report
111966067516 correctly fails on planner result `abandoned`; dependent suites
are skipped. Neither this source lane nor the release coordinator cancelled
that current run. Preserve its complete metadata, three annotations and raw
source-report failure under receipt SHA256
`88b21306597b9af5476a4f129f796f0d86aecf7ba3cb027ba78ae31d0db3467f`.
The earlier superseded known-red Check37362139186 cancellation is a separate,
explicitly archived action and grants no success to either source.

The runner correction #8066 actually merges at21:02:06Z as verified signed main
`2902dc98751b408e803ee663aeee494ffab8413b`, with parent literal mainf83.
Incorporate that genuine incoming source once and retain all reviewed compiler,
original corpus, 282 other active snapshots, four approved full refinements
and raw histories. Fresh automatic Actions must execute that new source;
no old hosted-label rerun, skipped-suite success, altered gate or protected
instruction credit follows from this acquisition failure.

## Authentic protected instruction failure and bounded work reduction

Exact source a02 Check37373805875 passes all four required checks, the four
Rust workers (16,253 tests), four refined whole snapshots, canonical corpus,
32 mounts/132 phases, 45 full map graphs and 26 real Vue write outcomes.
Protected candidate641 Check37378371144 nevertheless fails its unchanged
instruction ceiling: `atelier_dom_codegen_stress-wide` measures764,504 in
each of three repeats against745,254. Its literal immediate predecessorf036
measures738,248 in all three repeats with the identical complete input/window.
Remove only this known-red PR from the queue and return it to Draft.

Retained complete Callgrind graphs show fifty plain handler identifiers each
take both function and reference shape scans, together66,700 instructions.
These scan costs are identical in predecessor/current; the additional26,256
instructions include different compiler inlining and string append attribution.
This evidence does not identify a unique source-level cause for that increase.
Reduce actual redundant work within the common `js_module_compatible` gate.
A direct retained Identifier whose full raw bytes equal its lexer-produced
name has no comments, TS fields or nested syntax; return the same existing
strict/module-identifier refusal before raw substring scans and the AST walk.
This identical algorithm runs in every build. Both original handler shape
calls and full cache/reference writes, expression links and battery8/ladder252
dual-run coverage remain unchanged. Escapes, comments, parentheses and every
other AST shape still take the complete old gate. Three independently authored
whole-output/map-link/dialect-boundary laws cover Unicode, member, arrow,
update/statement, strict names, comment markers and escaped spellings.
All original fixtures, nineteen pins, 132 phase oracles, four approved goldens
and budgets remain unchanged. Source inspection proves no measured saving;
fresh exact-source suites and protected all100+4-by-three/full Rust/runtime/
actual signed merge are still required.

The exact eight-line repair is committed as49c47190b4 overa02 with the verified
reporter trailer. Compose genuine verified signed maind43e764ad9233ea40a162a7a49d1a84db54ac591
(parent8e6dabfb66), including actual8058/8044/8057 and both LSP Stack layers,
without conflict. All reviewed production/test objects, original fixtures,
goldens and strict oracles stay unchanged; all incoming non-owned source and
canonical whole-line clauses are retained. Fresh automatic source Actions
own execution of this successor; no old source/protected acceptance transfers.

## Two additional authenticated source and protected failures

Source9b7 Check37381825689 Build112005583932 fails E0277 at the two new
detached handler-test Box calls: GetAllocator is implemented for &Allocator,
so this API requires &&allocator. Fix only those references; the production
identifier gate, all actual input/output vectors and underlying allocator stay
exact. No Rust test execution is credited for this failed build.

The earlier protected641 also fails Clippy job111993650555 at unchanged SFC
coverage165<167: the separate aggregate patches snapshot still has the same
two old unknown-$ref assignment/postfix outputs. All other56/58 patch blocks
pass. Its complete report/raw590025B log is retained under the scoped aggregate
authority. Refine only those two entire output blocks with the independently
reviewed IsRef import and conditional operators; keep their original inputs,
first-three helper order, all other56 blocks and existing comparator exact.
Archive both old complete blocks and pin old/current entire aggregate plus all
unchanged blocks. The original165/167 failure stays failed; no reference is
regenerated from its current output. A genuine ordinary-source integration law
requires all58 ordered identities, complete module comparisons and no error
exemptions. Original coverage floors167/743 and instruction ceilings stay
unchanged. Actual original32/132+26 runtime, whole Rust, full coverage and
protected100+4/signed merge remain required; finite publisher cut is independent.

Source9b7 also fails tooling3 job112005583643: the new retained test adds a
second preferred L0 import site where the existing law requires one. Merge
cstr into the original Allocator import and regenerate the actual inventory;
no import quota changes. Independent review catches two unexecuted custody
premises: input hashes now cover only exact UTF-8 INPUT/OUTPUT delimiter
contents (including LF), and the 58-case law follows original PKL order,
while separately requiring 58 unique one-to-one reference identities and
no error exemptions. Snapshot order and all source/output bytes stay exact.
