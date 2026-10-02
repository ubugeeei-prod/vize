# Native DOM conditional ownership and dependencies

Paired issues: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839) and
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

L3 owns conditional eligibility, source-order discriminants, branch grouping
and ordered semantic dependency demands. L4 owns their selected-runtime helper
spellings, numeric patch bits, JavaScript syntax and source links. The actual
producer extends its existing `Artifact::visit_events` call; it introduces no
additional stage, parse, AST/tree walk, numbering authority or serialized IR.

`NativeAnalysis::dom()` retains a sparse private conditional table beside the
same sealed L2 owner. Each row borrows the exact retained `IfOp` and `IfBranch`
payloads. A branch records its privately checked u32 source-order discriminant,
the actual event's canonical root NodeId, admitted condition semantics and
native-element block ownership. Consumers receive read-only accessors; callers
cannot manufacture rows or associate a foreign tree with these facts.

The bounded admission is one root If with one conditional branch and an optional
authored final Else, each containing exactly one native HTML element root.
The condition requires complete retained context-reference semantics from the
real resolver and explicitly declared context identities. A foreign AST, span,
source or coordinate capability cannot borrow another table's evidence.
Retained literal conditions deliberately remain unsupported pending an audited
control constant policy. An empty/multiple-root branch, ElseIf chain, nested or
mixed control, template carrier, namespace crossing or special authored key/ref
retains a located typed refusal. Existing body and binding refusals continue to
apply; consumers reject all unsupported facts before producing a fragment.

The neutral conditional factory allows equal or overlapping branch spans and
empty regions. `owner_span` is checked ownership evidence, not branch identity.
At If Enter, private scratch reads actual branch headers and each direct region's
O(1) `ops.len()`. The existing direct-child events consume those counts in
header order, explicitly skip empty headers and retain supplied root ids.
Descendants and attached bindings do not advance the branch cursor. Every
direct child must retain the current header's owner_span, and If Leave must
exhaust the headers. Accounting failures are `DecisionBuildError`; valid but
unadmitted shapes remain `DomRejection`. Branches are not flattened into the
conditional node's ordinary child sequence, and absent fallback creates no
synthetic L2 node or id.

Semantic dependency roles contain no runtime strings, helper-table indices or
numeric Vue flags. The existing events record stable first-use demands:
interpolation display and authored comments at Enter; grouped text values,
class/style normalization and element creation at the original owner Leave;
fragment demands during existing root-group accounting; and conditional
placeholder demand when the actual first branch root closes, after its block
demand and before later branch events. The placeholder demand remains
present with an authored Else, matching the complete pinned Vue module's import
contract. A later branch's display demand cannot move a previously completed
branch's block or placeholder demand. The original whole-If Leave timing
incorrectly put an Else display helper before the placeholder when the first
branch was static. The complete Vue 3.5.35 static-first/Else-display reference
detects that regression and executes both branches against the actual runtime.
L4 maps each role to the selected checked vocabulary and
registers it in the same Writer helper set before append emission; subsequent
uses deduplicate there. This requires no separate helper planning walk.

The ordinary semantic decisions, authored child/binding ordering and Inline
placement remain unchanged. Ordered dependency facts follow the existing
native DOM emitter's child, normalization and owner-closing registration order.
The producer keeps a has-text scratch flag during those same events instead of
scanning child groups again to discover text demands.

Storage review records the sparse conditional side table, borrowed ordered
branch rows, per-open-conditional cursor/direct-root count and stable semantic
dependency vector. The source-sized DOM scratch remains separate from shared
neutral frames and absent for SSR/Vapor. The per-file storage ratchet and
source-derived consumption shards are refreshed without changing classifiers
or any instruction/allocation ceiling. No cost improvement is claimed.

Meaningful laws use the genuine checked element/binding/conditional factories
and complete native resolver tables. They prove equal-span branch ownership,
actual root ids, checked discriminants and condition pointer retention; unused
Else placeholder demand; nested descendant/binding accounting; empty/multiple
root and template/text refusal; foreign/incomplete/literal condition rejection;
ordinary-flow dependency order; and typed incomplete/wrong-owner accounting
failures. Eleven independently captured complete Vue modules and raw reference
maps cover reversed conditional timing, class/style normalization including
literal values, ordinary properties, mixed text, root fragments and root
display; actual retained factory/resolver laws compare the resulting semantic
demands to those references. These dev-only oracles do not count as complete
native module or raw map acceptance. Existing no-control DOM and context laws
remain required. Delivery
adds complete pinned If/Else module comparison and actual Vue 3.5.35 execution
in the dependent L4 consumer, including Recorded/NoLinks output/helper parity.
Those consumer proofs and full current-workspace Actions are separate from
cached-foundation native source laws.

Unfinished: For admission requires a genuine construction-time declaration
registry minting per-alias BindingIds with their actual For owner, ScopeTag and
lexical scope. The existing ForSite/name lookup does not provide those ids.
Collection lookup must see ParentScope before new aliases; body lookup must see
the real local identities and restore nested shadows. L3 never packs tag/name
into invented ids or treats unknown/local names as context. Native BindingRef::Js
formal emission needs its separate checked original-coordinate splice contract;
a retained FormalParameter is not an ExprRef. Full file/script/global binding,
literal condition semantics, remaining control surfaces, complete raw source-map
parity, hoist/cache placement and compiler product selection remain unfinished.
No legacy-backed native shortcut or product route is added. The fix-history
gate #6880, exact-head full/100 Actions and protected native Stack/queue with
actual merge/fresh-main proof still apply.
