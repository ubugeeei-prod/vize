# Original attached template symbol position queries

Issues: #6836, #6871 and #6883. This bounded provider follows the actual
Handler and OriginalFor File receivers on main `ead89a7592e653a8ed585f4f3fed88fac71b3551`.
The paired central semantic-query paragraph also links the
[current completion audit](./2026-10-04-native-completion-audit.md).
Whole semantic API, LSP/product replacement and fix-history closure remain open.

## Genuine provider and scope

`FileArtifact::template_symbol_at_offset` scans only the private, normally
attached Handler/OriginalFor tables. Actual File completion is required first.
The original event body and For collection/Params owners survive in those rows;
no caller AST/source pair, numeric ID or generic JsExpr coordinate supplies
membership. Queries use the source/decode projection already held by the actual
resolution. They add no parsing, decoding, AST/op walk, index, allocation,
pipeline stage or resident per-node storage. Scan performance is not measured
or advertised as an improvement.

The site borrow carries its actual owner and authored half-open UTF8 name span.
Non-character positions, out-of-bounds offsets and incomplete Files refuse;
EOF, opaque blocks/header text and positions outside these families return
None. Any overlapping observations are ambiguous rather than prioritized by
table order or numeric identity. Script and NativeSfc expression queries keep
their original APIs and boundaries.

`TemplateSymbolRef::File` uses the genuine BindingRef, including real
template-origin aliases. `HandlerLocalRef` keeps its actual FileHandler and
original local binding row. Identical numbers in another handler/File do not
join; `$event` has a real implicit local binding but no authored declaration.
Each Handler declaration site reports its original HandlerDeclaration.scope,
which can differ from the hoisted var binding's scope. HandlerScopeId stays
distinct from File ScopeId.

For collection occurrences retain their original enclosing scope; aliases
retain their actual Params declaration and child scope. A collection can refer
to an enclosing template alias, so no script-unit/declaration fallback is
invented. Its actual introducing scope span does not become body visibility.
Whole original source pointers, normal attachment and source-frame projection
are checked before returning sites. Missing value/key/target records refuse.

`for_each_template_reference_to` rejects a foreign symbol before visiting.
It emits only uses in these attached families; order is unspecified. A late
projection/membership refusal invalidates all prior callback work, which the
caller must discard before publication. A visitor panic mutates no File state.
The provider allocates no result vector or sorted index. A product may combine
the actual script/expression/family APIs and own/sort its final response.

## Lifetime and genuine consumer boundary

Private site/local constructors and real borrow lifetimes prevent fabricated
handler-local rows or use after the File is dropped. The provider never reads
the attached NonNull pointer through unsafe; existing private factory records
and normal File completion establish the attachment. Existing accepts_on and
for_head_for allocation checks remain unchanged.

The currently admitted NativeSfc follows a different normal Component route
which does not mint these selected Handler/For records. Adding new scans there
would create an unreachable facade. Events' genuinely retained
NativeSelectedSfcObservation is the separate required whole-SFC owner for an
actual Maestro retained-worker consumer. It must parse the original descriptor
and script observations once, retain the genuine selected File on worker locals,
and preserve snapshot/version/configuration/cancellation publication guards.
This provider does not claim that consumer or default routing.

## Real source laws and acceptance

Ten laws construct actual Descriptor-selected NativeTemplateOwner outputs,
with original setup observations where required; no fabricated neutral File or
expected-row assembly grants acceptance. They cover:

- local versus outer binding, nested let shadowing, implicit event identity
  and original hoisted-var declaration-site scope;
- complete original entity/escaped Unicode spellings, CRLF/non-BMP UTF16
  projection, half-open names, source holes and invalid UTF8 positions;
- actual For value/key declarations, original Params/resolution membership,
  enclosing collection scope, nested parent-alias collection and shadowing,
  admitted Unicode names and retained entity-output refusals in the
  original alias/collection namespaces;
- foreign equal-byte Files, equal local IDs in separate handlers, incomplete
  late-header outcomes with zero reference callbacks, moved owners and repeated
  queries without arena growth, query order and actual visitor-unwind reuse.

Constructor and owner-drop compile-fail laws accompany the actual provider.
Local validation is source review/rustfmt/pure contracts only under the team's
local-build capacity constraint. Exact-head affected hosted Rust Actions must
compile and execute these real laws. The existing affected/full native-navigation
action remains in place; full queue archive/doctests/differential suites and
unchanged all100 ×3 instruction ceilings/ratchets must pass at the candidate,
followed by the literal protected merge. No skipped test or API-existence
observation receives runtime or product completion credit.

Initial source `ded1549db` compiled and passed affected Clippy, original native
navigation/minimal/strict and doctests, but one of the ten new runtime laws
wrongly expected encoded For names to be admitted. The genuine L1 For family
explicitly refuses `EntityOutput` before File attachment. The correction keeps
that exact authored encoded input and its actual retained syntax/refusal,
requires incomplete File query refusal, and verifies positive name geometry
only for the admitted original Unicode input. The other nine laws passed.
No production grammar or refusal was widened; the superseded head was never
queued. Fresh exact-head hosted and protected acceptance remain required.
