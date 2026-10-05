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
