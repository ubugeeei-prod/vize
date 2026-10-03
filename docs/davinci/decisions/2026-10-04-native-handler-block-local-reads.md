# Original handler block-local reads

Issues: #6839, #6840, #6844. This private next slice follows the actual whole
scriptless DOM provider; it does not change the queued source or admit setup.

Pinned Vue 3.5.35 emits `_ctx.x` for root-local reads, root declarations read
inside a child block, and reads outside or in a sibling of a declaring block.
The original-body writer must keep typed refusals for those cases. It cannot
silently preserve `x` while claiming compatible whole-module output.

Explicit active-block const/let/var references preserve their authored names
in fresh pinned complete component output. The actual L3 On event may admit
only a real borrowed reference from its own complete HandlerResolution table,
with its exact HandlerLocalId, kind, original declaration, active scope chain
and plain decoded/authored UTF-8 name. Non-event locals admit only ordinary Read
references; write/update, shorthand and constructor roles keep typed refusals.
Copied or foreign reference facts and
equal-index local identities confer no authority. Outer/setup reads refuse.
Original stock FunctionBody, comments, profile and decode projections remain
retained. The existing writer, complete body bytes and named spans are unchanged.
No second parser, AST/header/op walk, fresh name lookup, allocation or stage is
introduced; guards inspect only already recorded genuine facts.

A forward lexical reference proves declaration identity, not successful
initialization. The unchanged JavaScript body retains its TDZ ReferenceError.
Forward var reads retain JavaScript's own undefined property value. Const/let/
var, block shadowing, ancestor scopes, repeated var declaration sites, Unicode
entity text and repeated separate original handlers require independent full
component/map/runtime laws. Counterfactual copied/foreign/cross-scope references,
mixed outer reads and unsupported syntax must retain whole-result refusals.

The 31-unit stock guard counts conservative words/punctuation in the wrapped
parser input; it is not a decoded UTF-16 length gate. The Unicode positive
reaches exactly that unchanged boundary. Runtime comparison must execute fresh
pinned reference and actual source-built native modules, including real TDZ
error behavior, updates, handler replacement and cleanup. Native map fields
remain unready until copied from genuine hosted complete capture; missing
capture or incomplete frozen equality fails without skips.

Publication follows literal #7644 integration and a genuine delta replay on
fresh main. Paired issue/central records, exact source Actions, protected full
all-100/instruction acceptance and actual merge are separate required receipts.
No local Cargo/build/install, old fixture mutation or budget waiver is allowed.
Setup/outer access, root/sibling prefixing, default/history migration and broad
handler/For/runtime coverage remain unfinished.

First source efc8f5048 / Check37158867010 fails before native capture because
the new child test module appears before its parent's require macro declaration.
Move only that module registration after the macro, retaining production and
all prior laws. Add independent pinned original declaration/scope geometry to
the existing complete capture law, and require a fresh hosted source; no native
ten-component/runtime credit follows from the failed compilation. The draft
stays unqueued and null maps remain an independently mandatory refusal.

Corrected source f94ff6ffd / Check37159098112 actually captures all ten complete source-built modules/maps in artifact11287315868. Every source/code byte matches the untouched independent pinned component. Freeze only those real maps and remove the temporary Rust null-map branch; complete map equality is unconditional. Independent Node execution of this genuine capture passes all27 laws without skips, including10 native and15 reference components, preserved TDZ ReferenceError, own undefined, separate handlers/update replacements and cleanup. This local Node proof reuses installed pinned dependencies, never a Rust build. New exact-source hosted capture/runtime, protected full/all-100 and actual merge remain required. The final read-only local role gate refuses writes/updates/shorthand/constructors explicitly, retaining the previous EventParameter behavior and unchanged writer.
